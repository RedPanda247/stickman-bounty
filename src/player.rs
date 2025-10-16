use bevy::{
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

#[derive(Component, Default)]
struct DashComponent(Option<DashData>);

struct DashData {
    direction: Vec2,
    speed: f32,
    duration: f32,
    timer: Timer,
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
            println!(
                "Screen coords: {}/{}",
                mouse_screen_position.x, mouse_screen_position.y
            );
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
}

fn right_click_end_position_system(
    mut right_click_start_position: ResMut<RightClickStartPostion>,
    mut ev_dash: EventWriter<DashEvent>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    player_query: Query<Entity, (With<Player>, With<DashComponent>)>,
) {
    // If the right mouse button was released
    if mouse_input.just_released(MouseButton::Right) {

        // Assume we only have one window
        let window = windows.single().unwrap();

        // If we have a mouse position
        if let Some(mouse_screen_position) = window.cursor_position() {
            println!(
                "Mouse let go: {}/{}",
                mouse_screen_position.x, mouse_screen_position.y
            );
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
                            speed: 1000.,
                            duration: 0.2,
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
    mut dash_entity_query: Query<Entity, With<DashComponent>>,
    mut can_dash_query: Query<&mut DashComponent>,
) {
    for dash_event in event_reader.read() {
        // Get the entity that event describes should dash
        // Gets entity from all entities with DashComponent
        if let Ok(dash_entity) = dash_entity_query.get_mut(dash_event.entity) {

            // Get the DashComponent of that entity
            if let Ok(mut dash_component) = can_dash_query.get_mut(dash_entity) {
                // If the entity is not already dashing, start a new dash
                if dash_component.0.is_none() {
                    dash_component.0 = Some(DashData {
                        direction: dash_event.direction,
                        speed: dash_event.speed,
                        duration: dash_event.duration,
                        timer: Timer::from_seconds(dash_event.duration, TimerMode::Once),
                    });
                }
            }
        }
    }
}

fn start_stop_dash_system(
    mut can_dash_query: Query<(&DashComponent, &mut Velocity), Changed<DashComponent>>,
) {
    for (dash_component, mut velocity) in &mut can_dash_query {
        if let Some(dash_data) = &dash_component.0 {
            velocity.linvel = dash_data.direction * dash_data.speed;
            println!(
                "Started dashing in direction: {}/{} for {} seconds at speed {}",
                dash_data.direction.x, dash_data.direction.y, dash_data.duration, dash_data.speed
            );
        } else {
            velocity.linvel = Vec2::ZERO;
            println!("Stopped dashing");
        }
    }
}

fn dashing_system(
    time: Res<Time>,
    mut can_dash_query: Query<&mut DashComponent, Changed<DashComponent>>,
) {
    for mut dash_component in &mut can_dash_query {
        if let Some(dash_data) = &mut dash_component.0 {
            dash_data.timer.tick(time.delta());
            if dash_data.timer.finished() {
                dash_component.0 = None;
            }
        }
    }
}
