use bevy::prelude::*;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameState>()
        .init_state::<GameState>();
    }
}

#[derive(Component)]
pub enum GameEntity {
    LevelEntity,
    MainMenuEntity,
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