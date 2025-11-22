use avian2d::prelude::*;
use bevy::prelude::*;

use crate::abilities::*;
use crate::enemy::*;
use crate::game_data::*;
use crate::loading::*;
use crate::main_menu::*;
use crate::player::*;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LoadLevelEntities>()
            .add_systems(Update, ev_load_level_entities)
            .add_systems(OnEnter(GameState::LevelComplete), spawn_level_complete_ui)
            .add_systems(
                Update,
                level_ui_button_interactions.run_if(in_state(GameState::LevelComplete).or(in_state(GameState::LevelPaused))),
            )
            .add_systems(OnEnter(GameState::LevelPaused), (spawn_level_paused_ui, pause_physics))
            .add_systems(OnExit(GameState::LevelPaused), resume_physics)
            .add_observer(close_level_menu)
            .add_systems(Update, pause_game);
    }
}

#[derive(Component)]
enum LevelUiButton {
    ReturnToMainMenu,
    Resume,
}
#[derive(Component)]
struct LevelMenuUIRoot;

fn spawn_level_paused_ui(mut commands: Commands) {
    commands.spawn((
        LevelMenuUIRoot,
        GameEntity::LevelEntity,
        BackgroundColor(Color::hsla(0., 0., 0., 0.5)),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (
                LevelUiButton::ReturnToMainMenu,
                GrowOnHover,
                Button,
                BorderColor::all(Color::WHITE),
                BorderRadius::MAX,
                BackgroundColor(Color::BLACK),
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
                children![(
                    Text::new("Back to main menu"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextShadow::default(),
                )],
            ),
            (
                LevelUiButton::Resume,
                GrowOnHover,
                Button,
                BorderColor::all(Color::WHITE),
                BorderRadius::MAX,
                BackgroundColor(Color::BLACK),
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
                children![(
                    Text::new("Resume game"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextShadow::default(),
                )],
            )
        ],
    ));
}

fn pause_game(
    input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::Escape) {
        if **current_state == GameState::LevelPaused {
            commands.trigger(CloseLevelMenu);
        } else if **current_state == GameState::PlayingLevel {
            next_state.set(GameState::LevelPaused);
        }
    }
}

fn pause_physics(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

fn resume_physics(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

#[derive(Event)]
struct CloseLevelMenu;

fn close_level_menu(
    _close_menu: On<CloseLevelMenu>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    level_menu_ui_root_qy: Query<Entity, With<LevelMenuUIRoot>>,
) {
    game_state.set(GameState::PlayingLevel);
    // Delete Menu UI
    for entity in level_menu_ui_root_qy.iter() {
        commands.entity(entity).despawn();
    }
}

fn level_ui_button_interactions(
    qy_main_menu_buttons: Query<
        (&Interaction, &LevelUiButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_load_game_state: MessageWriter<LoadGameState>,
    mut commands: Commands,
) {
    for (interaction, button) in qy_main_menu_buttons.iter() {
        if let Interaction::Pressed = interaction {
            match button {
                LevelUiButton::ReturnToMainMenu => {
                    ev_load_game_state.write(LoadGameState {
                        game_state_to_load: LoadableGameStates::MainMenu,
                        loading_screen: LoadingScreen::Basic,
                    });
                }
                LevelUiButton::Resume => {
                    commands.trigger(CloseLevelMenu);
                }
            }
        }
    }
}

fn spawn_level_complete_ui(mut commands: Commands) {
    commands.spawn((
        GameEntity::LevelEntity,
        BackgroundColor(Color::hsla(0., 0., 0., 0.5)),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![(
            LevelUiButton::ReturnToMainMenu,
            GrowOnHover,
            Button,
            BorderColor::all(Color::WHITE),
            BorderRadius::MAX,
            BackgroundColor(Color::BLACK),
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
            children![(
                Text::new("Back to main menu"),
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )],
        )],
    ));
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

                // Player
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
                    (
                        Enemy,
                        Health(100.),
                        ShootCooldown {
                            cooldown: 1.,
                            cooldown_start: None,
                        },
                    ),
                );
                spawn_character(
                    commands,
                    CharacterBundle {
                        size: default_character_size,
                        position: vec3(700., 700., 0.),
                        color: Color::srgb(8.0, 0.0, 0.0),
                    },
                    (
                        Enemy,
                        Health(100.),
                        ShootCooldown {
                            cooldown: 1.,
                            cooldown_start: None,
                        },
                    ),
                );
                spawn_character(
                    commands,
                    CharacterBundle {
                        size: default_character_size,
                        position: vec3(1000., 700., 0.),
                        color: Color::srgb(8.0, 0.0, 8.0),
                    },
                    (
                        Enemy,
                        BountyTarget,
                        Health(100.),
                        ShootCooldown {
                            cooldown: 3.,
                            cooldown_start: None,
                        },
                    ),
                );
                // Spawn Player UI
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
                        // BorderColor::all(Color::WHITE),
                        // BorderRadius::MAX,
                        // BackgroundColor(Color::BLACK),
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
