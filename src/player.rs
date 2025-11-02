use avian2d::prelude::*;
use bevy::prelude::*;

use crate::enemy::*;
use crate::game_data::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_movement,
                camera_movement,
                right_click_start_position_system,
                right_click_end_position_system,
                dashing_system,
                grapple_input_system,
            )
                .run_if(in_state(GameState::PlayingLevel)),
        )
        .add_systems(FixedLast, dash_collision_system)
        .add_systems(FixedLast, fixed_attach_grappling_hook)
        .add_observer(end_dash)
        .add_observer(recieve_dash_event)
        .add_observer(grapple_event_observer)
        // Add this observer to fan out Swinging/PullingEnemy
        .add_observer(hook_attachment_observer)
        .init_resource::<RightClickStartPostion>()
        .init_resource::<MovementModifiers>()
        .register_type::<MovementModifiers>()
        .insert_resource(MovementModifiers::default())
        .init_resource::<GrappleKeybind>();
    }
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct MovementModifiers {
    movement_force: f32,
    max_running_speed: f32,
    jumping_force: f32,
}
impl Default for MovementModifiers {
    fn default() -> Self {
        MovementModifiers {
            movement_force: 4000.,
            max_running_speed: 0.2,
            jumping_force: 0.2,
        }
    }
}

#[derive(Component)]
pub struct Player;

pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_info: Query<&mut LinearVelocity, With<Player>>,
    time: Res<Time>,
    movement_modifiers: Res<MovementModifiers>,
) {
    for mut rb_vels in &mut player_info {
        let max_running_speed =
            movement_modifiers.movement_force * movement_modifiers.max_running_speed;

        if keyboard_input.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
            rb_vels.y = movement_modifiers.movement_force * movement_modifiers.jumping_force;
        }

        let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
        let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

        let x_axis_movement = (-(left as i8) + right as i8) as f32;
        let horizontal_velocity_delta_from_movement =
            x_axis_movement * movement_modifiers.movement_force * time.delta_secs();

        let horizontal_velocity = rb_vels.x;

        if (horizontal_velocity + horizontal_velocity_delta_from_movement).abs()
            <= max_running_speed
        {
            rb_vels.x += horizontal_velocity_delta_from_movement;
        }
    }
}

fn camera_movement(
    mut qy_camera_transform: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player_transform_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    time_res: Res<Time>,
) {
    for mut camera_transform in &mut qy_camera_transform {
        for player_transform in player_transform_query {
            let camera_follow_strength = 10.;
            let translation_delta = player_transform.translation - camera_transform.translation;
            camera_transform.translation +=
                translation_delta * camera_follow_strength * time_res.delta_secs();
        }
    }
}

#[derive(Component)]
pub struct CanDash;

#[derive(Component)]
pub struct Dashing {
    direction: Vec2,
    speed: f32,
    duration: f32,
    start_time: f32,
    started_moving: bool,
}

#[derive(Resource, Default)]
struct RightClickStartPostion(Option<Vec2>);

fn right_click_start_position_system(
    mut right_click_start_position: ResMut<RightClickStartPostion>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        let window = windows.single().unwrap();

        if let Some(mouse_screen_position) = window.cursor_position() {
            right_click_start_position.0 = Some(mouse_screen_position);
        }
    }
}

#[derive(EntityEvent)]
struct DashEvent {
    entity: Entity,
    direction: Vec2,
    speed: f32,
    duration: f32,
    start_time: f32,
}

