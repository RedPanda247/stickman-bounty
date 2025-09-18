use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::level::LoadLevelEntities;

mod level;
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins, 
            bevy_framepace::FramepacePlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0),
            RapierDebugRenderPlugin::default(),
        ))
        // Project plugins
        .add_plugins(level::LevelPlugin)
        .add_systems(Startup, startup)
        .run();
}
fn startup(mut commands: Commands, mut ev_load_leve_entities: EventWriter<level::LoadLevelEntities>) {
    commands.spawn(Camera2d);
    ev_load_leve_entities.write(LoadLevelEntities{level: 1});
}
