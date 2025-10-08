use bevy::{input::mouse::{self, MouseButtonInput}, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::game_data::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (player_movement, camera_movement).run_if(in_state(GameState::PlayingLevel)))
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

#[derive(Resource)]
struct RightClickStartPostion(Option<Vec2>);

fn dash_input_system(mut right_click_start_position: Res<RightClickStartPostion> ,players: Query<&Transform, With<Player>>, keyboard_input: Res<ButtonInput<KeyCode>>, mouse_input: Res<ButtonInput<MouseButton>>) {
    if mouse_input.just_pressed(MouseButton::Right) {
    }
    for player_transform in &players {
        if keyboard_input.any_just_pressed([KeyCode::ArrowRight]) {
            println!("Dash!");
        }
    }
    
}
