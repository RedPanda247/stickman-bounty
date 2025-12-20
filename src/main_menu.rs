use bevy::prelude::*;

use crate::game_data::*;
use crate::level::LatestUnlockedLevel;
use crate::loading::*;

#[derive(Component)]
pub struct GrowOnHover;
#[derive(Component)]
enum MainMenuButton {
    StartGame,
}
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, grow_on_hover)
            .add_systems(
                Update,
                main_menu_buttons.run_if(in_state(GameState::MainMenu)),
            )
            .add_observer(load_main_menu_entities);
    }
}

fn main_menu_buttons(
    mut qy_main_menu_buttons: Query<
        (&Interaction, &StartLevelButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_load_game_state: MessageWriter<LoadGameState>,
    latest_unlocked_level: Res<LatestUnlockedLevel>,
) {
    for (interaction, button_type) in &mut qy_main_menu_buttons {
        if let Interaction::Pressed = interaction {
            match button_type {
                StartLevelButton(LevelIdentifier::Id(id)) => {
                    if *id <= latest_unlocked_level.0 as u8 {
                        ev_load_game_state.write(LoadGameState {
                            game_state_to_load: LoadableGameStates::Level(LevelIdentifier::Id(*id)),
                            loading_screen: LoadingScreen::Basic,
                        });
                    }
                    
                }
            }
        }
    }
}

fn grow_on_hover(
    mut interaction_query: Query<
        (&Interaction, &mut UiTransform),
        (Changed<Interaction>, With<GrowOnHover>),
    >,
) {
    for (interaction, mut transform) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                transform.scale = Vec2::splat(1.);
            }
            Interaction::Hovered => {
                transform.scale = Vec2::splat(1.1);
            }
            Interaction::None => {
                transform.scale = Vec2::splat(1.);
            }
        }
    }
}

#[derive(Component)]
struct StartLevelButton(LevelIdentifier);

#[derive(Event)]
pub struct LoadMainMenuEntities;

pub fn load_main_menu_entities(
    _: On<LoadMainMenuEntities>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    latest_unlocked_level: Res<LatestUnlockedLevel>,
) {
    

    let latest_unlocked = latest_unlocked_level.0;
    commands.spawn((
        GameEntity::MainMenuEntity,
        Text2d::new("Stickman Bounty"),
        Transform::from_xyz(0., 200., 0.),
        TextFont {
            font_size: 100.,
            ..default()
        },
        Name::new("Game title"),
    ));

    let asset_server_clone = asset_server.clone();

    commands.spawn((
        GameEntity::MainMenuEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            align_content: AlignContent::SpaceAround,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Children::spawn(SpawnIter((1..=2).into_iter().map(move |index| {
            (
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                Children::spawn((
                    if index > latest_unlocked {
                        Spawn(ImageNode::new(asset_server_clone.load("lock-64.png")))
                    } else {
                        Spawn(
                            ImageNode::new(asset_server_clone.load("lock-64.png"))
                                .with_color(Color::hsla(0., 0., 0., 0.)),
                        )
                    },
                    Spawn(if index >= latest_unlocked {
                        (
                            GrowOnHover,
                            StartLevelButton(LevelIdentifier::Id(index as u8)),
                            Button,
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
                            children![(
                                Text::new(format!("Start level {}", index)),
                                TextFont {
                                    font_size: 33.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                TextShadow::default(),
                            )],
                        )
                    } else {
                        (
                            GrowOnHover,
                            StartLevelButton(LevelIdentifier::Id(index as u8)),
                            Button,
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
                            children![(
                                Text::new(format!("Start level {}", index)),
                                TextFont {
                                    font_size: 33.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                TextShadow::default(),
                            )],
                        )
                    }),
                )),
            )
        }))),
    ));

    // commands.spawn((
    //     GameEntity::MainMenuEntity,
    //     Node {
    //         width: Val::Percent(100.0),
    //         height: Val::Percent(100.0),
    //         align_items: AlignItems::Center,
    //         align_content: AlignContent::SpaceAround,
    //         justify_content: JustifyContent::Center,
    //         flex_direction: FlexDirection::Column,
    //         ..default()
    //     },
    //     Name::new("main menu ui root"),
    //     children![
    //         (
    //             GrowOnHover,
    //             StartLevelButton(LevelIdentifier::Id(1)),
    //             Button,
    //             Node {
    //                 width: Val::Auto,
    //                 height: Val::Auto,
    //                 padding: UiRect::all(Val::Px(10.)),
    //                 border: UiRect::all(Val::Px(5.0)),
    //                 // horizontally center child text
    //                 justify_content: JustifyContent::Center,
    //                 // vertically center child text
    //                 align_items: AlignItems::Center,
    //                 ..default()
    //             },
    //             BorderColor::all(Color::WHITE),
    //             BorderRadius::MAX,
    //             BackgroundColor(Color::BLACK),
    //             children![(
    //                 Text::new("Start level 1"),
    //                 TextFont {
    //                     font_size: 33.0,
    //                     ..default()
    //                 },
    //                 TextColor(Color::srgb(0.9, 0.9, 0.9)),
    //                 TextShadow::default(),
    //             )]
    //         ),
    //         (
    //             GrowOnHover,
    //             StartLevelButton(LevelIdentifier::Id(2)),
    //             Button,
    //             Node {
    //                 width: Val::Auto,
    //                 height: Val::Auto,
    //                 padding: UiRect::all(Val::Px(10.)),
    //                 border: UiRect::all(Val::Px(5.0)),
    //                 // horizontally center child text
    //                 justify_content: JustifyContent::Center,
    //                 // vertically center child text
    //                 align_items: AlignItems::Center,
    //                 ..default()
    //             },
    //             BorderColor::all(Color::WHITE),
    //             BorderRadius::MAX,
    //             BackgroundColor(Color::BLACK),
    //             children![(
    //                 Text::new("Start level 2"),
    //                 TextFont {
    //                     font_size: 33.0,
    //                     ..default()
    //                 },
    //                 TextColor(Color::srgb(0.9, 0.9, 0.9)),
    //                 TextShadow::default(),
    //             )]
    //         )
    //     ],
    // ));
}
