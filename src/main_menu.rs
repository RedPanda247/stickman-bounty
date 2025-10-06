use std::{thread::sleep, time::Duration};

use bevy::prelude::*;

use crate::game_data::*;
use crate::loading::*;

#[derive(Component)]
struct GrowOnHover;
#[derive(Component)]
enum MainMenuButton {
    StartGame,
}
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, grow_on_hover).add_systems(
            Update,
            main_menu_buttons.run_if(in_state(GameState::MainMenu)),
        );
    }
}

fn main_menu_buttons(
    mut qy_main_menu_buttons: Query<
        (&Interaction, &MainMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_load_game_state: EventWriter<LoadGameState>,
) {
    for (interaction, mut button) in &mut qy_main_menu_buttons {
        if let Interaction::Pressed = interaction {
            match button {
                MainMenuButton::StartGame => {
                    ev_load_game_state.write(LoadGameState {
                        game_state_to_load: LoadableGameStates::MainMenu,
                        loading_screen: LoadingScreen::Basic,
                    });
                }
            }
        }
    }
}

fn grow_on_hover(
    mut interaction_query: Query<
        (&Interaction, &mut Transform),
        (Changed<Interaction>, With<GrowOnHover>),
    >,
) {
    for (interaction, mut transform) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                transform.scale = Vec3::splat(1.);
            }
            Interaction::Hovered => {
                transform.scale = Vec3::splat(1.1);
            }
            Interaction::None => {
                transform.scale = Vec3::splat(1.);
            }
        }
    }
}

#[derive(Component)]
pub struct MainMenuEntity;
pub fn load_main_menu_entities(commands: &mut Commands) {
    commands.spawn((
        MainMenuEntity,
        Text2d::new("Stickman Bounty"),
        Transform::from_xyz(0., 200., 0.),
        TextFont {
            font_size: 100.,
            ..default()
        },
        Name::new("Game title"),
    ));
    commands.spawn((
        MainMenuEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            align_content: AlignContent::SpaceAround,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Name::new("main menu ui root"),
        children![(
            GrowOnHover,
            MainMenuButton::StartGame,
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
            BorderColor(Color::WHITE),
            BorderRadius::MAX,
            BackgroundColor(Color::BLACK),
            children![(
                Text::new("Start game"),
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )]
        )],
    ));
}