fn right_click_end_position_system(
    mut right_click_start_position: ResMut<RightClickStartPostion>,
    // mut ev_dash: EventWriter<DashEvent>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    player_query: Query<Entity, (With<Player>, With<CanDash>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    // If the right mouse button was released
    if mouse_input.just_released(MouseButton::Right) {
        // Assume we only have one window
        let window = windows.single().unwrap();

        // If we have a mouse position
        if let Some(mouse_screen_position) = window.cursor_position() {
            // If we have a start position and it's different from the end position
            if let Some(start_position) = right_click_start_position.0 {
                if start_position != mouse_screen_position {
                    // Calculate the direction from start to end position
                    let mut direction = (mouse_screen_position - start_position).normalize();

                    direction.y = -direction.y; // Invert Y axis because window Y cords go downwards

                    // Send a dash for for all players
                    for player_entity in player_query {
                        commands.trigger(DashEvent {
                            entity: player_entity,
                            direction: direction,
                            duration: 0.15,
                            speed: 4000.,
                            start_time: time.elapsed_secs(),
                        });
                        // ev_dash.write(DashEvent {
                        //     entity: player_entity,
                        //     direction: direction,
                        //     duration: 0.15,
                        //     speed: 4000.,
                        //     start_time: time.elapsed_secs(),
                        // });
                    }
                }
            }
            // Clear the start position
            right_click_start_position.0 = None;
        }
    }
}

fn recieve_dash_event(
    dash_event: On<DashEvent>,
    mut dash_entity_query: Query<(&mut LinearVelocity, Option<&mut GravityScale>), With<CanDash>>,
    mut commands: Commands,
) {
    if let Ok((mut velocity, gravity_opt)) = dash_entity_query.get_mut(dash_event.entity) {
        // apply dash
        velocity.0 = dash_event.direction * dash_event.speed;
        if let Some(mut gravityscale) = gravity_opt {
            *gravityscale = GravityScale(0.0);
        } else {
            // add a GravityScale component if missing
            commands.entity(dash_event.entity).insert(GravityScale(0.0));
        }

        commands.entity(dash_event.entity).insert(Dashing {
            direction: dash_event.direction,
            speed: dash_event.speed,
            duration: dash_event.duration,
            start_time: dash_event.start_time,
            started_moving: false,
        });
    } else {
        warn!(
            "Can't find entity {:?} with required components (Velocity, CanDash).",
            dash_event.entity
        );
    }
}
#[derive(EntityEvent)]
struct EndDash {
    entity: Entity,
}
fn end_dash(
    end_dash_event: On<EndDash>,
    mut query: Query<(Entity, &mut GravityScale, &mut LinearVelocity), With<Dashing>>,
    mut commands: Commands,
) {
    if let Ok((entity, mut gravity_scale, mut velocity)) = query.get_mut(end_dash_event.entity) {
        velocity.0 = Vec2::ZERO;
        *gravity_scale = GravityScale(1.);
        commands.entity(entity).remove::<Dashing>();
    }
}

fn dashing_system(time: Res<Time>, query: Query<(Entity, &mut Dashing)>, mut commands: Commands) {
    for (entity, dash_component) in query {
        // End dash if it has been on for the time it should
        if time.elapsed_secs() - dash_component.start_time > dash_component.duration {
            commands.trigger(EndDash { entity });
        }
    }
}
fn dash_collision_system(
    qy: Query<(Entity, &CollidingEntities, &mut Dashing)>,
    mut commands: Commands,
) {
    for (entity, colliding_entities, mut dashing) in qy {
        // skip first because, add component add speed, next frame, move by speed, colliding entities still not updated ->
        // check if we collided if not wait we think we collide with something here even though it is just the floor that
        // we were colliding with since before we started moving from the dash
        if !dashing.started_moving {
            dashing.started_moving = true;
            continue;
        }
        if !colliding_entities.is_empty() {
            commands.trigger(EndDash { entity });
        }
    }
}

#[derive(EntityEvent)]
struct StartGrapple {
    entity: Entity,
    grapple_world_target: Vec2,
}

#[derive(Component)]
pub struct CanGrapple;

#[derive(Component)]
struct Grappling; // marker: "I'm in a grapple flow"

#[derive(Component)]
struct Swinging {
    anchor: Vec2,     // world position of the anchor point
    rope_length: f32, // initial rope length
}

#[derive(Component)]
struct PullingEnemy {
    enemy: Entity,    // enemy being pulled
    rope_length: f32, // initial rope length
}

