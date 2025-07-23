use std::collections::HashMap;

use bevy::{
    ecs::{component::Component, entity::Entity, event::Event, system::Resource},
    math::{Quat, Vec3},
};
use renet::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    PlayerMove([f32; 3]),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    LobbySync(HashMap<ClientId, PlayerAttributes>),
    PlayerJoin(ClientId),
    PlayerLeave(ClientId),
}
#[derive(Resource)]
pub struct ServerPlayerRegistry(pub HashMap<ClientId, PlayerAttributes>);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Weapon {
    Gun,
    Shotgun,
    Gatling,
    RocketLauncher,
    Bfg,
}

#[derive(Event)]
pub struct PlayerHitEvent {
    pub victim_id: ClientId,
    pub damage: f32,
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

#[derive(Component, PartialEq)]
pub struct Player {
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
    pub position: [f32; 3],
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

// pub fn gunEntity(mut commands: Commands, asset_server: Res<AssetServer>) -> Entity {
//     let gun_model = asset_server.load("models/gun2.glb#Scene0");
//     let gun_entity = commands
//         .spawn(SceneBundle {
//             scene: gun_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     return gun_entity;
// }

// pub fn shotgunEntity(mut commands: Commands, asset_server: Res<AssetServer>) -> Entity {
//     let shotgun_model = asset_server.load("models/shotgun2.glb#Scene0");
//     let shotgun_entity = commands
//         .spawn(SceneBundle {
//             scene: shotgun_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     return shotgun_entity;
// }

// pub fn gatlingEntity(mut commands: Commands, asset_server: Res<AssetServer>) -> Entity {
//     let gatling_model = asset_server.load("models/minigun.glb#Scene0");
//     let gatling_entity = commands
//         .spawn(SceneBundle {
//             scene: gatling_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     return gatling_entity;
// }

// pub fn rocketEntity(mut commands: Commands, asset_server: Res<AssetServer>) -> Entity {
//     let rocket_launcher_model = asset_server.load("models/rocket.glb#Scene0");
//     let rocket_launcher_entity = commands
//         .spawn(SceneBundle {
//             scene: rocket_launcher_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     return rocket_launcher_entity;
// }

// pub fn bfgEntity(mut commands: Commands, asset_server: Res<AssetServer>) -> Entity {
//     let bfg_model = asset_server.load("models/bfg2.glb#Scene0");
//     let bfg_entity = commands
//         .spawn(SceneBundle {
//             scene: bfg_model,
//             transform: Transform::IDENTITY,
//             visibility: Visibility::Hidden,
//             ..Default::default()
//         })
//         .id();
//     return bfg_entity;
// }

// #[derive(Resource, Clone)]
// pub struct GunEntity(pub Entity);
// #[derive(Resource, Clone)]
// pub struct ShotgunEntity(pub Entity);
// #[derive(Resource, Clone)]
// pub struct GatlingEntity(pub Entity);
// #[derive(Resource, Clone)]
// pub struct RocketEntity(pub Entity);
// #[derive(Resource, Clone)]
// pub struct BfgEntity(pub Entity);
