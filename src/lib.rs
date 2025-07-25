use std::collections::HashMap;

use bevy::{
    ecs::{component::Component, entity::Entity, system::Resource},
    math::{Quat, Vec3},
};
use renet::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    PlayerJoin(ClientId),
    PlayerLeave(ClientId),
    LobbySync(HashMap<ClientId, PlayerAttributes>),
    PlayerHit {
        new_health: f32,
        client_id: ClientId,
    },
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Weapon {
    Gun,
    Shotgun,
    Gatling,
    RocketLauncher,
    Bfg,
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerShoot {
    pub weapon: Weapon,
    pub from: Vec3,
    pub to: Vec3,
}

#[derive(Component)]
pub struct PlayerStats {
    pub username: String,
    pub health: f32,
    pub armor: f32,
    pub owned_weapon: HashMap<Weapon, bool>,
    pub actual_weapon: Weapon,
    pub ammo: HashMap<Weapon, f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerSync {
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32), // Quat as tuple
    pub health: f32,
    pub armor: f32,
    pub owned_weapon: HashMap<Weapon, bool>,
    pub actual_weapon: Weapon,
    pub ammo: HashMap<Weapon, f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShotFiredMessage {
    pub shooter_id: ClientId,        // client_id du tireur
    pub weapon: Weapon,              // arme utilisée
    pub target_id: Option<ClientId>, // client_id de la cible, s'il y en a une
}

#[derive(Component, Debug)]
pub struct PlayerEntity(pub ClientId);

#[derive(Debug, Component, PartialEq)]
pub struct Player {
    pub username: String,
    pub position: (f32, f32, f32),
    pub rotation: Quat,
    pub health: f32,
    pub armor: f32,
    pub velocity: Vec3,
    pub speed: f32,
    pub jump_strength: f32, // Ajoutez ce champ
    pub gravity: f32,
    pub owned_weapon: HashMap<Weapon, bool>,
    pub actual_weapon: Weapon,
    pub entities: HashMap<Weapon, Entity>,
    pub ammo: HashMap<Weapon, f32>, // Changement en HashMap
}

impl Player {
    // Fonction pour ajouter une arme dans le HashMap
    pub fn add_weapon(&mut self, weapon: Weapon) {
        self.owned_weapon.insert(weapon, true);
    }

    // Fonction pour vérifier si une arme est possédée
    pub fn has_weapon(&self, weapon: &Weapon) -> bool {
        *self.owned_weapon.get(weapon).unwrap_or(&false)
    }
}

#[derive(Resource, Clone)]
pub struct PlayerLobby(pub HashMap<ClientId, PlayerAttributes>);
impl Default for PlayerLobby {
    fn default() -> Self {
        PlayerLobby(HashMap::new())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerAttributes {
    pub username: String,
    pub position: (f32, f32, f32),
    pub rotation: Quat,
    pub health: f32,
    pub armor: f32,
    pub owned_weapon: HashMap<Weapon, bool>,
    pub actual_weapon: Weapon,
    pub ammo: HashMap<Weapon, f32>,
}

impl PlayerAttributes {
    // Fonction pour ajouter une arme dans le HashMap
    pub fn add_weapon(&mut self, weapon: Weapon) {
        self.owned_weapon.insert(weapon, true);
    }

    // Fonction pour vérifier si une arme est possédée
    pub fn has_weapon(&self, weapon: &Weapon) -> bool {
        *self.owned_weapon.get(weapon).unwrap_or(&false)
    }
}
