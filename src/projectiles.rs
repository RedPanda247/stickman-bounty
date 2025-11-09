use crate::game_data::*;
use bevy::prelude::*;

pub struct ProjectilesPlugin;

impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
struct Projectile;

fn spawn_projectile(commands: &mut Commands, position: Vec3) {
    commands.spawn((
        GameEntity::LevelEntity, 
        Projectile,
        Transform::from_translation(position),
    ));
}
