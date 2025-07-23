use super::camera_controller::CameraController;
use crate::game::{
    level::targets::{DeadTarget, Target},
    shooting,
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier3d::{plugin::RapierContext, prelude::*};
use multiplayer_demo::{Player, PlayerEntity, PlayerShoot};
use renet::{DefaultChannel, RenetClient};
#[derive(Component)]
pub struct Shootable;

#[derive(Component)]
pub struct TracerSpawnSpot;
pub fn update_player(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<CameraController>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    target_query: Query<Option<&Target>, With<Shootable>>,
    player_entity_query: Query<&PlayerEntity>,
    spawn_spot: Query<&GlobalTransform, With<TracerSpawnSpot>>,
    mut client: ResMut<RenetClient>,
) {
    let spawn_spot = spawn_spot.get_single().unwrap();
    let window = window_query.get_single().unwrap();
    let (camera, camera_global_transform) = camera_query.get_single().unwrap();

    if let Ok((mut player, _transform)) = player_query.get_single_mut() {
        if mouse_input.just_pressed(MouseButton::Left) {
            if player.ammo[&player.actual_weapon] > 0. {
                // Clone ou copier l'arme actuelle
                let actual_weapon = player.actual_weapon.clone();
                if let Some(ammo) = player.ammo.get_mut(&actual_weapon) {
                    *ammo -= 1.0;
                }
                player.health-=5.;
                // Calcul du rayon depuis le centre de l'écran
                let Some(ray) = camera.viewport_to_world(
                    &camera_global_transform,
                    Vec2::new(window.width() / 2., window.height() / 2.),
                ) else {
                    return;
                };

                // Détection de collision via raycast
                let predicate = |handle| target_query.get(handle).is_ok();
                let query_filter = QueryFilter::new().predicate(&predicate);
                let hit = rapier_context.cast_ray_and_get_normal(
                    ray.origin,
                    ray.direction.into(),
                    f32::MAX,
                    true,
                    query_filter,
                );

                if let Some((entity, ray_intersection)) = hit {
                    // Si c'est une Target, on la "tue"
                    if let Ok(target) = target_query.get(entity) {
                        if target.is_some() {
                            commands.entity(entity).insert(DeadTarget);
                        }
                    }

                    // Si c'est un joueur, on log son ID
                    if let Ok(PlayerEntity(client_id)) = player_entity_query.get(entity) {
                        info!("🎯 Joueur touché ! client_id = {:?}", client_id);
                        // TODO : envoyer l'événement au serveur ici si nécessaire
                        // ex : event_writer.send(PlayerHitEvent { victim_id: *client_id });
                        let shoot_msg = PlayerShoot {
                            weapon: actual_weapon,
                            from: spawn_spot.translation(),
                            to: ray_intersection.point,
                        };
                        let msg = bincode::serialize(&shoot_msg).unwrap();
                        client.send_message(DefaultChannel::ReliableOrdered, msg);
                    }

                    // Tracer visuel
                    let tracer_material = StandardMaterial {
                        base_color: Color::srgb(1.0, 0.0, 0.0),
                        unlit: true,
                        ..default()
                    };

                    commands.spawn((
                        PbrBundle {
                            transform: Transform::from_translation(Vec3::splat(f32::MAX)),
                            mesh: meshes.add(Cuboid::from_size(Vec3::new(0.15, 0.15, 1.0))),
                            material: materials.add(tracer_material),
                            ..default()
                        },
                        shooting::tracer::BulletTracer::new(
                            spawn_spot.translation(),
                            ray_intersection.point,
                            250.0,
                        ),
                    ));
                }
            }
        }
    }
}
