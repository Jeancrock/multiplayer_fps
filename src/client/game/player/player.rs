use bevy::{core_pipeline::Skybox, prelude::*};
use bevy_rapier3d::prelude::*;
use multiplayer_demo::{Player, PlayerLobby, Weapon};
use std::collections::HashMap;

use crate::{
    game::{level::level::reinterpret_cubemap, player::weapon},
    resources::{MyClientId, MyUsername, SkyCubeMap},
};

use super::{
    camera_controller,
    input::*,
    player_movement::*,
    player_shooting::{update_player, Shootable, TracerSpawnSpot},
};
use crate::game::{math::coordinates::blender_to_world, shooting};
pub struct PlayerPlugin;

#[derive(Resource, Default)]
pub struct PlayerInitialized(pub bool);

// impl Plugin for PlayerPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_plugins(shooting::tracer::TracerPlugin)
//             .init_resource::<PlayerInput>()
//             .init_resource::<PlayerInitialized>()
//             .add_systems(
//                 Update,
//                 (
//                     update_movement_input,
//                     update_player,
//                     camera_controller::update_camera_controller,
//                     weapon::switch_weapon,
//                 ),
//             )
//             //physics timestep
//             .add_systems(FixedUpdate, update_movement)
//             .add_systems(Startup, init_player);
//     }
// }

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(shooting::tracer::TracerPlugin)
            .init_resource::<PlayerInput>()
            .init_resource::<PlayerInitialized>()
            // .add_systems(Startup, setup_ui_camera)
            .add_systems(
                Update,
                (
                    try_init_player,
                    reinterpret_cubemap.run_if(resource_exists::<SkyCubeMap>),
                    update_movement_input,
                    update_player,
                    camera_controller::update_camera_controller,
                    weapon::switch_weapon,
                ),
            )
            .add_systems(FixedUpdate, update_movement);
    }
}

// fn init_player(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut _meshes: ResMut<Assets<Mesh>>,
//     mut _materials: ResMut<Assets<StandardMaterial>>,
//     myId: Res<MyClientId>,
//     lobby: Res<PlayerLobby>,
// ) {
//     let fov = 103.0_f32.to_radians();
//     let sky_model = asset_server.load("models/sky4.png");
//     commands.insert_resource(SkyCubeMap {
//         image: sky_model.clone(),
//         loaded: false,
//     });
//     let spawn;
//     if let Some(attr) = lobby.0.get(&myId.0) {
//         spawn = attr.position;
//         // continue avec spawn...
//     } else {
//         error!("Client ID {:?} not found in PlayerLobby", myId.0);
//         return;
//     }
//     let camera_entity = commands
//         .spawn((
//             Camera3dBundle {
//                 transform: Transform::from_translation(spawn.into()),
//                 projection: Projection::Perspective(PerspectiveProjection {
//                     fov: fov,
//                     ..default()
//                 }),
//                 ..default()
//             },
//             camera_controller::CameraController {
//                 sensitivity: 0.035,
//                 rotation: Vec2::ZERO,
//                 rotation_lock: 88.0,
//             },
//             Skybox {
//                 image: sky_model.clone(),
//                 brightness: 500.0,
//             },
//             Shootable,
//         ))
//         .id();
//     // Ajoute la caméra 2D UI juste après
//     commands.spawn(Camera2dBundle::default());
//     let gun_model = asset_server.load("models/guntest2.glb#Scene0");
//     let gun_entity = commands
//         .spawn(SceneBundle {
//             scene: gun_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     let shotgun_model = asset_server.load("models/test.glb#Scene0");
//     let shotgun_entity = commands
//         .spawn(SceneBundle {
//             scene: shotgun_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     let gatling_model = asset_server.load("models/minigun.glb#Scene0");
//     let gatling_entity = commands
//         .spawn(SceneBundle {
//             scene: gatling_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     let rocket_launcher_model = asset_server.load("models/rocket.glb#Scene0");
//     let rocket_launcher_entity = commands
//         .spawn(SceneBundle {
//             scene: rocket_launcher_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     let bfg_model = asset_server.load("models/bfg2.glb#Scene0");
//     let bfg_entity = commands
//         .spawn(SceneBundle {
//             scene: bfg_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     let spawn_spot = blender_to_world(Vec3::new(0.530462, 2.10557, -0.466568));
//     let tracer_spawn_entity = commands
//         .spawn((
//             TransformBundle {
//                 local: Transform::from_translation(spawn_spot),
//                 ..Default::default()
//             },
//             TracerSpawnSpot,
//         ))
//         .id();
//     let spawn = lobby.0[&myId.0].position;
//     let player_entity = commands
//         .spawn((
//             Player {
//                 position: spawn,
//                 rotation: Quat::IDENTITY,
//                 health: 100.,
//                 armor: 0.,
//                 velocity: Vec3::ZERO,
//                 speed: 4.0,
//                 jump_strength: 12.,
//                 gravity: 40.,
//                 owned_weapon: HashMap::from([
//                     (Weapon::Gun, true),
//                     (Weapon::Shotgun, true),
//                     (Weapon::Gatling, true),
//                     (Weapon::RocketLauncher, true),
//                     (Weapon::Bfg, true),
//                 ]),
//                 entities: HashMap::from([
//                     (Weapon::Gun, gun_entity),
//                     (Weapon::Shotgun, shotgun_entity),
//                     (Weapon::Gatling, gatling_entity),
//                     (Weapon::RocketLauncher, rocket_launcher_entity),
//                     (Weapon::Bfg, bfg_entity),
//                 ]),
//                 actual_weapon: Weapon::Gun,
//                 ammo: HashMap::from([
//                     (Weapon::Gun, 30.),
//                     (Weapon::Shotgun, 15.),
//                     (Weapon::Gatling, 50.),
//                     (Weapon::RocketLauncher, 5.),
//                     (Weapon::Bfg, 1.),
//                 ]),
//             },
//             SpatialBundle {
//                 // Spawn 30px au dessus du sol
//                 transform: Transform::from_translation(Vec3::new(spawn.0, spawn.1 + 3., spawn.2)),
//                 ..Default::default()
//             },
//             // Zone de colision du player
//             Collider::cuboid(1., 2., 1.),
//             RigidBody::KinematicPositionBased,
//             KinematicCharacterController {
//                 up: Vec3::Y,
//                 offset: CharacterLength::Absolute(0.01),
//                 ..default()
//             },
//         ))
//         .id();
//     commands.entity(camera_entity).push_children(&[
//         tracer_spawn_entity,
//         gun_entity,
//         shotgun_entity,
//         gatling_entity,
//         rocket_launcher_entity,
//         bfg_entity,
//     ]);
//     commands.entity(player_entity).add_child(camera_entity);
// }

