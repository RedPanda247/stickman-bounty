use avian2d::prelude::*;
use bevy::{ecs::relationship::RelationshipSourceCollection, prelude::*};

use crate::player::Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemySeesPlayer;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        // Run LOS checks during the fixed update so results line up with physics/colliders
        app.add_systems(FixedUpdate, fixed_look_for_player);
    }
}

/// Raycast from each Enemy towards the Player and log whether the line-of-sight is blocked.
fn fixed_look_for_player(
    spatial_query: SpatialQuery,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    players: Query<(Entity, &Transform), With<Player>>,
    mut commands: Commands,
) {
    // If there's not exactly one player, bail out early.
    let Ok((player_entity, player_tf)) = players.single() else {
        return;
    };

    let player_pos = player_tf.translation.truncate();

    // For each enemy check if they can see the player
    for (enemy_entity, enemy_tf) in enemies.iter() {
        let enemy_pos = enemy_tf.translation.truncate();
        let distance_to_player = enemy_pos.distance(player_pos);

        // If the player is on the same cordinates af the player then obviously it sees the player
        if distance_to_player == 0.0 {
            commands.entity(enemy_entity).insert(EnemySeesPlayer);
            continue;
        }

        let dir = (player_pos - enemy_pos).normalize();

        let dir2 = Dir2::new(dir).expect("invalid direction for Dir2");
        let filter = SpatialQueryFilter::from_excluded_entities(enemy_entity.iter());

        let hit = spatial_query.cast_ray(enemy_pos, dir2, distance_to_player, true, &filter);

        if let Some(hit_data) = hit {
            if hit_data.entity == player_entity {
                commands.entity(enemy_entity).insert(EnemySeesPlayer);
            } else {
                commands.entity(enemy_entity).remove::<EnemySeesPlayer>();
            }
        }
    }
}
