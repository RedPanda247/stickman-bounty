use crate::game_data::*;
use crate::main_menu::*;
use bevy::{prelude::*, asset::LoadState};

pub struct LoadingPlugin;

const MIN_LOADING_SCREEN_TIME_SECS: f32 = 4.0;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        // Systems to handle loading
        app.add_systems(
            Update,
            ev_load_game_content.run_if(not(in_state(GameState::Loading))),
        )
        .add_systems(
            OnEnter(GameState::Loading),
            (
                despawn_game_entities,
                spawn_loading_screen_entities,
                spawn_content_by_state_being_loaded,
            )
                .chain(),
        )
        .add_systems(
            Update,
            check_if_loading_complete.run_if(in_state(GameState::Loading)),
        );

        // Resources and Events
        app.insert_resource(GameStateBeingLoaded(LoadableGameStates::MainMenu))
            .insert_resource(AssetsBeingLoaded(Vec::new()))
            .insert_resource(LoadingScreen::FromGameState)
            .insert_resource(LoadingScreenStartTime(0.))
            .add_event::<LoadGameState>();
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
    pub game_state_to_load: LoadableGameStates,
    pub loading_screen: LoadingScreen,
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
        game_state.set(GameState::Loading);
        res_game_state_being_loaded.0 = event.game_state_to_load.clone();
        *res_loading_screen = event.loading_screen.clone();
        res_loading_screen_start_time.0 = time.elapsed_secs();
    }
}

#[derive(Resource, Clone, Copy)]
pub enum LoadingScreen {
    FromGameState,
    StartGame,
    Basic,
}

#[derive(Clone)]
pub enum LoadableGameStates {
    Level(LevelIdentifier),
    MainMenu,
}

impl Into<GameState> for LoadableGameStates {
    fn into(self) -> GameState {
        match self {
            LoadableGameStates::Level(_) => GameState::PlayingLevel,
            LoadableGameStates::MainMenu => GameState::MainMenu,
        }
    }
}


fn check_if_loading_complete(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut res_assets_being_loaded: ResMut<AssetsBeingLoaded>,
    qy_loading_screen_entities: Query<(Entity, &GameEntity)>,
    time: Res<Time>,
    res_loading_screen_start_time: Res<LoadingScreenStartTime>,
    res_game_state_being_loaded: Res<GameStateBeingLoaded>,
    asset_server: Res<AssetServer>,
) {
    // Get the amount of time that the loading screen has been open for
    let loading_screen_time_elapsed = time.elapsed_secs() - res_loading_screen_start_time.0;

    // Remove loaded assets from vec
    res_assets_being_loaded.0.retain(|handle| !matches!(asset_server.get_load_state(handle), Some(LoadState::Loaded)));

    // Check if all assets are loaded and if the minimum time has passed
    if res_assets_being_loaded.0.is_empty()
        && loading_screen_time_elapsed >= MIN_LOADING_SCREEN_TIME_SECS
    {
        // Delete all Loading screen entities
        for (entity, game_entity) in qy_loading_screen_entities.iter() {
            if let GameEntity::LoadingScreenEntity = game_entity {
                commands.entity(entity).despawn();
            }
        }

        // Change Game State
        game_state.set(res_game_state_being_loaded.0.clone().into());
    }
}

fn spawn_loading_screen_entities(
    mut commands: Commands,
    loading_screen: Res<LoadingScreen>,
    game_state_being_loaded: Res<GameStateBeingLoaded>,
) {
    match *loading_screen {
        LoadingScreen::FromGameState => {
            match &game_state_being_loaded.0 {
                LoadableGameStates::Level(level_identifier) => {
                    match level_identifier {
                        // TODO: Add more level loading screens as needed
                        LevelIdentifier::Id(_) => {
                            // spawn level loading screen for levels
                        }
                    }
                }
                LoadableGameStates::MainMenu => {
                    // spawn main menu loading screen
                }
            }
        }
        LoadingScreen::StartGame => {
            
        },
        LoadingScreen::Basic => {
            // spawn start game loading screen
            commands.spawn((
                ZIndex(100),
                GameEntity::LoadingScreenEntity,
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                children![(Text::new("Loading..."),)],
            ));
            commands.spawn((
                GameEntity::LoadingScreenEntity,
                // Go to front to block visibility for content being loaded
                Transform::from_xyz(0., 0., 1.),
                Sprite {
                    color: Color::srgb(0.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(20000., 20000.)),
                    ..Default::default()
                },
            ));
        }
    }
}

fn spawn_content_by_state_being_loaded(
    mut commands: Commands,
    game_state_being_loaded: Res<GameStateBeingLoaded>,
) {
    match &game_state_being_loaded.0 {
        LoadableGameStates::Level(level_identifier) => match level_identifier {
            LevelIdentifier::Id(id) => {
                crate::level::load_level_entities(&mut commands, LevelIdentifier::Id(*id));
            }
        },
        LoadableGameStates::MainMenu => {
            load_main_menu_entities(&mut commands);
        }
    }
}

fn despawn_game_entities(mut command: Commands, qy_game_entities: Query<Entity, With<GameEntity>>) {
    for game_entity in qy_game_entities {
        command.entity(game_entity).despawn();
    }
}
