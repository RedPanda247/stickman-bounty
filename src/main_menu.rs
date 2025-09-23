use bevy::{prelude::*, input_focus};

use crate::level::*;

fn ui_button_ststem(mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut Button, &Children),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
    mut load_level_writer: EventWriter<LoadLevelEvent>,
) {
    for (entity, interaction, mut button, children) in &mut interaction_query {
        if let Ok(mut text) = text_query.get_mut(children[0]) {
            match *interaction {
                Interaction::Pressed => {
                    **text = "Clicked".to_string();
                    input_focus.set(entity);
                    button.set_changed();
                    load_level_writer.write(LoadLevelEvent { level: 1 });
                }
                Interaction::Hovered => {
                    **text = "Hovered".to_string();
                    input_focus.set(entity);
                    button.set_changed();
                }
                Interaction::None => {
                    **text = "Button".to_string();
                    input_focus.clear();
                }
            }
        }
    }
}


#[derive(Component)]
pub struct MainMenuEntity;
pub fn lead_main_menu_entities(mut commands: Commands) {
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
        children![
            (
                Button,
                Node {
                    width: Val::Auto,
                    height: Val::Auto,
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
                    Text::new("Button"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextShadow::default(),
                )]
            ),
            (
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
                    Text::new("Button"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextShadow::default(),
                )]
            )
        ],
    ));
}