use bevy::{core_pipeline::Skybox, prelude::*};
use bevy_rapier3d::prelude::*;
use multiplayer_demo::{Player, Weapon};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::{collections::HashMap, io::{self, Write}};

use crate::game::{level::level::SpawnSpots, player::weapon};

use super::{
    camera_controller,
    input::*,
    player_movement::*,
    player_shooting::{update_player, Shootable, TracerSpawnSpot},
};
use crate::game::{math::coordinates::blender_to_world, shooting};
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(shooting::tracer::TracerPlugin)
            .init_resource::<PlayerInput>()
            .add_systems(
                Update,
                (
                    update_movement_input,
                    update_player,
                    camera_controller::update_camera_controller,
                    weapon::switch_weapon,

                ),
            )
            //physics timestep
            .add_systems(FixedUpdate, update_movement)
            .add_systems(Startup, init_player);
    }
}

#[derive(Resource)]
pub struct SkyCubeMap {
    pub image: Handle<Image>,
    pub loaded: bool,
}

fn init_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    spawn_spots: Res<SpawnSpots>,
) {
    let fov = 103.0_f32.to_radians();
    let sky_model = asset_server.load("models/sky4.png");

    commands.insert_resource(SkyCubeMap {
        image: sky_model.clone(),
        loaded: false,
    });

    let mut rng = thread_rng();
    let spawn_transform = *spawn_spots
        .spots
        .choose(&mut rng)
        .unwrap_or(&Transform::IDENTITY);

    let camera_entity = commands
        .spawn((
            Camera3dBundle {
                transform: spawn_transform,
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: fov,
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
                brightness: 500.0, // Ajuste la luminosité selon les besoins
            },
            Shootable,
        ))
        .id();

    let gun_model = asset_server.load("models/guntest2.glb#Scene0");
    let gun_entity = commands
        .spawn(SceneBundle {
            scene: gun_model,
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .id();

    let shotgun_model = asset_server.load("models/test.glb#Scene0");
    let shotgun_entity = commands
        .spawn(SceneBundle {
            scene: shotgun_model,
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .id();

    let gatling_model = asset_server.load("models/minigun.glb#Scene0");
    let gatling_entity = commands
        .spawn(SceneBundle {
            scene: gatling_model,
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .id();
    let rocket_launcher_model = asset_server.load("models/rocket.glb#Scene0");
    let rocket_launcher_entity = commands
        .spawn(SceneBundle {
            scene: rocket_launcher_model,
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .id();
    let bfg_model = asset_server.load("models/bfg2.glb#Scene0");
    let bfg_entity = commands
        .spawn(SceneBundle {
            scene: bfg_model,
            transform: Transform::IDENTITY,
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .id();

    let spawn_spot = blender_to_world(Vec3::new(0.530462, 2.10557, -0.466568));

    let tracer_spawn_entity = commands
        .spawn((
            TransformBundle {
                local: Transform::from_translation(spawn_spot),
                ..Default::default()
            },
            TracerSpawnSpot,
        ))
        .id();

    let mut rng = thread_rng();
    let spawn_points = spawn_list_maker(); // Vec<(f32, f32, f32)>

    let spawn: (f32, f32, f32) = *spawn_points.choose(&mut rng).unwrap();


    print!("Entrez votre nom d'utilisateur (bobby): ");
    io::stdout().flush().unwrap();

    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Échec lecture");
    let _username = username.trim();

    let player_entity = commands
        .spawn((
            Player {
                position: spawn,
                rotation:Quat::IDENTITY,
                health: 100.,
                armor: 0.,
                velocity: Vec3::ZERO,
                speed: 4.0,
                jump_strength: 12.,
                gravity: 40.,
                owned_weapon: HashMap::from([
                    (Weapon::Gun, true),
                    (Weapon::Shotgun, true),
                    (Weapon::Gatling, true),
                    (Weapon::RocketLauncher, true),
                    (Weapon::Bfg, true),
                ]),
                entities: HashMap::from([
                    (Weapon::Gun, gun_entity),
                    (Weapon::Shotgun, shotgun_entity),
                    (Weapon::Gatling, gatling_entity),
                    (Weapon::RocketLauncher, rocket_launcher_entity),
                    (Weapon::Bfg, bfg_entity),
                ]),
                actual_weapon: Weapon::Gun,
                ammo: HashMap::from([
                    (Weapon::Gun, 30.),
                    (Weapon::Shotgun, 15.),
                    (Weapon::Gatling, 50.),
                    (Weapon::RocketLauncher, 5.),
                    (Weapon::Bfg, 1.),
                ]),
            },
            SpatialBundle {
                // Spawn 30px au dessus du sol
                transform: Transform::from_translation(Vec3::new(spawn.0, spawn.1 + 3., spawn.2)),
                ..Default::default()
            },
            // Zone de colision du player
            Collider::cuboid(1., 2., 1.),
            RigidBody::KinematicPositionBased,
            KinematicCharacterController {
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                ..default()
            },
        ))
        .id();
    commands.entity(camera_entity).push_children(&[
        tracer_spawn_entity,
        gun_entity,
        shotgun_entity,
        gatling_entity,
        rocket_launcher_entity,
        bfg_entity,
    ]);
    commands.entity(player_entity).add_child(camera_entity);
}

pub fn spawn_list_maker() -> Vec<(f32, f32, f32)> {
    let mut spawn_list: Vec<(f32, f32, f32)> = Vec::new();

    let maze_grid = vec![
        vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1,
        ],
        vec![
            1, 2, 0, 0, 0, 2, 1, 2, 0, 0, 0, 2, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0,
            2, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 2, 1, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0,
            0, 1,
        ],
        vec![
            1, 2, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 2, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            2, 1,
        ],
        vec![
            1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1,
            1, 1,
        ],
        vec![
            1, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0,
            2, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 2, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1,
            1, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 0, 1, 2, 0, 1, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 2,
            0, 1,
        ],
        vec![
            1, 2, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0,
            0, 1,
        ],
        vec![
            1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 2, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0,
            0, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1,
        ],
        vec![
            1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            2, 1,
        ],
        vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1,
        ],
    ];
    let size = 4.0;
    for (y, row) in maze_grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 2 {
                spawn_list.push((
                    x as f32 * size - (maze_grid[0].len() as f32 * size / 2.0),
                    2.0,
                    y as f32 * size - (maze_grid.len() as f32 * size / 2.0),
                ));
            }
        }
    }

    spawn_list
}
