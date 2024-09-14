use killer_critters::{
    basic::*, bevy_tree_query::*, map::*, models::*, player::*, sdf::*, tile::*, tile_factory::*,
};
use bevy::{animation::animate_targets, pbr::CascadeShadowConfigBuilder, prelude::*};
use std::f32::consts::PI;
use web_time::{Duration, Instant};

const PER_FRAME_MOTION: f32 = TILE_SIZE / 20.0;
const EXPLOSION_DURATION: Duration = Duration::from_millis(100);
const BOMB_EXPLOSION_DELAY: Duration = Duration::from_secs(3);
const FREE_SPACE_BORDER: f32 = 0.4;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Setup,
    Playing,
    GameOver,
}

const MAP_DIMENSIONS: (usize, usize) = (19, 13);

#[derive(Resource)]
struct AudioAssets {
    explosion_sound: Handle<AudioSource>,
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    let window = Window {
        canvas: Some("#canvas_killer_critters".into()),
        fit_canvas_to_parent: true,
        ..default()
    };
    #[cfg(not(target_arch = "wasm32"))]
    let mut window = Window::default();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // if cmdline arg --help, then print help
        if std::env::args().any(|arg| arg == "--help") {
            println!("Usage: killer-critters [--fullscreen]");
            std::process::exit(0);
        }

        // if cmdline arg --fullscreen, then set fullscreen
        if std::env::args().any(|arg| arg == "--fullscreen") {
            window.mode = bevy::window::WindowMode::BorderlessFullscreen(
                bevy::window::MonitorSelection::Primary,
            );
        }
    }

    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .add_systems(Startup, setup_once)
        .add_systems(OnEnter(GameState::Playing), setup_per_game)
        .add_systems(
            Update,
            (
                setup_scene_once_loaded.before(animate_targets),
                keyboard_control,
                keyboard_debug,
                gamepad_events,
                update_tile_graphics,
                map_transitions,
                check_for_death,
                check_for_win,
                check_pickup,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), game_over)
        .add_systems(Update, restart_game.run_if(in_state(GameState::GameOver)))
        .init_state::<GameState>()
        .run();
}

fn setup_once(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    commands.spawn(AudioBundle {
            source: asset_server.load("sound/Vicious.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                ..default()
            },
        });

    let audio_assets = AudioAssets {
        explosion_sound: asset_server.load("sound/smoke-bomb-6761.ogg"),
    };
    commands.insert_resource(audio_assets);

    commands.insert_resource(ResourceTileFactory::new(
        &mut animation_graphs,
        &asset_server,
    ));

    // Camera
    let map_center = Vec3::new(
        (MAP_DIMENSIONS.0 as f32 - 1.0) * TILE_SIZE / 2.0,
        0.0,
        (MAP_DIMENSIONS.1 as f32 - 1.0) * TILE_SIZE / 2.0,
    );
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(map_center + Vec3::new(0.0, 20.0, 1.0))
            .looking_at(map_center, Vec3::Y),
        ..default()
    });

    // Plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(500.0, 500.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // Animal
    for i in 0..2 {
        commands.spawn((
            SceneBundle {
                scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(MODEL_ANIMAL_PATH[i])),
                ..default()
            },
            Player {
                controller: match i {
                    0 => PlayerController::KeyboardArrows,
                    1 => PlayerController::KeyboardWASD,
                    2 => PlayerController::Gamepad(0),
                    _ => PlayerController::Gamepad(1),
                },
                ..Default::default()
            },
            animation_graphs.add(AnimalAnimation::load_graph(
                &asset_server,
                MODEL_ANIMAL_PATH[i],
            )),
        ));
    }

    next_state.set(GameState::Playing);
}

