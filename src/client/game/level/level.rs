use crate::{game::{player::player_shooting::Shootable, ui::map::MazeMap}, resources::SkyCubeMap};
use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_resource::{FilterMode, SamplerDescriptor, TextureViewDescriptor, TextureViewDimension},
        texture::ImageSampler,
    },
};
use bevy_rapier3d::prelude::*;

use super::targets;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        // Assurez-vous de ne pas ajouter de plugins par défaut manuellement
        app.add_plugins(targets::TargetsPlugin)
        // .insert_resource(SpawnSpots::default())
        .add_systems(Startup, init_level);
    }
}

fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    maze_map:Res<MazeMap>,
) {

    // Ajouter le sol avec le maillage personnalisé et le matériau transparent
    commands.spawn({
        SceneBundle {
            transform: Transform::IDENTITY,
            visibility: Visibility::Visible,
            ..default()
        }
    });

    // Créer un maillage personnalisé avec des UV ajustées pour répéter la texture
    let ground_mesh = Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(62.)));

    let level_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Transparent (RGBA avec alpha 0)
        alpha_mode: AlphaMode::Blend,                 // Activer la transparence
        metallic: 0.0,  // Pas de comportement métallique
        reflectance: 0.0,  // Pas de reflets spéculaires
        ..default()
    });

    // Ajouter le sol avec le maillage personnalisé et le matériau transparent
    commands.spawn((
        Collider::cuboid(62., 1., 62.), // Collider pour le sol
        PbrBundle {
            material: level_material.clone(),
            transform: Transform::IDENTITY,
            mesh: meshes.add(ground_mesh), // Utiliser le maillage avec UV ajustées
            ..default()
        },
        Shootable,
    ));

    // Créer un maillage personnalisé avec des UV ajustées pour répéter la texture
    let sky_mesh = Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(62.)));

    let sky_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.0, 0.0, 0.0), // Transparent (RGBA avec alpha 0)
        alpha_mode: AlphaMode::Blend,                 // Activer la transparence
        ..default()
    });

// Ajouter le ciel avec le maillage personnalisé et le matériau transparent
commands.spawn((
    Collider::cuboid(62., 0.1, 62.), // Collider pour le ciel
    PbrBundle {
        material: sky_material.clone(),
        transform: Transform {
            scale: Vec3::splat(-1.0), // Inverser pour voir de l'intérieur
            translation: Vec3::new(0., 10., 0.), // Position du sol
            ..default()
        },
        mesh: meshes.add(sky_mesh), // Utiliser le maillage avec UV ajustées
        ..default()
    },
    Shootable,
));

    // Définir la grille du labyrinthe (1 = mur, 0 = chemin)
    let maze_grid = maze_map.grid.clone();

    // Génération du labyrinthe à partir de la grille
    let maze_size = maze_grid.len();
    let cell_size = 4.0;

    // Charger la texture du sol
    let ground_texture_handle:Handle<bevy::prelude::Image> = asset_server.load("textures/moon_ground.png");
    
    // Avant de générer les murs, charge la texture et crée le matériau associé
    let wall_texture_handle = asset_server.load("textures/wall3.png");
    let wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(wall_texture_handle.clone()),
        unlit: true,  // Marque le matériau comme unlit
        ..default()
    });

    let mut spawn_spots = vec![];

    for (y, row) in maze_grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            // Créer un maillage personnalisé pour chaque case du sol
            let mut ground_mesh = Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(cell_size / 2.)));

            // Obtenir les UV et les ajuster pour que la texture se répète sans être déformée
            if let Some(VertexAttributeValues::Float32x2(uvs)) =
                ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
            {
                for uv in uvs.iter_mut() {
                    // Ajuster les coordonnées UV pour que la texture se répète sur chaque case
                    uv[0] *= 1.0; // Maintenir les proportions correctes sur l'axe X
                    uv[1] *= 1.0; // Maintenir les proportions correctes sur l'axe Y
                }
            }

            // Créer le sol pour chaque case
            commands.spawn((
                Collider::cuboid(cell_size / 2.0, 0.1, cell_size / 2.0), // Collider pour chaque case
                PbrBundle {
                    material: materials.add(
                        StandardMaterial {
                        base_color_texture: Some(ground_texture_handle.clone()), // Appliquer la texture du sol ici
                        metallic: 0.0,  // Pas de comportement métallique
                        reflectance: 0.0,  // Pas de reflets spéculaires
                        ..default()
                    }),
                    transform: Transform::from_xyz(
                        x as f32 * cell_size - (maze_grid.len() as f32 * cell_size / 2.0),
                        0.0, // Positionner le sol à la base
                        y as f32 * cell_size - (maze_grid[0].len() as f32 * cell_size / 2.0),
                    ),
                    mesh: meshes.add(ground_mesh), // Utiliser le maillage personnalisé pour chaque case
                    ..default()
                },
                Shootable,
            ));

            if cell == 1 {
                // Placer un mur là où la grille indique un mur (1)
                commands.spawn((
                    Collider::cuboid(cell_size / 2.0, cell_size / 2.0, cell_size / 2.0),
                    PbrBundle {
                        material: wall_material.clone(),
                        transform: Transform::from_xyz(
                            x as f32 * cell_size - (maze_size as f32 * cell_size / 2.0),
                            cell_size / 2.0, // hauteur des murs
                            y as f32 * cell_size - (maze_size as f32 * cell_size / 2.0),
                        ),
                        mesh: meshes.add(Cuboid::from_length(cell_size)),
                        ..default()
                    },
                    Shootable,
                ));
            }
            if cell == 2 {
                spawn_spots.push(Transform::from_xyz(x as f32, 0.0, y as f32));
            }
        }
    }

    // Ajouter des lumières
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 50_000.0,
            shadows_enabled: true,
            color:Color::srgba(0.737, 0.024, 0.012, 0.2),
            // color:Color::srgba(1.,1.,1.,1.),
            ..default()
        },
        transform: Transform::from_xyz(100.0, 200.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}



pub fn reinterpret_cubemap(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<SkyCubeMap>, // Modifier cubemap pour être mutable
    mut skyboxes: Query<&mut Skybox>,
) {
    
    // Vérifier si l'image est chargée et que cubemap.loaded est encore false
    if !cubemap.loaded && asset_server.load_state(&cubemap.image) == LoadState::Loaded {
        // Récupérer l'image et vérifier si elle est sous la bonne forme
        let image = images.get_mut(&cubemap.image).unwrap();

        // Si l'image est en 2D empilée, la réinterpréter en tant qu'array cubemap
        if image.texture_descriptor.array_layer_count() == 1 {
            let face_count = image.height() / image.width();
            image.reinterpret_stacked_2d_as_array(face_count);
            image.sampler = ImageSampler::Descriptor(SamplerDescriptor {
                mag_filter: FilterMode::Linear,  // Appliquer un filtrage linéaire
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Linear,
                ..default()
            }.into());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        // Mettre à jour l'image de la skybox pour toutes les entités correspondantes
        for mut skybox in &mut skyboxes {
            skybox.image = cubemap.image.clone();
        }

        // Marquer cubemap comme chargé pour éviter de réitérer cette opération
        cubemap.loaded = true;
    }
}