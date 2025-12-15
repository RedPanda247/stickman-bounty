use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::reflect::Tuple;

use crate::level::*;
use crate::player::*;
pub struct GameDataPlugin;

impl Plugin for GameDataPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameState>().init_state::<GameState>();
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
    LevelComplete,
    LevelPaused,
    GameOver,
}

#[derive(Component)]
pub struct GameCharacter;

#[derive(Clone)]
pub enum LevelIdentifier {
    Id(u8),
}

#[derive(Component)]
pub struct Defense(pub f32);
#[derive(Component)]
pub struct Health(pub f32);
#[derive(Component)]
pub struct CanBeHitByProjectile;

pub const PROJECTILE_DEFAULT_VELOCITY: f32 = 1_000.;
pub const PROJECTILE_DEFAULT_KNOCKBACK: f32 = 100_000.;

#[derive(Component)]
pub struct CharacterBundle {
    pub size: Vec2,
    pub position: Vec3,
    pub color: Color,
    pub custom_sprite: Option<Sprite>,
}

impl Default for CharacterBundle {
    fn default() -> Self {
        Self {
            size: Vec2::splat(100.),
            position: Vec3::new(0., 400., 0.),
            color: Color::BLACK,
            custom_sprite: None,
        }
    }
}

pub fn spawn_character(
    commands: &mut Commands,
    bundle: CharacterBundle,
    additional_components: impl Bundle,
) -> Entity {
    // Default sprite
    let mut sprite = Sprite {
        color: bundle.color,
        custom_size: Some(bundle.size),
        ..Default::default()
    };
    // If a custom sprite has been specified
    if let Some(custom_sprite) = bundle.custom_sprite {
        sprite = custom_sprite;
    }
    commands
        .spawn((
            GameEntity::LevelEntity,
            GameCharacter,
            FacingDirection::default(),
            CanBeHitByProjectile,
            sprite,
            RigidBody::Dynamic,
            Mass(800.),
            LinearVelocity::ZERO,
            LockedAxes::ROTATION_LOCKED,
            Transform::from_xyz(bundle.position.x, bundle.position.y, bundle.position.z),
            Collider::rectangle(bundle.size.x, bundle.size.y),
            additional_components,
        ))
        .id()
}

#[derive(Default)]
pub struct GroundSpawnData {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}
impl GroundSpawnData {
    pub fn new(x1: i32, x2: i32, y1: i32, y2: i32) -> Self {
        GroundSpawnData { 
            x1: x1 as f32, 
            x2: x2 as f32, 
            y1: y1 as f32, 
            y2: y2 as f32, 
        }
    }
}

pub fn spawn_ground(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ground_spawn_data: GroundSpawnData,
) {
    let GroundSpawnData { x1, x2, y1, y2 } = ground_spawn_data;
    commands.spawn((
        GameEntity::LevelEntity,
        Ground,
        CanBeHitByProjectile,
        Sprite {
            color: Color::srgb(0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(x2 - x1, y2 - y1)),
            image: asset_server.load("ground.jpg"),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.,
            },
            ..Default::default()
        },
        RigidBody::Static,
        Transform::from_xyz(avg([x1, x2]), avg([y1, y2]), 0.),
        Collider::rectangle(x2 - x1, y2 - y1),
    ));
}

fn avg<T, I>(iter: I) -> f32
where
    T: Into<f32> + std::ops::Add<Output = T> + Default + Copy,
    I: IntoIterator<Item = T>,
{
    let items: Vec<T> = iter.into_iter().collect();
    if items.is_empty() {
        return 0.0;
    }

    let sum = items
        .iter()
        .copied()
        .reduce(|a, b| a + b)
        .unwrap_or_default();
    sum.into() / items.len() as f32
}
