use avian2d::prelude::*;
use bevy::prelude::*;

use crate::level::FacingDirection;
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
