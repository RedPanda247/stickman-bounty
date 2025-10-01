use crate::game_data::*;
use bevy::{prelude::*};

pub struct LoadingPlugin;

const MIN_LOADING_SCREEN_TIME_SECS: f32 = 4.0;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ev_load_game_content.run_if(not(in_state(GameState::Loading))))
            .add_systems(OnEnter(GameState::Loading), (despawn_game_entities, spawn_loading_screen_entities))
            .add_systems(Update, check_if_loading_complete.run_if(in_state(GameState::Loading)));
    }
}

// Resources to keep track of loading info

#[derive(Resource)]
struct GameStateBeingLoaded(LoadableGameStates);

#[derive(Resource)]
struct AssetsBeingLoaded(Vec<UntypedHandle>);

#[derive(Resource)]
struct LoadingScreenStartTime(f32);

// Events to trigger loading

#[derive(Event)]
pub struct LoadGameState {
    game_state_to_load: LoadableGameStates,
    loading_screen: LoadingScreen,
}

fn ev_load_game_content(
    mut ev_load_game_content: EventReader<LoadGameState>,
    mut game_state: ResMut<NextState<GameState>>,
    mut res_game_state_being_loaded: ResMut<GameStateBeingLoaded>,
    mut res_loading_screen: ResMut<LoadingScreen>,
    mut res_loading_screen_start_time: ResMut<LoadingScreenStartTime>,
    time: Res<Time>,
) {
    for event in ev_load_game_content.read() {
        game_state.set(event.game_state_to_load.clone().into());
        res_game_state_being_loaded.0 = event.game_state_to_load.clone();
        *res_loading_screen = event.loading_screen.clone();
        res_loading_screen_start_time.0 = time.elapsed_secs();
    }
}

#[derive(Resource, Clone, Copy)]
enum LoadingScreen {
    FromGameState,
    StartGame,
}

#[derive(Clone)]
enum LoadableGameStates {
    Level(LevelIdentifier),
    MainMenu,
}

impl Into<GameState> for LoadableGameStates {
    fn into(self) -> GameState {
        match self {
            LoadableGameStates::Level(_) => {
                GameState::PlayingLevel
            },
            LoadableGameStates::MainMenu => {
                GameState::MainMenu
            }
        }
    }
}

#[derive(Clone)]
enum LevelIdentifier {
    Id(u8),
}

#[derive(Component)]
struct LoadingScreenEntity;

fn check_if_loading_complete(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut res_assets_being_loaded: ResMut<AssetsBeingLoaded>,
    qy_loading_screen_entities: Query<Entity, With<LoadingScreenEntity>>,
    time: Res<Time>,
    res_loading_screen_start_time: Res<LoadingScreenStartTime>,
) {
    let loading_screen_time_elapsed = time.elapsed_secs() - res_loading_screen_start_time.0;
    if res_assets_being_loaded.0.is_empty() && loading_screen_time_elapsed >= MIN_LOADING_SCREEN_TIME_SECS {
        for loading_screen_entity in qy_loading_screen_entities.iter() {
            commands.entity(loading_screen_entity).despawn();
        }
        game_state.set(GameState::PlayingLevel);
    }
}

fn spawn_loading_screen_entities(mut command: Commands, loading_screen: Res<LoadingScreen>, game_state_being_loaded: Res<GameStateBeingLoaded>) {
    match *loading_screen {
        LoadingScreen::FromGameState => {
            match &game_state_being_loaded.0 {
                LoadableGameStates::Level(level_identifier) => {
                    match level_identifier {
                        // TODO: Add more level loading screens as needed
                        LevelIdentifier::Id(_) => {
                            // spawn level loading screen for level 1
                        }
                    }
                },
                LoadableGameStates::MainMenu => {
                    // spawn main menu loading screen
                }
            }
        },
        LoadingScreen::StartGame => {
            // spawn start game loading screen
        }
    }
    command.spawn((
        LoadingScreenEntity,
    ));

}

fn despawn_game_entities(mut command: Commands, qy_game_entities: Query<Entity, With<GameEntity>>) {
    for game_entity in qy_game_entities {
        command.entity(game_entity).despawn();
    }
}
