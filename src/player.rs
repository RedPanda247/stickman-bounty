use avian2d::prelude::*;
use bevy::prelude::*;

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
            )
                .run_if(in_state(GameState::PlayingLevel)),
        )
        // This needs to run in the fixed update in sync with the physics, because otherwise it will run multiple times before the physics have even moved the player which means it will keep detecting the current value of the collidingEntities component multiple times before the physics start moving the entity from the ground for exemple with the dash velocity
        .add_systems(FixedLast, dash_collision_system)
        .add_observer(end_dash)
        .add_observer(recieve_dash_event)
        // .add_observer(dash_collision)
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
        let window = window_qy
            .single()
            .expect("Multiple Windows present, not compatible with current grapple implementation");
        let mouse_window_pos = window
            .cursor_position()
            .expect("Couldn't get mouse position in window in grappling input system");
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
