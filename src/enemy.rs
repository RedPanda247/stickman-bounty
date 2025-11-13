use avian2d::prelude::*;
use bevy::{ecs::relationship::RelationshipSourceCollection, prelude::*};

use crate::game_data::*;
use crate::projectiles::*;
use crate::player::*;

#[derive(Component)]
pub struct Enemy;
#[derive(Component)]
struct ReadyToShoot;
#[derive(Component)]
pub struct ShootCooldown {
    pub cooldown: f32,
    pub cooldown_start: Option<f32>,
}

#[derive(Component)]
pub struct EnemySeesPlayer;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        // Run LOS checks during the fixed update so results line up with physics/colliders
        app.add_systems(
            FixedUpdate,
            (fixed_look_for_player, (update_shoot_cooldown, shoot_player).chain()).run_if(in_state(GameState::PlayingLevel)),
        );
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

const PROJECTILE_DAMAGE: f32 = 10.;

fn shoot_player(
    enemy_qy: Query<
        (Entity, &Transform),
        (With<Enemy>, (With<ReadyToShoot>, With<EnemySeesPlayer>)),
    >,
    mut cooldown_qy: Query<&mut ShootCooldown>,
    player_qy: Query<&Transform, With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_qy.single() {
        for (enemy_entity, enemy_transform) in enemy_qy.iter() {
            let dir_to_player =
                (player_transform.translation - enemy_transform.translation).truncate().normalize();
            println!("enemy shot: {}", enemy_entity);
            spawn_projectile(
                &mut commands,
                vec3(enemy_transform.translation.x, enemy_transform.translation.y + 200., 0.),
                // enemy_transform.translation,
                dir_to_player,
                PROJECTILE_DEFAULT_VELOCITY,
                PROJECTILE_DAMAGE,
                PROJECTILE_DEFAULT_KNOCKBACK,
                vec![enemy_entity],
            );
            commands.entity(enemy_entity).remove::<ReadyToShoot>();
            // If the entity has a ShootCooldown component reset the cooldown start time
            if let Ok(mut cooldown) = cooldown_qy.get_mut(enemy_entity) {
                println!("updated cooldown");
                cooldown.cooldown_start = Some(time.elapsed_secs());
            }
        }
    } else {
        warn!("couldn't get single player query in shoot_player system");
    }
}

fn update_shoot_cooldown(
    mut cooldown_qy: Query<(Entity, &mut ShootCooldown)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    // For each entity with ShootCooldown component
    for (cooldown_entity, mut cooldown) in cooldown_qy.iter_mut() {
        // If it has a start time for the cooldown
        if let Some(cooldown_start) = cooldown.cooldown_start {
            // If the cooldown is not done skip to next entity
            if (time.elapsed_secs() - cooldown_start) <= cooldown.cooldown {
                continue;
            }
        } else {
            // Set the start time of the cooldown to now
            cooldown.cooldown_start = Some(time.elapsed_secs());
        }
        // Add the ReadyToShoot component to entities that were not filtered away
        commands.entity(cooldown_entity).insert(ReadyToShoot);
    }
}
