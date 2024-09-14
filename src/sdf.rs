use bevy::prelude::*;
use zerocopy::AsBytes;
use crate::map::Map;
use crate::tile::*;

fn is_walkable(tile: &Tile) -> bool {
  matches!(tile.tile_type, TileType::Empty | TileType::PowerUp(_))
}

pub fn closest_dist_to_tile(pos_in_map: Vec2, map_tile_index: IVec2) -> f32 {
  let tile_min = map_tile_index.as_vec2() - Vec2::splat(0.5);
  let tile_max = tile_min + Vec2::ONE;

  let closest = pos_in_map.clamp(tile_min, tile_max);
  (pos_in_map - closest).length()
}

// The extra_free_in_map is a bit of a hack to handle the case where the player
// is standing on a tile containing their own bomb that they have just placed.
pub fn map_sdf(map: &Map, pos_in_map: Vec2, tiles: &Query<&Tile>, extra_free_in_map: Option<IVec2>) -> (f32, Vec2) {
  let center_index = map.get_index_from_position(pos_in_map).unwrap_or_default();
  let rad = 2; // Adjust this radius as needed

  let center_tile = tiles.get(map[center_index]).ok();
  let is_in_empty = center_tile.map_or(true, |t| is_walkable(t) || (extra_free_in_map.is_some_and(|extra_free| extra_free == center_index)));

  let (sdf, closest_point) = (-rad..=rad)
      .flat_map(|dy| (-rad..=rad).map(move |dx| IVec2::new(dx, dy)))
      .filter_map(|offset| {
          let index = center_index + offset;
          if map.contains(index) {
              tiles.get(map[index]).ok().and_then(|tile| {
                  let is_empty = is_walkable(tile) || (extra_free_in_map.is_some_and(|extra_free| extra_free == index));
                  if is_empty != is_in_empty {
                      let dist = closest_dist_to_tile(pos_in_map, index);
                      Some((dist, index.as_vec2()))
                  } else {
                      None
                  }
              })
          } else {
              None
          }
      })
      .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
      .map(|(dist, point)| (if is_in_empty { -dist } else { dist }, point))
      .unwrap_or((if is_in_empty { -2.0 } else { 2.0 }, pos_in_map));

  let derivative = if sdf != 0.0 {
      (pos_in_map - closest_point).normalize()
  } else {
      Vec2::ZERO
  };

  (sdf, derivative)
}

pub fn jet_colormap(sdf: f32) -> Color {
  let x = (sdf + 1.0) / 2.0; // Normalize sdf to [0, 1] range
  let x = x.clamp(0.0, 1.0); // Ensure x is within [0, 1]

  if x < 0.25 {
      Color::srgb(0.0, 4.0 * x, 1.0)
  } else if x < 0.5 {
      Color::srgb(0.0, 1.0, 1.0 + 4.0 * (0.25 - x))
  } else if x < 0.75 {
      Color::srgb(4.0 * (x - 0.5), 1.0, 0.0)
  } else {
      Color::srgb(1.0, 1.0 + 4.0 * (0.75 - x), 0.0)
  }
}

pub fn map_sdf_image(map: &Map, tiles: &Query<&Tile>) -> Image {
  let scale = 40;
  let width = (map.width() * scale) as u32;
  let height = (map.height() * scale) as u32;
  let mut sdfs = vec![0.0; (width * height) as usize];
  let mut data = vec![0.0; (width * height * 4) as usize];

  for y in 0..height {
      for x in 0..width {
          let pos = Vec2::new(x as f32 / scale as f32 - 0.5, y as f32 / scale as f32 - 0.5);
          let sdf = map_sdf(map, pos, tiles, None);
          let idx = (y * width + x) as usize;
          sdfs[idx] = sdf.0;
      }
  }

  // let (min, max) = sdfs.iter().fold((f32::MAX, f32::MIN), |(min, max), &sdf| {
  //     (min.min(sdf), max.max(sdf))
  // });
  let (min, max) = (-1.0, 1.0);

  println!("min: {}, max: {}", min, max);

  let scale = 1.0 / (max - min);

  for (i, sdf) in sdfs.iter().enumerate() {
      let idx = i * 4;
      let val = (sdf - min) * scale;
      match jet_colormap(val) {
          Color::Srgba(color) => {
              data[idx] = color.red;
              data[idx + 1] = color.green;
              data[idx + 2] = color.blue;
              data[idx + 3] = color.alpha;
          }
          _ => {
          }
      };
  }

  let img = Image::new(
      bevy::render::render_resource::Extent3d {
          width,
          height,
          depth_or_array_layers: 1,
      },
      bevy::render::render_resource::TextureDimension::D2,
      data.as_bytes().to_vec(),
      bevy::render::render_resource::TextureFormat::Rgba32Float,
      bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
          | bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD,
  );


  img
}

