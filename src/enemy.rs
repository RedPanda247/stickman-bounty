use avian2d::prelude::*;
use bevy::{ecs::relationship::RelationshipSourceCollection, prelude::*};

use crate::player::Player;

#[derive(Component)]
pub struct Enemy;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        // Run LOS checks during the fixed update so results line up with physics/colliders
        app.add_systems(FixedUpdate, fixed_look_for_player);
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
        let dist = enemy_pos.distance(player_pos);
        if dist == 0.0 {
            info!("Enemy {:?} is on top of player", enemy_entity);
            continue;
        }
        let dir = (player_pos - enemy_pos).normalize_or_zero();
        // info!("{} at {} looking for player in direction {}", enemy_entity, enemy_tf.translation, dir);

        // Call avian2d spatial query to get ray hits along the half-line up to `dist`.
        // The avian2d API expects a `Dir2` direction, a max hits count and a `SpatialQueryFilter`.
        let dir2 = Dir2::new(dir).expect("invalid direction for Dir2");
        let filter = SpatialQueryFilter::from_excluded_entities(enemy_entity.iter());

        // Request multiple hits (enough to catch all blocking colliders) and sort by distance

        // We need to allow detection of multiple entities because (the ray checks all entities/colliders that it detects)
        // so even if we set it to max hit 1 entity it will find all that is collides with and not store them in any specific order
        // and then proon the list in some way so that it is the max amount of RayHitData meaning 
        // If there is an object between the player and the ray then the ray will collide with the player and the object between
        // and maybe put the player first in the list and then remove the other entity so the list is of length 1 and then only the
        // player will be in the list so it looks like the ray only collided with the player
        let hits = spatial_query.ray_hits(enemy_pos, dir2, dist, 2, true, &filter);

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
