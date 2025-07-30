// src/client/resources.rs

// Import des modules standard pour la gestion des collections
use std::collections::HashSet;

// Import des modules Bevy pour les ressources et les assets
use bevy::{asset::Handle, ecs::system::Resource, render::texture::Image};
// Import du type ClientId de renet
use renet::ClientId;

/// Ressource contenant l'identifiant unique du client local
/// Utilisée pour identifier ce client dans les communications réseau
#[derive(Resource)]
pub struct MyClientId(pub ClientId);

/// Ressource contenant le nom d'utilisateur du joueur local
/// Stocke le nom choisi par l'utilisateur lors du démarrage du client
#[derive(Resource)]
pub struct MyUsername(pub String);

/// Implémentation des méthodes pour MyUsername
impl MyUsername {
    /// Crée une nouvelle instance de MyUsername avec le nom d'utilisateur fourni
    /// 
    /// # Arguments
    /// * `username` - Le nom d'utilisateur à stocker
    /// 
    /// # Returns
    /// * `MyUsername` - Une nouvelle instance avec le nom d'utilisateur
    pub fn new(username: String) -> Self {
        return Self(username);
    }
}

/// Ressource indiquant si le client est synchronisé avec le serveur
/// Utilisée pour contrôler l'envoi des messages (bloqué tant que non synchronisé)
/// Évite l'envoi de messages avant que la connexion soit établie
#[derive(Resource, Default)]
pub struct IsSynced(pub bool);

/// Ressource pour la gestion de la skybox (carte cubique du ciel)
/// Contient la texture de la skybox et un flag indiquant si elle est chargée
#[derive(Resource)]
pub struct SkyCubeMap {
    pub image: Handle<Image>, // Handle vers la texture de la skybox
    pub loaded: bool,         // Flag indiquant si la texture est chargée
}

/// Ressource pour suivre les joueurs qui ont été spawnés localement
/// Utilise un HashSet pour éviter les doublons et permettre une recherche rapide
/// Permet de savoir quels joueurs sont déjà représentés dans la scène 3D
#[allow(dead_code)] // Suppression de l'avertissement pour le code non utilisé
#[derive(Resource, Default)]
pub struct SpawnedPlayers(pub HashSet<ClientId>);