fn setup_per_game(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Transform, &mut Player)>,
) {
    // Recreate the map
    let map_component = make_basic_map(&mut commands, MAP_DIMENSIONS.0, MAP_DIMENSIONS.1);
    let starting_positions = map_component.spawn_points().to_vec();

    commands.spawn((
        map_component,
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    // Reset character positions
    let mut starting_positions = starting_positions.iter();
    for (entity, mut transform, mut player) in &mut player_query {
        let starting_position = starting_positions.next().unwrap();
        transform.translation =
            Vec3::new(starting_position.x as f32, 0.0, starting_position.y as f32);
        commands.entity(entity).insert(Alive {});
        player.num_bombs = Player::default().num_bombs;
        player.firepower = Player::default().firepower;
    }
}

// An `AnimationPlayer` is automatically added to the scene when it's ready.
// When the player is added, start the animation.
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Query<&Handle<AnimationGraph>>,
    query_parent: Query<&Parent>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        if let Ok(graph) = query_for_parent(entity, &query_parent, &animations) {
            transitions
                .play(&mut player, AnimationNodeIndex::new(1), Duration::ZERO)
                .repeat();

            commands
                .entity(entity)
                .insert(graph.clone())
                .insert(transitions);
        } else {
            println!(
                "No AnimationGraph component found for entity with AnimationPlayer (`{}`).",
                entity
            );
        }
    }
}

fn play_animation(
    scene_entity: Entity,
    target_animation: AnimationNodeIndex,
    transition_duration: Duration,
    repeat: bool,
    query_children: &Query<&Children>,
    query_transitions: &mut Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
) {
    if let Ok((mut transitions, mut player)) =
        query_for_children_mut(scene_entity, &query_children, query_transitions)
    {
        let current_animation = transitions.get_main_animation();
        if current_animation != Some(target_animation) {
            let anim = transitions.play(&mut player, target_animation, transition_duration);
            if repeat {
                anim.repeat();
            }
        }
    }
}

fn vec3_xz(v: Vec2) -> Vec3 {
    Vec3::new(v.x, 0.0, v.y)
}

fn control_player(
    control: &PlayerControl,
    player: (Entity, &mut Transform, &mut Player),
    query_children: &Query<&Children>,
    maps: &Query<(&Transform, &Map), Without<Player>>,
    tiles: &mut ParamSet<(Query<&Tile>, Query<&mut Tile>)>,
    query_transitions: &mut Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
) {
    let (player_entity, parent_from_frame, player) = player;

    let cur_pos_in_world = parent_from_frame.translation.xz();
    let mut new_pos_in_world = cur_pos_in_world + PER_FRAME_MOTION * control.motion;

    for (map_transform, map) in maps {
        let cur_pos_in_map = cur_pos_in_world - map_transform.translation.xz();
        let new_pos_in_map = new_pos_in_world - map_transform.translation.xz();

        let icur_pos_in_map = map.get_index_from_position(cur_pos_in_map);
        let inew_pos_in_map =
            map.get_index_from_position(new_pos_in_world - map_transform.translation.xz());

        if let (Some(icur_pos_in_map), Some(mut inew_pos_in_map)) =
            (icur_pos_in_map, inew_pos_in_map)
        {
            let cur_tile_is_bomb = matches!(
                tiles.p0().get(map[icur_pos_in_map]).unwrap().tile_type,
                TileType::Bomb(_)
            );

            let new_sdf = map_sdf(
                map,
                new_pos_in_map,
                &tiles.p0(),
                if cur_tile_is_bomb {
                    Some(icur_pos_in_map)
                } else {
                    None
                },
            );

            if -FREE_SPACE_BORDER < new_sdf.0 && new_sdf.0 < 0.0 {
                new_pos_in_world = cur_pos_in_world + new_sdf.1 * PER_FRAME_MOTION;
                inew_pos_in_map = map
                    .get_index_from_position(new_pos_in_world - map_transform.translation.xz())
                    .unwrap();
            }

            let tile_entity = map[inew_pos_in_map];
            if let Ok(mut tile) = tiles.p1().get_mut(tile_entity) {
                if control.action == PlayerAction::DropBomb && tile.tile_type == TileType::Empty {
                    if player.num_bombs > 0 {
                        tile.tile_type = TileType::Bomb(Some(Bomb {
                            when_to_explode: Instant::now() + BOMB_EXPLOSION_DELAY,
                            firepower: player.firepower,
                            player_entity: player_entity,
                        }));
                        tile.set_changed();
                        player.num_bombs -= 1;
                    }
                }
            }
        }
    }

    parent_from_frame.translation = vec3_xz(new_pos_in_world);

    let mut target_animation = AnimalAnimation::Idle.to_node_index();

    if control.motion.length() > 0.0 {
        let dir2d = control.motion.normalize();
        let roty = Quat::from_rotation_y(dir2d.x.atan2(dir2d.y));
        parent_from_frame.rotation = roty;

        target_animation = AnimalAnimation::Run.to_node_index();
    }

    play_animation(
        player_entity,
        target_animation,
        Duration::from_millis(400),
        true,
        query_children,
        query_transitions,
    );
}

