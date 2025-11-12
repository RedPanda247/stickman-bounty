use bevy::prelude::*;
use avian2d::prelude::*;
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

#[derive(Component)]
pub struct CharacterBundle {
    pub size: f32,
    pub position: Vec3,
    pub color: Color,
}

impl Default for CharacterBundle {
    fn default() -> Self {
        Self {
            size: 100.0,
            position: Vec3::new(0., 400., 0.),
            color: Color::BLACK,
        }
    }
}

pub fn spawn_character(
    commands: &mut Commands,
    bundle: CharacterBundle,
    additional_components: impl Bundle,
) -> Entity {
    commands
        .spawn((
            GameEntity::LevelEntity,
            CanBeHitByProjectile,
            Sprite {
                color: bundle.color,
                custom_size: Some(Vec2::new(bundle.size, bundle.size)),
                ..Default::default()
            },
            RigidBody::Dynamic,
            LinearVelocity::ZERO,
            LockedAxes::ROTATION_LOCKED,
            Transform::from_xyz(bundle.position.x, bundle.position.y, bundle.position.z),
            Collider::rectangle(bundle.size, bundle.size),

            additional_components,
        ))
        .id()
}