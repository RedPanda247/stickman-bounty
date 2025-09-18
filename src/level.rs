use bevy::prelude::*;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadLevelEntities>()
            .add_systems(Update, load_level_entities);
    }
}
#[derive(Event)]
pub struct LoadLevelEntities {
    pub level: u8,
}

pub fn load_level_entities(
    mut commands: Commands,
    mut ev_load_level_entities: EventReader<LoadLevelEntities>,
) {
    for event in ev_load_level_entities.read() {
        if event.level == 1 {
            commands.spawn((Sprite {
                color: Color::WHITE,
                custom_size: Some(vec2(100., 100.)),
                ..default()
            },));
        }
    }
}
