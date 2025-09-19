use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

mod level;
use level::*;
mod player;
use player::*;
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            bevy_framepace::FramepacePlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0),
            RapierDebugRenderPlugin::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ))
        // Project plugins
        .add_plugins((LevelPlugin, PlayerPlugin))
        .add_systems(Startup, startup)
        .run();
}
fn startup(mut commands: Commands, mut ev_load_leve_entities: EventWriter<LoadLevelEntities>) {
    commands.spawn(Camera2d);
    ev_load_leve_entities.write(LoadLevelEntities { level: 1 });
}
