use bevy::prelude::*;

pub fn find_matching_child_entity<F: Fn(Entity) -> bool>(
  entity: Entity,
  query_children: &Query<&Children>,
  matcher: F,
) -> Option<Entity> {
  let mut to_check = vec![entity];
  while let Some(current) = to_check.pop() {
      if matcher(current) {
          return Some(current);
      }
      if let Ok(children) = query_children.get(current) {
          to_check.extend(children.iter().copied());
      }
  }
  None
}

pub fn find_matching_parent_entity<F: Fn(Entity) -> bool>(
  entity: Entity,
  query_parent: &Query<&Parent>,
  matcher: F,
) -> Option<Entity> {
  let mut current = entity;
  loop {
      if matcher(current) {
          return Some(current);
      }
      if let Ok(parent) = query_parent.get(current) {
          current = parent.get();
      } else {
          return None;
      }
  }
}

pub fn query_for_children_mut<'a, D: bevy::ecs::query::QueryData, F: bevy::ecs::query::QueryFilter>(
  entity: Entity,
  query_children: &Query<&Children>,
  query: &'a mut Query<D, F>,
) -> Result<D::Item<'a>, bevy::ecs::query::QueryEntityError> {
  let matching_entity =
      find_matching_child_entity(entity, query_children, |e| query.get(e).is_ok())
          .ok_or(bevy::ecs::query::QueryEntityError::NoSuchEntity(entity))?;
  query.get_mut(matching_entity)
}

pub fn query_for_parent<'a, D: bevy::ecs::query::QueryData, F: bevy::ecs::query::QueryFilter>(
  entity: Entity,
  query_parent: &Query<&Parent>,
  query: &'a Query<D, F>,
) -> Result<bevy::ecs::query::ROQueryItem<'a, D>, bevy::ecs::query::QueryEntityError> {
  let matching_entity =
      find_matching_parent_entity(entity, query_parent, |e| query.get(e).is_ok())
          .ok_or(bevy::ecs::query::QueryEntityError::NoSuchEntity(entity))?;
  query.get(matching_entity)
}

pub fn query_for_parent_mut<'a, D: bevy::ecs::query::QueryData, F: bevy::ecs::query::QueryFilter>(
  entity: Entity,
  query_parent: &Query<&Parent>,
  query: &'a mut Query<D, F>,
) -> Result<D::Item<'a>, bevy::ecs::query::QueryEntityError> {
  let matching_entity =
      find_matching_parent_entity(entity, query_parent, |e| query.get(e).is_ok())
          .ok_or(bevy::ecs::query::QueryEntityError::NoSuchEntity(entity))?;
  query.get_mut(matching_entity)
}
