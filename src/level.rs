use bevy::prelude::*;
use avian2d::prelude::*;

use crate::enemy::*;
use crate::game_data::*;
use crate::player::*;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LoadLevelEntities>()
            .add_systems(Update, ev_load_level_entities);
    }
}
#[derive(Message)]
pub struct LoadLevelEntities {
    pub level: LevelIdentifier,
}

pub fn ev_load_level_entities(
    mut commands: Commands,
    mut ev_load_level_entities: MessageReader<LoadLevelEntities>,
) {
    for event in ev_load_level_entities.read() {
        load_level_entities(&mut commands, event.level.clone());
    }
}

pub fn load_level_entities(commands: &mut Commands, level: LevelIdentifier) {
    match level {
        LevelIdentifier::Id(id) => {
            if id == 1 {
                let player_size = 50.;
                let ground_height = 100.;
                let ground_width = 10000.;

                commands.spawn((
                    Player,
                    CanDash,
                    GameEntity::LevelEntity,
                    Sprite {
                        color: Color::srgb(0.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new(player_size * 2., player_size * 2.)),
                        ..Default::default()
                    },
                    RigidBody::Dynamic,
                    LinearVelocity::ZERO,
                    LockedAxes::ROTATION_LOCKED,
                    Transform::from_xyz(0., 400., 0.),
                    Collider::rectangle(player_size, player_size),
                ));
                commands.spawn((
                    GameEntity::LevelEntity,
                    Sprite {
                        color: Color::srgb(0.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new(ground_width * 2., ground_height * 2.)),
                        ..Default::default()
                    },
                    RigidBody::Static,
                    Transform::from_xyz(0., -100., 0.),
                    Collider::rectangle(ground_width, ground_height),
                ));
                // Spawn enemy
                spawn_character(
                    commands,
                    CharacterBundle {
                        size: 50.,
                        position: vec3(500., 700., 0.),
                        color: Color::srgb(8.0, 0.0, 0.0),
                    },
                    (Enemy),
                );
            }
        }
    }
}
