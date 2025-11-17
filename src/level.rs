use avian2d::prelude::*;
use bevy::prelude::*;

use crate::abilities::*;
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
    asset_server: Res<AssetServer>,
) {
    for event in ev_load_level_entities.read() {
        load_level_entities(&mut commands, event.level.clone(), &asset_server);
    }
}

pub fn load_level_entities(
    commands: &mut Commands,
    level: LevelIdentifier,
    asset_server: &AssetServer,
) {
    match level {
        LevelIdentifier::Id(id) => {
            if id == 1 {
                let default_character_size = 40.;
                let ground_height = 100.;
                let ground_width = 10000.;

                commands.spawn((
                    Player,
                    CanBeHitByProjectile,
                    Health(100.),
                    CollisionEventsEnabled,
                    CollidingEntities::default(),
                    CanDash,
                    CanGrapple,
                    GameEntity::LevelEntity,
                    Sprite {
                        color: Color::srgb(0.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new(
                            default_character_size,
                            default_character_size,
                        )),
                        ..default()
                    },
                    RigidBody::Dynamic,
                    LinearVelocity::ZERO,
                    LockedAxes::ROTATION_LOCKED,
                    Transform::from_xyz(0., 400., 0.),
                    Collider::rectangle(default_character_size, default_character_size),
                ));
                commands.spawn((
                    GameEntity::LevelEntity,
                    CanBeHitByProjectile,
                    Sprite {
                        color: Color::srgb(0.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new(50., 800.)),
                        image: asset_server.load("example.png"),
                        image_mode: SpriteImageMode::Tiled {
                            tile_x: true,
                            tile_y: true,
                            stretch_value: 1.,
                        },
                        ..Default::default()
                    },
                    RigidBody::Static,
                    Transform::from_xyz(200., 0., 0.),
                    Collider::rectangle(50., 800.),
                ));
                commands.spawn((
                    GameEntity::LevelEntity,
                    CanBeHitByProjectile,
                    Sprite {
                        color: Color::srgb(0.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new(ground_width, ground_height)),
                        image: asset_server.load("example.png"),
                        image_mode: SpriteImageMode::Tiled {
                            tile_x: true,
                            tile_y: true,
                            stretch_value: 1.,
                        },
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
                        size: default_character_size,
                        position: vec3(500., 700., 0.),
                        color: Color::srgb(8.0, 0.0, 0.0),
                    },
                    (Enemy, Health(100.), ShootCooldown {cooldown: 1., cooldown_start: None}),
                );
                spawn_character(
                    commands,
                    CharacterBundle {
                        size: default_character_size,
                        position: vec3(700., 700., 0.),
                        color: Color::srgb(8.0, 0.0, 0.0),
                    },
                    (Enemy, Health(100.), ShootCooldown {cooldown: 1., cooldown_start: None}),
                );
                commands.spawn((
                    GameEntity::LevelEntity,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::End,
                        align_content: AlignContent::SpaceAround,
                        justify_content: JustifyContent::Start,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    children![(
                        Node {
                            width: Val::Auto,
                            height: Val::Auto,
                            padding: UiRect::all(Val::Px(10.)),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(Color::WHITE),
                        BorderRadius::MAX,
                        BackgroundColor(Color::BLACK),
                        children![
                            (
                                Text::new("Health: "),
                                TextFont {
                                    font_size: 33.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                TextShadow::default(),
                            ),
                            (
                                PlayerHealthUi,
                                Text::new(""),
                                TextFont {
                                    font_size: 33.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                TextShadow::default(),
                            )
                        ],
                    )],
                ));
            }
        }
    }
}