fn keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    maps: Query<(&Transform, &Map), Without<Player>>,
    query_children: Query<&Children>,
    mut tiles: ParamSet<(Query<&Tile>, Query<&mut Tile>)>,
    mut query_player: Query<(Entity, &mut Transform, &mut Player), Without<Map>>,
    mut query_transitions: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
) {
    let mut control_arrows = PlayerControl {
        motion: Vec2::ZERO,
        action: PlayerAction::None,
    };
    let mut control_wasd = PlayerControl {
        motion: Vec2::ZERO,
        action: PlayerAction::None,
    };

    {
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            control_arrows.motion.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            control_arrows.motion.y += 1.0;
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            control_arrows.motion.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            control_arrows.motion.x += 1.0;
        }

        if keyboard_input.just_pressed(KeyCode::Space) {
            control_arrows.action = PlayerAction::DropBomb;
        }

        if control_arrows.motion.length() > 0.0 {
            control_arrows.motion = control_arrows.motion.normalize();
        }
    }
    {
        if keyboard_input.pressed(KeyCode::KeyW) {
            control_wasd.motion.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            control_wasd.motion.y += 1.0;
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            control_wasd.motion.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            control_wasd.motion.x += 1.0;
        }

        if keyboard_input.just_pressed(KeyCode::KeyQ) {
            control_wasd.action = PlayerAction::DropBomb;
        }

        if control_wasd.motion.length() > 0.0 {
            control_wasd.motion = control_wasd.motion.normalize();
        }
    }

    for (entity, mut transform, mut player) in &mut query_player {
        if player.controller == PlayerController::KeyboardArrows {
            control_player(
                &control_arrows,
                (entity, &mut transform, &mut player),
                &query_children,
                &maps,
                &mut tiles,
                &mut query_transitions,
            );
        } else if player.controller == PlayerController::KeyboardWASD {
            control_player(
                &control_wasd,
                (entity, &mut transform, &mut player),
                &query_children,
                &maps,
                &mut tiles,
                &mut query_transitions,
            );
        }
    }
}

fn keyboard_debug(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    maps: Query<(&Transform, &Map), Without<Player>>,
    mut tiles: ParamSet<(Query<&Tile>, Query<&mut Tile>)>,
) {
    for (_, map) in &maps {
        if keyboard_input.just_pressed(KeyCode::KeyM) {
            spawn_sdf_texture(&mut commands, &mut images, map, &tiles.p0());
        }
    }
}

