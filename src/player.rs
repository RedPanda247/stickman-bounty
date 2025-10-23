use bevy::{
    ecs::system::command,
    input::mouse::{self, MouseButtonInput},
    prelude::*,
};
use bevy_rapier2d::prelude::*;

use crate::{game_data::*, player};

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
                recieve_dash_event,
                dashing_system,
            )
                .run_if(in_state(GameState::PlayingLevel)),
        )
        .add_event::<DashEvent>()
        .init_resource::<RightClickStartPostion>()
        .init_resource::<MovementModifiers>()
        .register_type::<MovementModifiers>()
        .insert_resource(MovementModifiers::default());
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
    mut player_info: Query<&mut Velocity, With<Player>>,
    time: Res<Time>,
    movement_modifiers: Res<MovementModifiers>,
) {
    for mut rb_vels in &mut player_info {
        let max_running_speed =
            movement_modifiers.movement_force * movement_modifiers.max_running_speed;

        if keyboard_input.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
            rb_vels.linvel.y = movement_modifiers.movement_force * movement_modifiers.jumping_force;
        }

        let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
        let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

        let x_axis_movement = (-(left as i8) + right as i8) as f32;
        let horizontal_velocity_delta_from_movement =
            x_axis_movement * movement_modifiers.movement_force * time.delta_secs();

        let horizontal_velocity = rb_vels.linvel.x;

        if (horizontal_velocity + horizontal_velocity_delta_from_movement).abs()
            <= max_running_speed
        {
            rb_vels.linvel.x += horizontal_velocity_delta_from_movement;
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
            let fraction_of_delta_translation_to_move_per_second = 1.;
            let translation_delta = player_transform.translation - camera_transform.translation;
            camera_transform.translation += translation_delta
                * fraction_of_delta_translation_to_move_per_second
                * time_res.delta_secs();
        }
    }
}

enum EntityAction {
    DASH,
}

#[derive(Component)]
pub struct CanDash;

#[derive(Component)]
struct StartDash {
    direction: Vec2,
    speed: f32,
    duration: f32,
}

#[derive(Component)]
pub struct Dashing {
    direction: Vec2,
    speed: f32,
    duration: f32,
    start_time: f32,
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

#[derive(Event)]
struct DashEvent {
    entity: Entity,
    direction: Vec2,
    speed: f32,
    duration: f32,
    start_time: f32,
}

fn right_click_end_position_system(
    mut right_click_start_position: ResMut<RightClickStartPostion>,
    mut ev_dash: EventWriter<DashEvent>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    player_query: Query<Entity, (With<Player>, With<CanDash>)>,
    time: Res<Time>,
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
                        ev_dash.write(DashEvent {
                            entity: player_entity,
                            direction: direction,
                            duration: 0.2,
                            speed: 2000.,
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

fn recieve_dash_event(
    mut event_reader: EventReader<DashEvent>,
    mut dash_entity_query: Query<( &mut Velocity, Option<&mut GravityScale> ), With<CanDash>>,
    mut commands: Commands,
) {
    for dash_event in event_reader.read() {
        if let Ok((mut velocity, gravity_opt)) = dash_entity_query.get_mut(dash_event.entity) {
            // apply dash
            velocity.linvel = dash_event.direction * dash_event.speed;
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
            });
        } else {
            warn!("Can't find entity {:?} with required components (Velocity, CanDash).", dash_event.entity);
        }
    }
}


fn dashing_system(time: Res<Time>, mut can_dash_query: Query<(Entity, &mut Dashing, &mut GravityScale, &mut Velocity)>, mut commands: Commands) {
    for (entity, mut dash_component, mut gravityscale, mut velocity) in can_dash_query {
        // End dash if it has been on for the time it should
        if time.elapsed_secs() - dash_component.start_time > dash_component.duration {
            velocity.linvel = Vec2::ZERO;
            *gravityscale = GravityScale(1.);
            commands.entity(entity).remove::<Dashing>();
        }
    }
}

#[derive(Event)]
struct StartJumpEvent {
    entity: Entity,
    force: f32,
}

#[derive(Component)]
struct PlayerControled;

#[derive(Component)]
struct CanJump;

#[derive(Component)]
struct StartJump {
    force: f32,
}
fn jump_input_system_1(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, (With<CanJump>, With<PlayerControled>)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut velocity in query {
            velocity.linvel.y = 10.;
        }
    }
}

fn jump_input_system_2(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, (With<CanJump>, With<PlayerControled>)>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in query {
            commands.entity(entity).insert(StartJump { force: 10. });
        }
    }
}

fn start_entity_jump_2(
    mut query: Query<(Entity, &mut Velocity, &StartJump), (With<CanJump>, Added<StartJump>)>,
    mut commands: Commands,
) {
    for (entity, mut velocity, start_jump_data) in &mut query {
        velocity.linvel.y = start_jump_data.force;
        commands.entity(entity).remove::<StartJump>();
    }
}

fn jump_input_system_3(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, (With<CanJump>, With<PlayerControled>)>,
    mut event_writer: EventWriter<StartJumpEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in query.iter() {
            event_writer.write(StartJumpEvent {
                entity,
                force: 10.0,
            });
        }
    }
}

fn handle_jump_event_system_3(
    mut events: EventReader<StartJumpEvent>,
    mut velocities: Query<&mut Velocity, With<CanJump>>,
) {
    for StartJumpEvent { entity, force } in events.read() {
        if let Ok(mut velocity) = velocities.get_mut(*entity) {
            velocity.linvel.y = *force;
        }
    }
}