/// Initialise le joueur local avec caméra, armes, et composants physiques,
/// une seule fois, à partir des données du lobby.
/// - `commands` : pour spawn les entités et insérer ressources
/// - `asset_server` : pour charger les modèles et textures
/// - `lobby` : état partagé des joueurs avec leurs attributs
/// - `my_id` : identifiant unique du joueur local
/// - `initialized` : ressource booléenne pour empêcher double initialisation
/// - `myusername` : nom du joueur local
pub fn try_init_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    lobby: Res<PlayerLobby>,
    my_id: Res<MyClientId>,
    mut initialized: ResMut<PlayerInitialized>,
    myusername: Res<MyUsername>,
) {
    // Quitte si déjà initialisé
    if initialized.0 {
        return;
    }

    // Récupère les attributs du joueur local depuis le lobby
    if let Some(attr) = lobby.0.get(&my_id.0) {
        let spawn = attr.position;
        let fov = 103.0_f32.to_radians();

        // Charge la skybox
        let sky_model = asset_server.load("models/sky4.png");
        commands.insert_resource(SkyCubeMap {
            image: sky_model.clone(),
            loaded: false,
        });

        // Spawn caméra avec contrôleur et skybox
        let camera_entity = commands
            .spawn((
                Camera3dBundle {
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov,
                        ..default()
                    }),
                    ..default()
                },
                camera_controller::CameraController {
                    sensitivity: 0.035,
                    rotation: Vec2::ZERO,
                    rotation_lock: 88.0,
                },
                Skybox {
                    image: sky_model.clone(),
                    brightness: 500.0,
                },
                Shootable,
            ))
            .id();

        // Spawn des armes, invisibles par défaut
        let gun_entity = commands.spawn(SceneBundle {
            scene: asset_server.load("models/guntest2.glb#Scene0"),
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        }).id();

        let shotgun_entity = commands.spawn(SceneBundle {
            scene: asset_server.load("models/test.glb#Scene0"),
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        }).id();

        let gatling_entity = commands.spawn(SceneBundle {
            scene: asset_server.load("models/minigun.glb#Scene0"),
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        }).id();

        let rocket_launcher_entity = commands.spawn(SceneBundle {
            scene: asset_server.load("models/rocket.glb#Scene0"),
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        }).id();

        let bfg_entity = commands.spawn(SceneBundle {
            scene: asset_server.load("models/bfg2.glb#Scene0"),
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        }).id();

        // Point d'apparition des projectiles/tracers dans l'espace du joueur
        let spawn_spot = blender_to_world(Vec3::new(0.530462, 2.10557, -0.466568));
        let tracer_spawn_entity = commands.spawn((
            TransformBundle {
                local: Transform::from_translation(spawn_spot),
                ..Default::default()
            },
            TracerSpawnSpot,
        )).id();

        // Spawn de l'entité joueur avec composants physique et gameplay
        let player_entity = commands.spawn((
            Player {
                username: myusername.0.clone(),
                position: spawn,
                rotation: attr.rotation,
                health: attr.health,
                armor: attr.armor,
                velocity: Vec3::ZERO,
                speed: 4.0,
                jump_strength: 12.,
                gravity: 40.,
                owned_weapon: attr.owned_weapon.clone(),
                entities: HashMap::from([
                    (Weapon::Gun, gun_entity),
                    (Weapon::Shotgun, shotgun_entity),
                    (Weapon::Gatling, gatling_entity),
                    (Weapon::RocketLauncher, rocket_launcher_entity),
                    (Weapon::Bfg, bfg_entity),
                ]),
                actual_weapon: attr.actual_weapon,
                ammo: attr.ammo.clone(),
            },
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(spawn.0, spawn.1 + 3., spawn.2)),
                ..Default::default()
            },
            Collider::cuboid(1., 2., 1.),
            RigidBody::KinematicPositionBased,
            KinematicCharacterController {
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                ..default()
            },
        )).id();

        // Organise la hiérarchie : caméra enfant du joueur, armes et point d'apparition enfant de la caméra
        commands.entity(camera_entity).push_children(&[
            tracer_spawn_entity,
            gun_entity,
            shotgun_entity,
            gatling_entity,
            rocket_launcher_entity,
            bfg_entity,
        ]);
        commands.entity(player_entity).add_child(camera_entity);

        // Marque comme initialisé pour ne pas refaire la création
        initialized.0 = true;
    }
}
