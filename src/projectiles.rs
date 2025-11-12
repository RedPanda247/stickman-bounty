use crate::game_data::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use std::collections::HashSet;

pub struct ProjectilesPlugin;

impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedLast, spawn_collisions_continious)
            .add_observer(spawn_collisions)
            .add_observer(projectile_collision);
    }
}

#[derive(Component)]
struct Projectile {
    damage: f32,
    knockback: f32,
}

#[derive(Component)]
struct ProjectileDisgardInitialSpawnCollisionWith(Vec<Entity>);

pub fn spawn_projectile(
    commands: &mut Commands,
    position: Vec3,
    direction: Vec2,
    velocity: f32,
    damage: f32,
    knockback: f32,
    disgard_initial_collision_with: Vec<Entity>,
) {
    let projectile_size = 10.;
    commands.spawn((
        GameEntity::LevelEntity,
        Projectile { damage, knockback },
        ProjectileDisgardInitialSpawnCollisionWith(disgard_initial_collision_with),
        Transform::from_translation(position),
        RigidBody::Dynamic,
        GravityScale(0.),
        LinearVelocity(direction * velocity),
        Collider::rectangle(projectile_size, projectile_size),
        CollisionEventsEnabled,
        CollidingEntities::default(),
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(projectile_size, projectile_size)),
            ..default()
        },
    ));
}

fn spawn_collisions(
    collision_event: On<CollisionEnd>,
    mut projectile_qy: Query<&mut ProjectileDisgardInitialSpawnCollisionWith>,
    mut commands: Commands,
) {
    // Get both collider entities from the event
    let collision_entity_1 = collision_event.collider1;
    let collision_entity_2 = collision_event.collider2;

    // Try to remove c2 from c1's discard list if c1 is a projectile with that component
    let mut handled = false;
    if let Ok(mut pd) = projectile_qy.get_mut(collision_entity_1) {
        pd.0.retain(|&e| e != collision_entity_2);
        if pd.0.is_empty() {
            commands
                .entity(collision_entity_1)
                .remove::<ProjectileDisgardInitialSpawnCollisionWith>();
        }
        handled = true;
    }

    // Also handle the symmetric case: remove c1 from c2's discard list
    if let Ok(mut pd) = projectile_qy.get_mut(collision_entity_2) {
        pd.0.retain(|&e| e != collision_entity_1);
        if pd.0.is_empty() {
            commands
                .entity(collision_entity_2)
                .remove::<ProjectileDisgardInitialSpawnCollisionWith>();
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
            commands
                .entity(entity)
                .remove::<ProjectileDisgardInitialSpawnCollisionWith>();
            println!("removed disgard list");
        }
    }
}

#[derive(EntityEvent)]
struct ProjectileHitEvent {
    entity: Entity,
    damage: f32,
    knockback_impulse: Vec2,
}

fn projectile_collision(
    collision_event: On<CollisionStart>,
    projectile_qy: Query<(&Projectile, &LinearVelocity)>,
    hit_entity_qy: Query<(&CanBeHitByProjectile)>,
    disgard_initial_collision_qy: Query<&ProjectileDisgardInitialSpawnCollisionWith>,
    mut commands: Commands,
) {
    // Get both collider entities from the event
    let collision_entity_1 = collision_event.collider1;
    let collision_entity_2 = collision_event.collider2;

    // Check both orderings: (entity1 as projectile, entity2 as target) and vice versa
    check_projectile_hit(
        collision_entity_1,
        collision_entity_2,
        &projectile_qy,
        &hit_entity_qy,
        &disgard_initial_collision_qy,
        &mut commands,
    );
}

fn check_projectile_hit(
    projectile_entity: Entity,
    hit_entity: Entity,
    projectile_qy: &Query<(&Projectile, &LinearVelocity)>,
    hit_entity_qy: &Query<(&CanBeHitByProjectile)>,
    disgard_initial_collision_qy: &Query<&ProjectileDisgardInitialSpawnCollisionWith>,
    commands: &mut Commands,
) {
    if let Ok((projectile, linvel)) = projectile_qy.get(projectile_entity) {
        if let Ok(_) = hit_entity_qy.get(hit_entity) {
            // If the projectile has the "disgard collision with" component
            // check if the other entity is inside that list and if it is
            // disgard this collision
            if let Ok(disgard_collision_with) = disgard_initial_collision_qy.get(projectile_entity)
            {
                if disgard_collision_with.0.contains(&hit_entity) {
                    return;
                }
            }
            println!("projectile collision occurred");
            commands.trigger(ProjectileHitEvent {
                entity: hit_entity,
                damage: projectile.damage,
                knockback_impulse: linvel.0.normalize() * projectile.knockback,
            });
        }
    }
}
