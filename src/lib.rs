// Import des dépendances nécessaires pour la gestion des collections et des structures de données
use std::collections::HashMap;

// Import des modules Bevy pour l'ECS (Entity Component System)
use bevy::{
    ecs::{component::Component, entity::Entity, system::Resource},
    math::{Quat, Vec3},
};
// Import de renet pour la gestion des identifiants clients
use renet::ClientId;
// Import de serde pour la sérialisation/désérialisation des données
use serde::{Deserialize, Serialize};
// Import pour la gestion du temps
use std::time::Instant;

/// Énumération des messages envoyés par le serveur aux clients
/// Ces messages permettent la synchronisation entre le serveur et les clients
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    /// Message envoyé quand un nouveau joueur rejoint le serveur
    PlayerJoin(ClientId),
    /// Message envoyé quand un joueur quitte le serveur
    PlayerLeave(ClientId),
    /// Synchronisation de l'état du lobby avec tous les joueurs connectés
    LobbySync(HashMap<ClientId, PlayerAttributes>),
    /// Message envoyé quand un joueur est touché par un tir
    PlayerHit {
        client_id: ClientId,  // ID du joueur touché
        new_health: f32,      // Nouvelle santé du joueur
    },
    /// Message envoyé quand un joueur meurt
    PlayerDeath {
        dead: ClientId,                    // ID du joueur mort
        attr:PlayerAttributes,     // Nouvelle position de respawn
    },
}

/// Énumération des armes disponibles dans le jeu
/// Chaque arme a ses propres caractéristiques et comportements
#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Weapon {
    Gun,            // Arme de base (pistolet)
    Shotgun,        // Fusil à pompe
    Gatling,        // Mitrailleuse
    RocketLauncher, // Lance-roquettes
    Bfg,            // BFG (Big Fucking Gun) - arme puissante
}

/// Implémentation de l'affichage pour l'énumération Weapon
/// Permet de convertir une arme en chaîne de caractères pour l'affichage
impl std::fmt::Display for Weapon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Weapon::Gun => write!(f, "Gun"),
            Weapon::Shotgun => write!(f, "Shotgun"),
            Weapon::Gatling => write!(f, "Gatling"),
            Weapon::RocketLauncher => write!(f, "RocketLauncher"),
            Weapon::Bfg => write!(f, "Bfg"),
        }
    }
}

/// Structure représentant un tir effectué par un joueur
/// Contient les informations nécessaires pour traiter le tir côté serveur
#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerShoot {
    pub weapon: Weapon,  // Arme utilisée pour le tir
    pub from: Vec3,      // Position de départ du tir
    pub to: Vec3,        // Position de destination du tir
}

/// Composant Bevy représentant les statistiques d'un joueur
/// Cette structure est utilisée côté client pour gérer l'état local du joueur
#[derive(Component)]
pub struct PlayerStats {
    pub username: String,                           // Nom d'utilisateur du joueur
    pub health: f32,                                // Points de vie actuels
    pub armor: f32,                                 // Points d'armure actuels
    pub owned_weapon: HashMap<Weapon, bool>,        // Armes possédées par le joueur
    pub actual_weapon: Weapon,                      // Arme actuellement équipée
    pub ammo: HashMap<Weapon, f32>,                 // Munitions pour chaque arme
}

/// Structure de synchronisation des données joueur
/// Utilisée pour transmettre l'état d'un joueur entre le serveur et les clients
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerSync {
    pub position: (f32, f32, f32),                 // Position 3D du joueur
    pub rotation: (f32, f32, f32, f32),            // Rotation (Quaternion) du joueur
    pub health: f32,                               // Points de vie
    pub armor: f32,                                // Points d'armure
    pub owned_weapon: HashMap<Weapon, bool>,       // Armes possédées
    pub actual_weapon: Weapon,                     // Arme actuellement équipée
    pub ammo: HashMap<Weapon, f32>,                // Munitions disponibles
}

/// Message envoyé quand un tir est effectué
/// Contient les informations sur le tir pour la synchronisation réseau
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShotFiredMessage {
    pub shooter_id: ClientId,        // ID du joueur qui a tiré
    pub weapon: Weapon,              // Arme utilisée pour le tir
    pub target_id: Option<ClientId>, // ID de la cible touchée (optionnel)
}

/// Composant Bevy pour identifier une entité comme étant un joueur
/// Contient l'ID client associé à cette entité
#[derive(Component, Debug)]
pub struct PlayerEntity(pub ClientId);

/// Ressource Bevy représentant le lobby des joueurs
/// Contient une HashMap de tous les joueurs connectés avec leurs attributs
#[derive(Debug, Resource, Clone)]
pub struct PlayerLobby(pub HashMap<ClientId, PlayerAttributes>);

/// Implémentation de Default pour PlayerLobby
/// Crée un lobby vide par défaut
impl Default for PlayerLobby {
    fn default() -> Self {
        PlayerLobby(HashMap::new())
    }
}

/// Structure complète des attributs d'un joueur
/// Contient toutes les informations nécessaires pour représenter un joueur dans le jeu
#[derive(Serialize, Deserialize, Debug, Clone, Component)]
pub struct PlayerAttributes {
    pub username: String,                           // Nom d'utilisateur
    pub position: (f32, f32, f32),                 // Position dans l'espace 3D
    pub rotation: Quat,                             // Rotation (quaternion)
    pub health: f32,                                // Points de vie
    pub armor: f32,                                 // Points d'armure
    pub velocity: Vec3,                             // Vecteur de vélocité
    pub owned_weapon: HashMap<Weapon, bool>,        // Armes possédées
    pub actual_weapon: Weapon,                      // Arme actuellement équipée
    pub ammo: HashMap<Weapon, f32>,                 // Munitions par arme
    pub entities: HashMap<Weapon, Entity>,          // Entités 3D des armes
}

/// Implémentation des méthodes pour PlayerAttributes
impl PlayerAttributes {
    /// Ajoute une arme à l'inventaire du joueur
    /// 
    /// # Arguments
    /// * `weapon` - L'arme à ajouter
    pub fn add_weapon(&mut self, weapon: Weapon) {
        self.owned_weapon.insert(weapon, true);
    }

    /// Vérifie si le joueur possède une arme spécifique
    /// 
    /// # Arguments
    /// * `weapon` - L'arme à vérifier
    /// 
    /// # Returns
    /// * `bool` - True si le joueur possède l'arme, False sinon
    pub fn has_weapon(&self, weapon: &Weapon) -> bool {
        *self.owned_weapon.get(weapon).unwrap_or(&false)
    }
}

/// Ressource Bevy pour gérer les joueurs récemment respawnés
/// Utilisée pour éviter les problèmes de collision ou de tir immédiat après respawn
#[derive(Default, Resource)]
pub struct RecentlyRespawned(pub HashMap<ClientId, Instant>);
