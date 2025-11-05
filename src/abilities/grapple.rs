use avian2d::prelude::*;
use bevy::prelude::*;
use crate::enemy::Enemy;
use crate::game_data::*;

// Tuning defaults
const DEFAULT_GRAPPLING_HOOK_SPRING_FORCE: f32 = 100_000.0;
const DEFAULT_GRAPPLING_HOOK_DAMPENING: f32 = 1_000_000.0;
const GRAPPLING_HOOK_SIZE: f32 = 20.0;

pub struct GrapplePlugin;
impl Plugin for GrapplePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GrappleKeybind>()
            .init_resource::<GrapplingHookConfig>()
            .add_observer(grapple_event_observer)
            .add_observer(hook_attachment_observer)
            
            .add_systems(
                FixedUpdate,
                (
                    fixed_attach_grappling_hook,
                    grappling_hook_swinging_spring_force_system_fixed,
                    damp_hook_spring_oscillation,
                    hook_follow_enemy,
                )
                    .run_if(in_state(GameState::PlayingLevel)),
            );
    }
}

#[derive(EntityEvent)]
pub struct StartGrapple {
    pub entity: Entity,
    pub grapple_world_target: Vec2,
}

#[derive(Component)]
pub struct CanGrapple;

#[derive(Component)]
pub struct Grappling; // marker: "I'm in a grapple flow"

#[derive(Component)]
pub struct Swinging {
    pub hook_entity: Entity,
    pub anchor: Vec2, // world position of the anchor point
    pub rope_rest_length: f32,
    pub previous_distance_from_hook: Option<f32>,
}

#[derive(Component)]
pub struct PullingEnemy {
    pub hook_entity: Entity,
    pub enemy: Entity,    // enemy being pulled
    pub rope_length: f32, // initial rope length
}

#[derive(Resource)]
pub struct GrappleKeybind(pub KeyCode);
impl Default for GrappleKeybind {
    fn default() -> Self {
        GrappleKeybind(KeyCode::Space)
    }
}

fn grapple_event_observer(grapple_start_event: On<StartGrapple>, mut commands: Commands) {
    commands
        .entity(grapple_start_event.entity)
        .insert(Grappling); // marker only

    spawn_grapple(
        &mut commands,
        grapple_start_event.entity,
        grapple_start_event.grapple_world_target.extend(0.),
    );
}
#[derive(Component)]
pub struct GrapplingHook {
    shooter_entity: Entity,
    attached_to: Option<GrapplingHookAttachmentType>,
}
#[derive(Clone, Copy)]
enum GrapplingHookAttachmentType {
    Enemy(Entity),
    World,
}
#[derive(EntityEvent)]
pub struct GrappleAttachedEvent {
    entity: Entity, // shooter entity
    attachment_type: GrapplingHookAttachmentType,
}

fn spawn_grapple(commands: &mut Commands, shooter_entity: Entity, world_position: Vec3) {
    commands.spawn((
        GameEntity::LevelEntity,
        GrapplingHook {
            shooter_entity,
            attached_to: None,
        },
        Transform::from_translation(world_position),
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::splat(GRAPPLING_HOOK_SIZE)),
            ..default()
        },
        Collider::rectangle(GRAPPLING_HOOK_SIZE, GRAPPLING_HOOK_SIZE),
        CollidingEntities::default(),
    ));
}

