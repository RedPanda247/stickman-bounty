use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement)
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