#[derive(Resource)]
struct GrappleKeybind(KeyCode);
impl Default for GrappleKeybind {
    fn default() -> Self {
        GrappleKeybind(KeyCode::Space)
    }
}
fn grapple_input_system(
    input: Res<ButtonInput<KeyCode>>,
    grapple_keybind: Res<GrappleKeybind>,
    mut commands: Commands,
    player_qy: Query<Entity, (With<CanGrapple>, With<Player>)>,
    window_qy: Query<&Window>,
    camera_transform_qy: Query<&Transform, With<Camera>>,
) {
    if input.just_pressed(grapple_keybind.0) {
        // get window
        let window = window_qy
            .single()
            .expect("Multiple Windows present, not compatible with current grapple implementation");
        // try to get raw mouse window position
        if let Some(mouse_window_pos) = window.cursor_position() {
            // invert y
            let mouse_window_pos = vec2(mouse_window_pos.x, window.height() - mouse_window_pos.y);
            // get camera transform
            let camera_transform = camera_transform_qy
                .single()
                .expect("Found multiple cameras, incompatible with current grapple implementation");

            // Convert camera position to Vec2 using truncate()
            let camera_pos = camera_transform.translation.truncate();

            // Calculate mouse world position (accounting for centered origin)
            let window_size = Vec2::new(window.width(), window.height());
            let mouse_world_pos = mouse_window_pos - window_size / 2.0 + camera_pos;

            for entity in player_qy.iter() {
                commands.trigger(StartGrapple {
                    entity,
                    grapple_world_target: mouse_world_pos,
                });
            }
        }
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
const GRAPPLING_HOOK_SIZE: f32 = 20.;
#[derive(Component)]
struct GrapplingHook {
    shooter_entity: Entity,
    attached_to: Option<GrapplingHookAttachmentType>,
}
#[derive(Clone, Copy)]
enum GrapplingHookAttachmentType {
    Enemy(Entity),
    World,
}
#[derive(EntityEvent)]
struct GrappleAttachedEvent {
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
    enemy_qy: Query<(), With<Enemy>>,
    mut commands: Commands,
) {
    for (mut hook, colliding, _hook_tf) in hook_qy.iter_mut() {
        if hook.attached_to.is_some() {
            continue;
        }

        // Ignore the shooter itself
        let mut hit_enemy: Option<Entity> = None;

        for other in colliding
            .iter()
            .copied()
            .filter(|e| *e != hook.shooter_entity)
        {
            if enemy_qy.contains(other) {
                hit_enemy = Some(other);
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
    transforms: Query<&Transform>,
    // to find the hook entity for this shooter and get its world pos if needed
    hook_q: Query<(&GrapplingHook, &Transform)>,
) {
    let shooter = grapple_attached_event.entity;

    match grapple_attached_event.attachment_type {
        GrapplingHookAttachmentType::World => {
            // Find the hook belonging to this shooter to get the anchor position
            if let (Ok(shooter_tf), Some((_hook, hook_tf))) = (
                transforms.get(shooter),
                hook_q.iter().find(|(h, _)| h.shooter_entity == shooter),
            ) {
                let anchor = hook_tf.translation.truncate();
                let rope_length = shooter_tf.translation.truncate().distance(anchor);

                commands.entity(shooter).insert(Swinging {
                    anchor,
                    rope_length,
                });
            }
        }
        GrapplingHookAttachmentType::Enemy(enemy) => {
            if let (Ok(shooter_tf), Ok(enemy_tf)) = (transforms.get(shooter), transforms.get(enemy))
            {
                let rope_length = shooter_tf
                    .translation
                    .truncate()
                    .distance(enemy_tf.translation.truncate());

                commands
                    .entity(shooter)
                    .insert(PullingEnemy { enemy, rope_length });
            }
        }
    }
}
fn grapple(qy: Query<(&Transform, &Grappling)>) {}
#[derive(EntityEvent)]
struct EndGrapple {
    entity: Entity,
}
fn end_grapple_input(
    input: Res<ButtonInput<KeyCode>>,
    grapple_keybind: Res<GrappleKeybind>,
    player_qy: Query<Entity, (With<Dashing>, With<Player>)>,
    mut commands: Commands,
) {
    if input.just_released(grapple_keybind.0) {
        for player in player_qy {
            commands.trigger(EndGrapple { entity: player });
        }
    }
}
