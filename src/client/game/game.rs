use super::{level::level, player::player, ui::ui, window::window};
use bevy::prelude::*;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};

/// Plugin principal du jeu qui regroupe tous les sous-plugins du client
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // Plugin de gestion du joueur (mouvement, actions, etc.)
            player::PlayerPlugin,

            // Plugin de physique 3D Rapier sans userdata personnalisé
            RapierPhysicsPlugin::<NoUserData>::default(),

            // Plugin de gestion du niveau (map, environnement)
            level::LevelPlugin,

            // Plugin pour la configuration et gestion de la fenêtre
            window::WindowSettingsPlugin,

            // Plugin pour l'interface utilisateur (UI)
            ui::UiPlugin,
        ));
    }
}
