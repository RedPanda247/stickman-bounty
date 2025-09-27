use bevy::{prelude::*};
use crate::game_data::*;

enum ContentBeingLoaded{
    Level,
    MainMenu,
}

#[derive(Component)]
struct LoadingScreenEntity;

#[derive(Event)]
pub struct LoadLevel{
    level: u8,
}

pub fn load_level(){

}

fn spawn_loadingscreen_entities(mut command: Commands) {
    
}

fn despawn_game_entities(mut command: Commands, qy_game_entities: Query<Entity, With<GameEntity>>) {
    for game_entity in qy_game_entities {
        command.entity(game_entity).despawn();
    }
}