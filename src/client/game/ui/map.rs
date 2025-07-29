use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use multiplayer_demo::PlayerLobby;
use renet::ClientId;

use crate::resources::MyClientId;

/// Ressource représentant la grille de la map 2D
#[derive(Resource)]
pub struct MazeMap {
    pub grid: Vec<Vec<i32>>,
    pub cell_size: f32,
    pub maze_size: usize,
}

/// Marqueur du container de la mini-carte UI
#[derive(Component)]
pub struct MiniMap;

/// Marqueur des points représentant les joueurs sur la mini-carte
#[derive(Component)]
pub struct PlayerDot {
    pub client_id: ClientId,
}
/// Marqueur des cases du labyrinthe affichées dans la mini-carte
#[derive(Component)]
pub struct MazeCell;

/// Setup initial du container MiniMap dans l'UI
pub fn setup_minimap(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                width: Val::Px(300.0),
                height: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0., 0., 0., 1.)),
            ..default()
        },
        MiniMap,
    ));
}

/// Affiche les cases du labyrinthe dans la mini-carte (1 = carré blanc)
pub fn setup_maze_grid(
    mut commands: Commands,
    maze_map: Res<MazeMap>,
    minimap_query: Query<Entity, With<MiniMap>>,
    existing_cells: Query<(), With<MazeCell>>,
) {
    if existing_cells.iter().next().is_some() {
        return;
    }

    let Ok(minimap_entity) = minimap_query.get_single() else {
        return;
    };

    let cell_pixel_size = 300.0 / maze_map.maze_size as f32;

    for (y, row) in maze_map.grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 1 {
                let px = x as f32 * cell_pixel_size;
                let py = (maze_map.maze_size as f32 - y as f32 - 1.0) * cell_pixel_size;

                commands.entity(minimap_entity).with_children(|parent| {
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                left: Val::Px(px),
                                bottom: Val::Px(py),
                                width: Val::Px(cell_pixel_size),
                                height: Val::Px(cell_pixel_size),

                                ..default()
                            },
                            inherited_visibility: InheritedVisibility::VISIBLE,

                            background_color: BackgroundColor(Color::srgba(1., 1., 1., 0.1)),
                            ..default()
                        },
                        MazeCell,
                    ));
                });
            }
        }
    }
}

pub fn update_player_dots(
    mut commands: Commands,
    maze_map: Res<MazeMap>,
    minimap_query: Query<Entity, With<MiniMap>>,
    mut dots_query: Query<(Entity, &mut Style, &PlayerDot)>,
    lobby: Res<PlayerLobby>,
    my_id: Res<MyClientId>,
) {
    let Ok(minimap_entity) = minimap_query.get_single() else {
        return;
    };

    let maze_center = (maze_map.maze_size as f32 / 2.0) * maze_map.cell_size;
    let scale = 300.0 / maze_map.maze_size as f32;

    let mut used_ids = std::collections::HashSet::new();

    for (client_id, player_attr) in &lobby.0 {
        used_ids.insert(*client_id);

        let (px, _py, pz) = player_attr.position;

        let x = ((px + maze_center) / maze_map.cell_size) * scale + 4.0;
        let y =
            (maze_map.maze_size as f32 - ((pz + maze_center) / maze_map.cell_size)) * scale - 6.0;

        // Existe déjà ?
        if let Some((_, mut style, _)) = dots_query
            .iter_mut()
            .find(|(_, _, dot)| dot.client_id == *client_id)
        {
            style.left = Val::Px(x);
            style.bottom = Val::Px(y);
        } else {
            let color = if *client_id == my_id.0 {
                Color::srgba(0.0, 50.0, 95.0, 1.0) // bleu
            } else {
                Color::srgba(1.0, 0.0, 0.0, 1.0) // rouge
            };

            commands.entity(minimap_entity).with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(x),
                            bottom: Val::Px(y),
                            width: Val::Px(3.5),
                            height: Val::Px(3.5),
                            ..default()
                        },
                        inherited_visibility: InheritedVisibility::VISIBLE,
                        background_color: BackgroundColor(color),
                        focus_policy: FocusPolicy::Pass,
                        ..default()
                    },
                    PlayerDot {
                        client_id: *client_id,
                    },
                ));
            });
        }
    }

    // Optionnel : supprimer les anciens dots qui ne sont plus dans le lobby
    for (entity, _, dot) in dots_query.iter_mut() {
        if !used_ids.contains(&dot.client_id) {
            commands.entity(entity).despawn_recursive();
        }
    }
}
