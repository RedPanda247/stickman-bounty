use avian2d::prelude::*;
use bevy::ecs::relationship::RelationshipSourceCollection;
use bevy::prelude::*;

use crate::abilities::*;
use crate::enemy::*;
use crate::game_data::*;
use crate::level::FacingDirection;
use crate::projectiles::*;

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
                grapple_input_system,
                end_grapple_input,
                player_shoot_input,
                player_health_ui,
                look_in_walk_direction,
                reset_jumps_on_ground,
            )
                .run_if(in_state(GameState::PlayingLevel)),
        )
        .add_observer(player_shoot_event)
        // Add this observer to fan out Swinging/PullingEnemy
        .init_resource::<RightClickStartPostion>()
        .init_resource::<MovementModifiers>()
        .register_type::<MovementModifiers>()
        .insert_resource(MovementModifiers::default())
        .init_resource::<GrappleKeybind>()
        .init_resource::<GrapplingHookConfig>();
    }
}

#[derive(Component)]
pub struct PlayerHealthUi;

fn player_health_ui(
    mut ui_qy: Query<&mut Text, With<PlayerHealthUi>>,
    player_qy: Query<&Health, With<Player>>,
) {
    if let Ok(player_health) = player_qy.single() {
        for mut health_ui in ui_qy.iter_mut() {
            health_ui.0 = player_health.0.round().to_string();
        }
    }
}

const PLAYER_PROJECTILE_DAMAGE: f32 = 20.;

#[derive(Event)]
struct PlayerShootEvent;

fn player_shoot_input(mouse_input: Res<ButtonInput<MouseButton>>, mut commands: Commands) {
    if mouse_input.just_released(MouseButton::Left) {
        commands.trigger(PlayerShootEvent);
    }
}

fn player_shoot_event(
    _shoot_event: On<PlayerShootEvent>,
    player_qy: Query<(Entity, &Transform), With<Player>>,
    window_qy: Query<&Window>,
    camera_transform_qy: Query<(&Transform), With<Camera>>,
    mut commands: Commands,
) {
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

        for (entity, transform) in player_qy.iter() {
            let direction = (mouse_world_pos - transform.translation.truncate()).normalize();
            let damage = PLAYER_PROJECTILE_DAMAGE;
            spawn_projectile(
                &mut commands,
                // vec2(transform.translation.x, transform.translation.y + 100.).extend(0.),
                transform.translation,
                direction,
                PROJECTILE_DEFAULT_VELOCITY,
                damage,
                PROJECTILE_DEFAULT_KNOCKBACK,
                vec![entity],
            );
        }
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

fn look_in_walk_direction(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_dir_qy: Query<&mut FacingDirection, With<Player>>,
) {
    let mut input_direction: Option<FacingDirection> = None;
    if keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        input_direction = Some(FacingDirection::Right);
    } else if keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        input_direction = Some(FacingDirection::Left);
    }
    if let Some(dir_ref) = input_direction.as_ref() {
        for mut player_dir in player_dir_qy.iter_mut() {
            *player_dir = match dir_ref {
                FacingDirection::Right => FacingDirection::Right,
                FacingDirection::Left => FacingDirection::Left,
            }
        }
    }
}

fn reset_jumps_on_ground(
    mut player_qy: Query<
        (
            &Transform,
            &LinearVelocity,
            &CollidingEntities,
            &mut JumpsLeft,
        ),
        With<Player>,
    >,
    ground_qy: Query<&Transform, With<Ground>>,
) {
    for (player_transform, velocity, colliding_entities, mut jumps_left) in player_qy.iter_mut() {
        let player_y = player_transform.translation.y;
        let mut is_grounded = false;

        // Check if the player is colliding with ground and is above it (or at same level falling)
        for &ground_entity in colliding_entities.iter() {
            if let Ok(ground_transform) = ground_qy.get(ground_entity) {
                let ground_y = ground_transform.translation.y;
                // Only consider grounded if player is at or above the ground and falling/stationary
                if player_y >= ground_y && velocity.y <= 0.0 {
                    is_grounded = true;
                    break;
                }
            }
        }

        if is_grounded {
            jumps_left.0 = 2;
        }
    }
}

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct JumpsLeft(pub i8);

#[derive(Component)]
pub struct Player;

pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_info: Query<(&mut LinearVelocity, &mut JumpsLeft), With<Player>>,
    time: Res<Time>,
    movement_modifiers: Res<MovementModifiers>,
) {
    for (mut rb_vels, mut jumps_left) in player_info.iter_mut() {
        let max_running_speed =
            movement_modifiers.movement_force * movement_modifiers.max_running_speed;

        if keyboard_input.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) && jumps_left.0 > 0 {
            rb_vels.y = movement_modifiers.movement_force * movement_modifiers.jumping_force;
            jumps_left.0 -= 1;
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
                    }
                }
            }
            // Clear the start position
            right_click_start_position.0 = None;
        }
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

fn end_grapple_input(
    input: Res<ButtonInput<KeyCode>>,
    grapple_keybind: Res<GrappleKeybind>,
    player_qy: Query<Entity, (With<Grappling>, With<Player>)>,
    mut commands: Commands,
) {
    if input.just_released(grapple_keybind.0) {
        for player in player_qy {
            commands.trigger(EndGrapple { entity: player });
        }
    }
}
