use crate::game_data::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use std::collections::HashSet;

pub struct ProjectilesPlugin;

impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedLast, spawn_collisions_continious)
            .add_observer(spawn_collisions)
            .add_observer(projectile_collision)
            .add_observer(projectile_hit_event);
    }
}

#[derive(Component)]
struct Projectile {
    damage: f32,
    knockback: f32,
}

#[derive(Component)]
struct ProjectileMarkedForDespawn;

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
    let projectile_size = 5.;
    commands.spawn((
        // Constant projectile components
        GameEntity::LevelEntity,
        RigidBody::Kinematic,
        GravityScale(0.),
        CollidingEntities::default(),
        CollisionEventsEnabled,
        Collider::rectangle(projectile_size, projectile_size),
        Sensor,
        // Dynamically decided components
        Projectile { damage, knockback },
        ProjectileDisgardInitialSpawnCollisionWith(disgard_initial_collision_with),
        Transform::from_translation(position),
        LinearVelocity(direction * velocity),
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
                .queue_silenced(|mut entity: EntityWorldMut| {
                    entity.remove::<ProjectileDisgardInitialSpawnCollisionWith>();
                });
        }
        handled = true;
    }

    // Also handle the symmetric case: remove c1 from c2's discard list
    if let Ok(mut pd) = projectile_qy.get_mut(collision_entity_2) {
        pd.0.retain(|&e| e != collision_entity_1);
        if pd.0.is_empty() {
            commands
                .entity(collision_entity_2)
                .queue_silenced(|mut entity: EntityWorldMut| {
                    entity.remove::<ProjectileDisgardInitialSpawnCollisionWith>();
                });
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
                .queue_silenced(|mut entity: EntityWorldMut| {
                    entity.remove::<ProjectileDisgardInitialSpawnCollisionWith>();
                });
            println!("removed disgard list");
        }
    }
}

#[derive(Event)]
struct ProjectileHitEvent {
    hit_entity: Entity,
    projectile_entity: Entity,
    damage: f32,
    knockback_impulse: Vec2,
}

fn projectile_collision(
    collision_event: On<CollisionStart>,
    projectile_qy: Query<(&Projectile, &LinearVelocity), Without<ProjectileMarkedForDespawn>>,
    hit_entity_qy: Query<(&CanBeHitByProjectile)>,
    disgard_initial_collision_qy: Query<&ProjectileDisgardInitialSpawnCollisionWith>,
    mut commands: Commands,
) {
    
    // Get both collider entities from the event
    let projectile_entity = collision_event.collider1;
    let hit_entity = collision_event.collider2;

    if let Ok((projectile, linvel)) = projectile_qy.get(projectile_entity) {
        // If the second entity is in CanBeHitByProjectile
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
            // Mark projectile for despawn to prevent multiple hits
            commands.entity(projectile_entity).queue_silenced(|mut entity: EntityWorldMut| {
                entity.insert(ProjectileMarkedForDespawn);
            });
            
            commands.trigger(ProjectileHitEvent {
                hit_entity,
                projectile_entity,
                damage: projectile.damage,
                knockback_impulse: linvel.0.normalize() * projectile.knockback,
            });
        }
    }
}

fn projectile_hit_event(
    projectile_hit_event: On<ProjectileHitEvent>,
    mut commands: Commands,
    mut hit_entity_qy: Query<(Forces), With<CanBeHitByProjectile>>,
    mut health_qy: Query<&mut Health>,
) {
    let hit_entity = projectile_hit_event.hit_entity;
    let projectile_entity = projectile_hit_event.projectile_entity;
    // Apply knockback to hit entity
    if let Ok(mut hit_entity_force) = hit_entity_qy.get_mut(hit_entity) {
        commands.entity(hit_entity).remove::<Sleeping>();
        hit_entity_force.apply_linear_impulse(projectile_hit_event.knockback_impulse);
    }
    // Deal damage to hit entity if it has a Health component
    if let Ok(mut health) = health_qy.get_mut(hit_entity) {
        health.0 -= projectile_hit_event.damage;
        if health.0 < 0. {
            health.0 = 0.;
        }
    }
    // Use queue_silenced to prevent error if already despawned
    commands
        .entity(projectile_entity)
        .queue_silenced(|mut entity: EntityWorldMut| {
            entity.despawn();
        });
}