fn gamepad_events(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<ButtonInput<GamepadButton>>,
    maps: Query<(&Transform, &Map), Without<Player>>,
    query_children: Query<&Children>,
    mut tiles: ParamSet<(Query<&Tile>, Query<&mut Tile>)>,
    mut query_player: Query<(Entity, &mut Transform, &mut Player), Without<Map>>,
    mut query_transitions: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
) {
    let mut control = PlayerControl {
        motion: Vec2::ZERO,
        action: PlayerAction::None,
    };

    for axis in axes.devices() {
        let value = axes.get(*axis).unwrap();
        match axis.axis_type {
            GamepadAxisType::LeftStickX => control.motion.x += value,
            GamepadAxisType::LeftStickY => control.motion.y -= value,
            _ => {}
        }
    }

    if buttons.just_pressed(GamepadButton {
        gamepad: Gamepad::new(0),
        button_type: GamepadButtonType::South,
    }) {
        control.action = PlayerAction::DropBomb;
    }

    for (entity, mut transform, mut player) in &mut query_player {
        if player.controller == PlayerController::Gamepad(0) {
            control_player(
                &control,
                (entity, &mut transform, &mut player),
                &query_children,
                &maps,
                &mut tiles,
                &mut query_transitions,
            );
        }
    }
}

fn update_tile_graphics(
    mut commands: Commands,
    mut query_tiles: Query<(Entity, &Tile, &Transform), Changed<Tile>>,
    game_assets: Res<ResourceTileFactory>,
) {
    for (entity, tile, transform) in &mut query_tiles {
        // save the transform
        let transform = transform.clone();

        // Removing PbrBundle and SceneBundle
        commands.entity(entity).remove::<PbrBundle>();
        commands.entity(entity).remove::<SceneBundle>();

        // Insert the correct bundle and replace the transform that was removed
        // when we removed the PbrBundle and SceneBundle
        match game_assets.make_tile(&tile.tile_type) {
            GameAsset::None => {}
            GameAsset::Scene(scene) => {
                commands.entity(entity).insert(scene.clone());
            }
            GameAsset::AnimatedScene(scene) => {
                commands.entity(entity).insert(scene.clone());
            }
        };
        let transform = Transform::from_translation(transform.translation);

        if let TileType::BreakableWall(_) = tile.tile_type {
            let r = random_orthogonal_rotation();
            commands.entity(entity).insert(transform.with_rotation(r));
        } else {
            commands.entity(entity).insert(transform);
        }
    }
}