fn fixed_attach_grappling_hook(
    mut hook_qy: Query<(&mut GrapplingHook, &CollidingEntities, &Transform)>,
    enemy_qy: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) {
    for (mut hook, colliding_entities, _hook_tf) in hook_qy.iter_mut() {
        if hook.attached_to.is_some() {
            continue;
        }

        let mut hit_enemy: Option<Entity> = None;

        for collided_with in colliding_entities
            .iter()
            .copied()
            .filter(|e| *e != hook.shooter_entity)
        {
            if enemy_qy.contains(collided_with) {
                hit_enemy = Some(collided_with);
                break;
            }
        }

        if let Some(enemy_entity) = hit_enemy {
            hook.attached_to = Some(GrapplingHookAttachmentType::Enemy(enemy_entity));
            commands.trigger(GrappleAttachedEvent {
                entity: hook.shooter_entity,
                attachment_type: GrapplingHookAttachmentType::Enemy(enemy_entity),
            });
        } else {
            hook.attached_to = Some(GrapplingHookAttachmentType::World);
            commands.trigger(GrappleAttachedEvent {
                entity: hook.shooter_entity,
                attachment_type: GrapplingHookAttachmentType::World,
            });
        }
    }
}
fn hook_attachment_observer(
    grapple_attached_event: On<GrappleAttachedEvent>,
    mut commands: Commands,
    // for distances
    transforms: Query<&Transform, With<RigidBody>>,
    // to find the hook entity for this shooter and get its world pos if needed
    hook_q: Query<(Entity, &GrapplingHook, &Transform)>,
) {
    let shooter = grapple_attached_event.entity;

    match grapple_attached_event.attachment_type {
        GrapplingHookAttachmentType::World => {
            // Find the hook belonging to this shooter to get the anchor position
            if let (Ok(shooter_tf), Some((hook_entity, _hook, hook_tf))) = (
                transforms.get(shooter),
                hook_q.iter().find(|(_, h, _)| h.shooter_entity == shooter),
            ) {
                let anchor = hook_tf.translation.truncate();
                let rope_length = shooter_tf.translation.truncate().distance(anchor);

                commands.entity(shooter).insert(Swinging {
                    hook_entity,
                    anchor,
                    rope_rest_length: rope_length,
                    previous_distance_from_hook: None,
                });
            }
        }
        GrapplingHookAttachmentType::Enemy(enemy) => {
            if let (Ok(shooter_tf), Some((entity, _hoook, hook_tf))) = (
                transforms.get(shooter),
                hook_q.iter().find(|(_, h, _)| h.shooter_entity == shooter),
            ) {
                let rope_length = shooter_tf
                    .translation
                    .truncate()
                    .distance(hook_tf.translation.truncate());

                commands.entity(shooter).insert(PullingEnemy {
                    hook_entity: entity,
                    enemy,
                    rope_length,
                });
            }
        }
    }
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct GrapplingHookConfig {
    pub spring_force: f32,
    pub spring_dampening: f32,
}

impl Default for GrapplingHookConfig {
    fn default() -> Self {
        GrapplingHookConfig {
            spring_force: DEFAULT_GRAPPLING_HOOK_SPRING_FORCE,
            spring_dampening: DEFAULT_GRAPPLING_HOOK_DAMPENING,
        }
    }
}

fn grappling_hook_swinging_spring_force_system_fixed(
    qy: Query<(Forces, &Transform, &Swinging)>,
    time: Res<Time>,
    grappling_hook_spring_force: Res<GrapplingHookConfig>,
) {
    for (mut force, transform, swinging) in qy {
        let distance_to_hook = transform.translation.truncate().distance(swinging.anchor);
        let spring_discomfort = distance_to_hook - swinging.rope_rest_length;
        let spring_force_1d = spring_discomfort * grappling_hook_spring_force.spring_force;
        let direction_to_hook = (swinging.anchor - transform.translation.truncate()).normalize();
        let spring_force_on_entity = direction_to_hook * spring_force_1d;
        force.apply_linear_impulse(spring_force_on_entity * time.delta_secs());
    }
}

fn damp_hook_spring_oscillation(
    qy: Query<(Forces, &Transform, &mut Swinging)>,
    time: Res<Time>,
    grappling_hook_config: Res<GrapplingHookConfig>,
) {
    for (mut force, transform, mut swinging) in qy {
        let distance_to_hook = transform.translation.truncate().distance(swinging.anchor);
        if let Some(previous_distance_from_hook) = swinging.previous_distance_from_hook {
            let delta_distance_to_hook = distance_to_hook - previous_distance_from_hook;
            let direction_to_hook = (swinging.anchor - transform.translation.truncate()).normalize();
            let spring_dampening_force_1d = delta_distance_to_hook * grappling_hook_config.spring_dampening;
            let spring_dampening_force_on_entity = spring_dampening_force_1d * direction_to_hook;
            force.apply_linear_impulse(spring_dampening_force_on_entity * time.delta_secs());
        }
        swinging.previous_distance_from_hook = Some(distance_to_hook);
    }
}

fn hook_follow_enemy(
    hook_qy: Query<(&mut Transform, &GrapplingHook), Without<Enemy>>,
    enemy_transform_qy: Query<&Transform, With<Enemy>>,
) {
    for (mut transform, hook) in hook_qy {
        if let Some(attached_to) = hook.attached_to {
            if let GrapplingHookAttachmentType::Enemy(enemy_entity) = attached_to {
                if let Ok(enemy_transform) = enemy_transform_qy.get(enemy_entity) {
                    transform.translation = enemy_transform.translation;
                }
            }
        }
    }
}
#[derive(EntityEvent)]
pub struct EndGrapple {
    pub entity: Entity,
}