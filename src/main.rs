#![windows_subsystem = "windows"]

use avian2d::prelude::*;
use bevy::{prelude::*};
use bevy_egui::EguiPlugin;
use bevy_framepace::{FramepaceSettings, Limiter};
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
mod projectiles;

mod abilities;
use abilities::AbilitiesPlugin;

use crate::{enemy::EnemyPlugin, projectiles::ProjectilesPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::Immediate,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            bevy_framepace::FramepacePlugin,
            EguiPlugin::default(),
            // WorldInspectorPlugin::default(),
            PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all()),
        ))
        // .insert_resource(FramepaceSettings{ limiter: Limiter::from_framerate(144.)})
        .insert_resource(Gravity(Vec2::NEG_Y * 3000.0))
        // Project plugins
        .add_plugins((
            LevelPlugin,
            PlayerPlugin,
            MainMenuPlugin,
            LoadingPlugin,
            GameDataPlugin,
            AbilitiesPlugin,
            EnemyPlugin,
            ProjectilesPlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}
fn startup(mut commands: Commands, mut ev_load_game_state: MessageWriter<LoadGameState>) {
    commands.spawn(Camera2d);
    ev_load_game_state.write(LoadGameState {
        // game_state_to_load: LoadableGameStates::Level(LevelIdentifier::Id(1)),
        game_state_to_load: LoadableGameStates::MainMenu,
        loading_screen: LoadingScreen::Basic,
    });
}

fn update(time: Res<Time>) {
    // println!("{}", (1. / time.delta_secs()).to_string());
}
