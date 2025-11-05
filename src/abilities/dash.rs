use bevy::prelude::*;
use avian2d::prelude::*;
use crate::game_data::GameState;

pub struct DashPlugin;
impl Plugin for DashPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(recieve_dash_event)
            .add_observer(end_dash)
            .add_systems(
                FixedUpdate,
                dashing_system.run_if(in_state(GameState::PlayingLevel)),
            );
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

#[derive(EntityEvent)]
pub struct DashEvent {
    pub entity: Entity,
    pub direction: Vec2,
    pub speed: f32,
    pub duration: f32,
    pub start_time: f32,
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