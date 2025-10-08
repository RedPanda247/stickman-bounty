use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game_data::*;
use crate::player::*;

#[derive(Component)]
pub struct LevelEntity;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadLevelEntities>()
            .add_systems(Update, ev_load_level_entities);
    }
}
#[derive(Event)]
pub struct LoadLevelEntities {
    pub level: LevelIdentifier,
}

pub fn ev_load_level_entities(
    mut commands: Commands,
    mut ev_load_level_entities: EventReader<LoadLevelEntities>,
) {
    for event in ev_load_level_entities.read() {
        load_level_entities(&mut commands, event.level.clone());
    }
}

pub fn load_level_entities(commands: &mut Commands, level: LevelIdentifier) {
    match level {
        LevelIdentifier::Id(id) => {
            if id == 1 {
                let sprite_size = 100.;
                let ground_height = 100.;
                let ground_width = 1000.;

                commands.spawn((
                    Player,
                    LevelEntity,
                    Sprite {
                        color: Color::srgb(0.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new((sprite_size * 2.), (sprite_size * 2.))),
                        ..Default::default()
                    },
                    RigidBody::Dynamic,
                    Velocity::zero(),
                    LockedAxes::ROTATION_LOCKED,
                    Transform::from_xyz(0., 400., 0.),
                    Collider::cuboid(sprite_size, sprite_size),
                ));
                commands.spawn((
                    LevelEntity,
                    Sprite {
                        color: Color::srgb(0.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new((ground_width * 2.), (ground_height * 2.))),
                        ..Default::default()
                    },
                    RigidBody::Fixed,
                    Transform::from_xyz(0., -100., 0.),
                    Collider::cuboid(ground_width, ground_height),
                ));
            }
        }
    }
}