fn map_transitions(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    mut maps: Query<&Map>,
    mut tiles: Query<&mut Tile>,
    mut query_player: Query<&mut Player>,
) {
    for map in &mut maps {
        for pos in map.pos_iter() {
            let tile_entity = map[pos];
            let mut tile_data = tiles.get_mut(tile_entity).unwrap();

            match tile_data.tile_type.clone() {
                TileType::Bomb(Some(bomb)) => {
                    if Instant::now() > bomb.when_to_explode {
                        commands.spawn((
                            AudioBundle {
                                source: audio_assets.explosion_sound.clone(),
                                settings: PlaybackSettings::DESPAWN,
                                ..default()
                            },
                        ));

                        // Explode the bomb
                        tile_data.tile_type = TileType::Explosion(
                            Some(Instant::now() + EXPLOSION_DURATION),
                            Box::new(TileType::Empty),
                        );

                        // increase a bomb counter for the player that placed the bomb
                        if let Ok(mut player) = query_player.get_mut(bomb.player_entity) {
                            player.num_bombs += 1;
                        }

                        for dir in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                            for dist in 1..=bomb.firepower {
                                let pos = pos + IVec2::new(dir.0 * dist, dir.1 * dist);
                                if !map.contains(pos) {
                                    continue;
                                }
                                let tile_entity = map[pos];
                                let mut tile_data = tiles.get_mut(tile_entity).unwrap();

                                match &mut tile_data.tile_type {
                                    TileType::Empty | TileType::PowerUp(_) => {
                                        tile_data.tile_type = TileType::Explosion(
                                            Some(Instant::now() + EXPLOSION_DURATION),
                                            Box::new(TileType::Empty),
                                        );
                                    }
                                    TileType::BreakableWall(contents) => {
                                        tile_data.tile_type = TileType::Explosion(
                                            Some(Instant::now() + EXPLOSION_DURATION),
                                            contents.clone(),
                                        );
                                        break; // don't go through walls
                                    }
                                    TileType::SolidWall => {
                                        break; // don't go through walls
                                    }
                                    TileType::Bomb(Some(other_bomb)) => {
                                        other_bomb.when_to_explode = Instant::now();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                TileType::Explosion(Some(explosion_time), contents) => {
                    if Instant::now() > explosion_time {
                        tile_data.tile_type = *contents.clone();
                    }
                }
                _ => {}
            }
        }
    }
}

fn check_for_death(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Transform, &Player), Without<Map>>,
    maps: Query<(&mut Transform, &Map), Without<Player>>,
    tiles: Query<&Tile>,
) {
    for (player_entity, transform, _player) in &mut players {
        let pos = transform.translation.xz();
        for (map_transform, map) in &maps {
            let pos_in_map = map.get_index_from_position(pos - map_transform.translation.xz());
            if let Some(pos) = pos_in_map {
                let tile_entity = map[pos];
                let tile = tiles.get(tile_entity).unwrap();
                if let TileType::Explosion(_, _) = tile.tile_type {
                    // Remove alive component
                    commands.entity(player_entity).remove::<Alive>();
                }
            }
        }
    }
}

fn check_for_win(
    mut next_state: ResMut<NextState<GameState>>,
    query_player: Query<(Entity, &Transform, &Player), With<Alive>>,
    query_children: Query<&Children>,
    mut query_transitions: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
) {
    let player_count = query_player.iter().count();
    if player_count == 1 {
        if let Ok((player_entity, _, _)) = query_player.get_single() {
            let target_animation = AnimalAnimation::Jump.to_node_index();

            play_animation(
                player_entity,
                target_animation,
                Duration::from_millis(400),
                true,
                &query_children,
                &mut query_transitions,
            );

            // Transition to GameOver state
            next_state.set(GameState::GameOver);
        }
    } else if player_count == 0 {
        // Draw
        next_state.set(GameState::GameOver);
    }
}

fn game_over(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        TextBundle::from_section(
            "Game Over! Press SPACE to restart",
            TextStyle {
                font: asset_server.load("fonts/Handjet/Handjet-Medium.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

fn restart_game(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    tiles: Query<Entity, With<Tile>>,
    maps: Query<Entity, With<Map>>,
    text: Query<Entity, With<Text>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Despawn maps and tiles
        for entity in tiles.iter().chain(maps.iter()) {
            commands.entity(entity).despawn();
        }

        // despawn game-over text
        for entity in text.iter() {
            commands.entity(entity).despawn();
        }

        // Transition back to Playing state
        next_state.set(GameState::Playing);
    }
}

fn check_pickup(
    mut query_players: Query<(&Transform, &mut Player), Without<Map>>,
    mut query_tiles: Query<&mut Tile>,
    query_maps: Query<(&mut Transform, &Map), Without<Player>>,
) {
    for (transform, mut player) in &mut query_players {
        let pos = transform.translation.xz();
        for (map_transform, map) in &query_maps {
            let pos_in_map = map.get_index_from_position(pos - map_transform.translation.xz());
            if let Some(pos) = pos_in_map {
                let tile_entity = map[pos];
                let mut tile = query_tiles.get_mut(tile_entity).unwrap();
                match tile.tile_type {
                    TileType::PowerUp(PowerUpType::Firepower) => {
                        player.firepower += 1;
                        tile.tile_type = TileType::Empty;
                    }
                    TileType::PowerUp(PowerUpType::ExtraBomb) => {
                        player.num_bombs += 1;
                        tile.tile_type = TileType::Empty;
                    }
                    _ => {}
                }
            }
        }
    }
}
