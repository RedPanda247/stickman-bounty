use bevy::{
    prelude::*,
};
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .run();
}
fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(vec2(100., 100.)),
            ..default()
        },
    ));
}