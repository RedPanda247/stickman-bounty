use crate::game_data::*;
use bevy::prelude::*;



pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, load_level.run_if(not(in_state(GameState::Loading))))
            .add_systems(OnEnter(GameState::Loading), load_game_content);
    }
}

enum LoadingScreen {
    Level(LevelIdentifier),
    MainMenu,
}

#[derive(Resource)]
struct LoadingInfo {
    content_being_loaded: PossibleGameStates,
    previous_game_state: PossibleGameStates,
    overite_loading_screen: Option<LoadingScreen>,
    assets_loading: Vec<UntypedHandle>,
}

enum PossibleGameStates {
    Level(LevelIdentifier),
    MainMenu,
}

enum LevelIdentifier{
    Id(u8),
}

#[derive(Component)]
struct LoadingScreenEntity;

#[derive(Event)]
pub struct LoadLevel {
    level: u8,
}

fn load_game_content() {
    
}

pub fn load_level(mut ev_load_level: EventReader<LoadLevel>, state: ResMut<NextState<GameState>>) {
    for load_level in ev_load_level.read() {
        
    }
}

fn spawn_loading_screen_entities(mut command: Commands) {}

fn despawn_game_entities(mut command: Commands, qy_game_entities: Query<Entity, With<GameEntity>>) {
    for game_entity in qy_game_entities {
        command.entity(game_entity).despawn();
    }
}
