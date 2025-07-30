// Import du module Bevy pour la définition des ressources
use bevy::ecs::system::Resource;

/// Ressource représentant les points de spawn disponibles dans le niveau
/// Contient une liste de coordonnées 3D où les joueurs peuvent apparaître
#[derive(Resource)]
pub struct SpawnSpots(pub Vec<(f32, f32, f32)>);

/// Implémentation des méthodes pour SpawnSpots
impl SpawnSpots {
    /// Crée une nouvelle instance de SpawnSpots avec la liste des points de spawn
    ///
    /// # Returns
    /// * `SpawnSpots` - Une nouvelle instance avec les points de spawn générés
    pub fn new() -> Self {
        return Self(spawn_list_maker());
    }
}

/// Fonction qui génère la liste des points de spawn basée sur la grille du labyrinthe
/// Analyse la grille du niveau et identifie les positions marquées comme points de spawn
///
/// # Returns
/// * `Vec<(f32, f32, f32)>` - Liste des coordonnées 3D des points de spawn
pub fn spawn_list_maker() -> Vec<(f32, f32, f32)> {
    // Initialisation de la liste des points de spawn
    let mut spawn_list: Vec<(f32, f32, f32)> = Vec::new();

    // Définition de la grille du labyrinthe
    // 0 = espace vide, 1 = mur, 2 = point de spawn
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

    // Taille de chaque cellule de la grille en unités du monde
    let size = 4.0;

    // Parcours de la grille pour identifier les points de spawn (cellules avec valeur 2)
    for (y, row) in maze_grid.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 2 {
                // Conversion des coordonnées de la grille en coordonnées 3D du monde
                // Centrage de la grille autour de l'origine (0,0,0)
                spawn_list.push((
                    x as f32 * size - (maze_grid[0].len() as f32 * size / 2.0), // Position X
                    2.0, // Hauteur fixe à 2 unités au-dessus du sol
                    y as f32 * size - (maze_grid.len() as f32 * size / 2.0), // Position Z
                ));
            }
        }
    }

    // Retour de la liste complète des points de spawn
    spawn_list
}
