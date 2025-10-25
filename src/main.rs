use bevy::{prelude::*};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod level;
use level::*;
mod player;
use player::*;
mod main_menu;
use main_menu::*;
mod loading;
use loading::*;
mod game_data;
use game_data::*;
mod enemy;

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
        .add_plugins((LevelPlugin, PlayerPlugin, MainMenuPlugin, LoadingPlugin, GameDataPlugin))
        .add_systems(Startup, startup)
        .add_systems(PostUpdate, update)
        .run();
}
fn startup(mut commands: Commands, mut ev_load_game_state: EventWriter<LoadGameState>) {
    commands.spawn(Camera2d);
    // main_menu::load_main_menu_entities(commands);
    // ev_load_level_entities.write(LoadLevelEntities { level: 1 });
    ev_load_game_state.write(LoadGameState {
        game_state_to_load: LoadableGameStates::MainMenu,
        loading_screen: LoadingScreen::Basic,
    });

}

fn update() {
    // sleep(Duration::from_secs(1));
}
