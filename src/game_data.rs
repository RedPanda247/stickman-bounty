use bevy::prelude::*;

pub struct GameDataPlugin;

impl Plugin for GameDataPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameState>()
        .init_state::<GameState>();
    }
}

#[derive(Component)]
pub enum GameEntity {
    LevelEntity,
    MainMenuEntity,
    LoadingScreenEntity,
}

#[derive(Default, Clone, Eq, PartialEq, Hash, Debug, Resource, Reflect, States, SystemSet)]
#[reflect(Resource)]
pub enum GameState {
    #[default]
    None,
    PlayingLevel,
    MainMenu,
    Loading,
}