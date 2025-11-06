use avian2d::prelude::*;
use bevy::prelude::*;

use crate::player::Player;

#[derive(Component)]
pub struct Enemy;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        // Run LOS checks during the fixed update so results line up with physics/colliders
        // app.add_systems(FixedUpdate, fixed_look_for_player);
    }
}

/// Raycast from each Enemy towards the Player and log whether the line-of-sight is blocked.
/// This uses `SpatialQuery::ray_hits(origin, dir, max_distance)` from avian2d.
fn fixed_look_for_player(
    mut spatial_query: SpatialQuery,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    players: Query<(Entity, &Transform), With<Player>>,
) {
    // If there's not exactly one player, bail out early.
    let Ok((player_entity, player_tf)) = players.single() else {
        return;
    };

    let player_pos = player_tf.translation.truncate();

    for (enemy_entity, enemy_tf) in enemies.iter() {
        let enemy_pos = enemy_tf.translation.truncate();
        let to_player = player_pos - enemy_pos;
        let dist = to_player.length();
        if dist == 0.0 {
            info!("Enemy {:?} is on top of player", enemy_entity);
            continue;
        }
        let dir = to_player / dist;

        // Call avian2d spatial query to get ray hits along the half-line up to `dist`.
        // The avian2d API expects a `Dir2` direction, a max hits count and a `SpatialQueryFilter`.
        let dir2 = Dir2::new(dir).expect("invalid direction for Dir2");
        let filter = SpatialQueryFilter::default();
        // ask for at most 1 hit and request sorted results so the closest is returned first
        let hits = spatial_query.ray_hits(enemy_pos, dir2, dist, 1, true, &filter);

        // Choose the nearest hit (smallest distance)
        if let Some(first_hit) = hits.iter().min_by(|a, b| a.distance.total_cmp(&b.distance)) {
            if first_hit.entity == player_entity {
                // Player is the first thing hit -> visible
                info!("Enemy {:?} has line-of-sight to player", enemy_entity);
            } else {
                // Some other collider occludes the player
                info!(
                    "Enemy {:?} LOS blocked by {:?} at distance {}",
                    enemy_entity, first_hit.entity, first_hit.distance
                );
            }
        } else {
            // No hits between enemy and player -> visible
            info!("Enemy {:?} sees player (no collider hits)", enemy_entity);
        }
    }
}