pub fn spawn_sdf_texture(
  commands: &mut Commands,
  images: &mut ResMut<Assets<Image>>,
  map: &Map,
  tiles: &Query<&Tile>,
) {
  let img = map_sdf_image(map, tiles);
  let texture_handle = images.add(img);

  // Create a new camera for 2D overlay
  commands.spawn((Camera2dBundle {
      camera: Camera {
          order: 1, // Render after the main camera
          ..default()
      },
      ..default()
  },));

  // Spawn the sprite with the SDF texture
  commands.spawn(SpriteBundle {
      texture: texture_handle,
      transform: Transform::from_xyz(0.0, 0.0, 1.0), // Adjust Z to be slightly in front
      sprite: Sprite {
          color: Color::WHITE,
          // custom_size: Some(Vec2::new(640.0, 360.0)), // Match your window size
          ..default()
      },
      ..default()
  });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closest_dist_to_tile() {
        // Test case 1: Point inside the tile
        let pos = Vec2::new(1.5, 1.5);
        let tile = IVec2::new(1, 1);
        assert_eq!(closest_dist_to_tile(pos, tile), 0.0);

        // Test case 2: Point on the edge of the tile
        let pos = Vec2::new(2.0, 1.5);
        let tile = IVec2::new(1, 1);
        assert_eq!(closest_dist_to_tile(pos, tile), 0.0);

        // Test case 3: Point outside the tile
        let pos = Vec2::new(2.5, 2.5);
        let tile = IVec2::new(1, 1);
        assert!((closest_dist_to_tile(pos, tile) - 0.7071067).abs() < 1e-6);

        // Test case 4: Point far from the tile
        let pos = Vec2::new(5.0, 5.0);
        let tile = IVec2::new(1, 1);
        assert!((closest_dist_to_tile(pos, tile) - 4.242641).abs() < 1e-6);

        // Test case 5: Point within x-range but above the tile
        let pos = Vec2::new(1.5, 0.5);
        let tile = IVec2::new(1, 1);
        assert_eq!(closest_dist_to_tile(pos, tile), 0.5);

        // Test case 6: Point within x-range but below the tile
        let pos = Vec2::new(1.5, 2.5);
        let tile = IVec2::new(1, 1);
        assert_eq!(closest_dist_to_tile(pos, tile), 0.5);

        // Test case 7: Point within y-range but left of the tile
        let pos = Vec2::new(0.5, 1.5);
        let tile = IVec2::new(1, 1);
        assert_eq!(closest_dist_to_tile(pos, tile), 0.5);

        // Test case 8: Point within y-range but right of the tile
        let pos = Vec2::new(2.5, 1.5);
        let tile = IVec2::new(1, 1);
        assert_eq!(closest_dist_to_tile(pos, tile), 0.5);
    }

    // // Helper function to create a test app
    // fn create_test_app() -> App {
    //     let mut app = App::new();
    //     app.add_plugins(MinimalPlugins)
    //     // .add_systems(Setup, setup_test)
    //     .add_systems(Update, test_map_sdf);
    //     app
    // }

    // fn setup_test(
    //     mut commands: Commands,
    // ) {
    // }

    // // System to test map_sdf
    // fn test_map_sdf(
    //     mut commands: Commands,
    //     tiles: Query<&Tile>,
    //     mut exit: EventWriter<AppExit>,
    // ) {
    //     let map = make_basic_map(&mut commands, 3, 3);
    //     let pos_in_map = Vec2::new(2.5, 2.5);
    //     let sdf_value = map_sdf(&map, pos_in_map, &tiles);

    //     // Perform your assertions here
    //     assert_eq!(sdf_value, 0.0);

    //     // Exit the app after the test
    //     exit.send(AppExit::Success);
    // }

    // #[test]
    // fn run_map_sdf_test() {
    //     let mut app = create_test_app();
    //     app.update();
    // }
}
