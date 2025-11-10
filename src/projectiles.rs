use crate::game_data::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use std::collections::HashSet;

pub struct ProjectilesPlugin;

impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct ProjectileDisgardInitialSpawnCollisionWith(Vec<Entity>);

#[derive(Component)]
pub struct ProjectileDamage(f32);

fn spawn_projectile(
    commands: &mut Commands,
    position: Vec3,
    direction: Vec2,
    velocity: f32,
    damage: f32,
) {
    commands.spawn((
        GameEntity::LevelEntity,
        Projectile,
        Transform::from_translation(position),
        RigidBody::Dynamic,
        LinearVelocity(direction * velocity),
        ProjectileDamage(damage),
        CollisionEventsEnabled,
        CollidingEntities::default(),
    ));
}

fn spawn_collisions(
    collision_event: On<CollisionEnd>,
    mut projectile_qy: Query<&mut ProjectileDisgardInitialSpawnCollisionWith>,
    mut commands: Commands,
) {
    // Get both collider entities from the event
    let c1 = collision_event.collider1;
    let c2 = collision_event.collider2;

    // Try to remove c2 from c1's discard list if c1 is a projectile with that component
    let mut handled = false;
    if let Ok(mut pd) = projectile_qy.get_mut(c1) {
        pd.0.retain(|&e| e != c2);
        if pd.0.is_empty() {
            commands.entity(c1).remove::<ProjectileDisgardInitialSpawnCollisionWith>();
        }
        handled = true;
    }

    // Also handle the symmetric case: remove c1 from c2's discard list
    if let Ok(mut pd) = projectile_qy.get_mut(c2) {
        pd.0.retain(|&e| e != c1);
        if pd.0.is_empty() {
            commands.entity(c2).remove::<ProjectileDisgardInitialSpawnCollisionWith>();
        }
        handled = true;
    }

    // If neither collider had the component, nothing to do.
    if !handled {
        return;
    }
}

fn spawn_collisions_continious(
    colliding_qy: Query<&CollidingEntities>,
    mut disgard_qy: Query<(Entity, &mut ProjectileDisgardInitialSpawnCollisionWith)>,
    mut commands: Commands,
) {
    // For each projectile that still has an initial-discard list, remove any entries
    // that it is no longer colliding with. If the list becomes empty, remove the component.
    for (entity, mut pd) in disgard_qy.iter_mut() {
        // Build a fast lookup of current colliding entities for this entity
        let current_colliding: HashSet<Entity> = match colliding_qy.get(entity) {
            Ok(colliding) => colliding.iter().copied().collect(),
            Err(_) => HashSet::new(),
        };

        // Keep only entities that are currently colliding with us
        pd.0.retain(|e| current_colliding.contains(e));

        // If nothing left to discard, remove the component
        if pd.0.is_empty() {
            commands.entity(entity).remove::<ProjectileDisgardInitialSpawnCollisionWith>();
        }
    }
}

fn projectile_collision(collision_event: On<CollisionStart>, ) {
    
}