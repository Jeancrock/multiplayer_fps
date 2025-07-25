use super::camera_controller::CameraController;
use crate::game::{
    level::targets::{DeadTarget, Target},
    shooting,
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier3d::{plugin::RapierContext, prelude::*};
use multiplayer_demo::{Player, PlayerEntity, PlayerShoot};
use renet::{DefaultChannel, RenetClient};

/// Marqueur pour les entités pouvant être touchées par les tirs (targets, joueurs)
#[derive(Component)]
pub struct Shootable;

/// Position d'apparition du tracer visuel (ex : canon de l'arme)
#[derive(Component)]
pub struct TracerSpawnSpot;

/// Gère le tir du joueur local : collision, dégâts, envoi réseau, effets visuels
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
    // Récupère la position du canon de l'arme
    let spawn_spot = match spawn_spot.get_single() {
        Ok(s) => s,
        Err(_) => return, // Si pas défini, on ne peut pas tirer
    };

    // Récupère la fenêtre principale
    let window = match window_query.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    // Récupère la caméra active
    let (camera, camera_transform) = match camera_query.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };

    // Récupère le joueur local
    if let Ok((mut player, _)) = player_query.get_single_mut() {
        // Tir déclenché par clic gauche
        if mouse_input.just_pressed(MouseButton::Left) {
            let actual_weapon = player.actual_weapon;

            // Vérifie les munitions restantes pour l'arme sélectionnée
            if let Some(ammo) = player.ammo.get_mut(&actual_weapon) {
                if *ammo <= 0. {
                    return; // Pas de munitions, ne tire pas
                }
                *ammo -= 1.0;
            } else {
                return; // Arme non trouvée dans les munitions (erreur possible)
            }

            // Calcule un rayon partant du centre de l'écran (viseur)
            let Some(ray) = camera.viewport_to_world(
                &camera_transform,
                Vec2::new(window.width() / 2., window.height() / 2.),
            ) else {
                return;
            };

            // Prépare un filtre de requête pour ne tester que les entités "Shootable"
            let predicate = |entity: Entity| target_query.get(entity).is_ok();
            let query_filter = QueryFilter::new().predicate(&predicate);

            // Lance un rayon dans le monde 3D pour détecter les collisions
            let hit = rapier_context.cast_ray_and_get_normal(
                ray.origin,
                ray.direction.into(),
                f32::MAX,
                true,
                query_filter,
            );

            if let Some((entity, intersection)) = hit {
                // Cible statique : insérer un marqueur de mort (DeadTarget)
                if let Ok(Some(_target)) = target_query.get(entity) {
                    commands.entity(entity).insert(DeadTarget);
                }

                // Joueur ennemi touché : envoi du message au serveur
                if let Ok(PlayerEntity(victim_id)) = player_entity_query.get(entity) {
                    info!("🎯 Joueur touché ! client_id = {:?}", victim_id);

                    let shoot_msg = PlayerShoot {
                        weapon: actual_weapon,
                        from: spawn_spot.translation(),
                        to: intersection.point,
                    };
                    let msg = bincode::serialize(&shoot_msg).unwrap();
                    client.send_message(DefaultChannel::ReliableOrdered, msg);
                }

                // Crée un tracer visuel (lazer rouge) entre spawn et point d'impact
                let tracer_material = StandardMaterial {
                    base_color: Color::srgb(1.0, 0.0, 0.0),
                    unlit: true,
                    ..default()
                };

                commands.spawn((
                    PbrBundle {
                        transform: Transform::from_translation(Vec3::splat(f32::MAX)), // Hors champ initialement
                        mesh: meshes.add(Cuboid::from_size(Vec3::new(0.15, 0.15, 1.0))),
                        material: materials.add(tracer_material),
                        ..default()
                    },
                    shooting::tracer::BulletTracer::new(
                        spawn_spot.translation(),
                        intersection.point,
                        250.0,
                    ),
                ));
            }
        }
    }
}
