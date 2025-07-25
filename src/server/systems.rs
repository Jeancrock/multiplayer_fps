// src/server/main.rs

use std::collections::HashMap;

use bevy::{
    ecs::{
        event::EventReader,
        system::{Res, ResMut},
    },
    log::info,
    math::{Quat, Vec3},
};
use multiplayer_demo::{PlayerAttributes, PlayerLobby, PlayerShoot, ServerMessage, Weapon};
use rand::{seq::SliceRandom, thread_rng};
use renet::{DefaultChannel, RenetServer, ServerEvent};

use crate::{resources::SpawnSpots, SERVER_ADDR};

/// Affiche un message lors du démarrage du serveur
pub fn setup_system() {
    info!("Server started on {}", SERVER_ADDR);
}

/// Envoie l'état actuel du lobby à tous les clients via un message non fiable
pub fn send_message_system(mut server: ResMut<RenetServer>, player_lobby: Res<PlayerLobby>) {
    let channel = DefaultChannel::Unreliable;
    let lobby = player_lobby.0.clone();
    let event = multiplayer_demo::ServerMessage::LobbySync(lobby);
    let message = bincode::serialize(&event).unwrap();
    server.broadcast_message(channel, message);
}

/// Reçoit les messages des clients et met à jour leur état dans le lobby
pub fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut player_lobby: ResMut<PlayerLobby>,
) {
    for client_id in server.clients_id() {
        if let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            if let Some(existing) = player_lobby.0.get_mut(&client_id) {
                if let Ok(player_update) = bincode::deserialize::<PlayerAttributes>(&message) {
                    // Met à jour les champs modifiables
                    existing.username = player_update.username;
                    existing.position = player_update.position;
                    existing.rotation = player_update.rotation;
                    existing.owned_weapon = player_update.owned_weapon;
                    existing.actual_weapon = player_update.actual_weapon;
                    existing.ammo = player_update.ammo;

                } else {
                    println!(
                        "Failed to deserialize PlayerAttributes from client {}",
                        client_id
                    );
                }
            } else {
                println!("Received message from unknown client: {}", client_id);
            }
        }
    }
}

/// Gère les événements réseau : connexion/déconnexion des clients
pub fn handle_events_system(
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    mut player_lobby: ResMut<PlayerLobby>,
    spawn_spots: Res<SpawnSpots>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("✅ Client {client_id} connected");

                let mut rng = thread_rng();
                let spawn: (f32, f32, f32) = *spawn_spots.0.choose(&mut rng).unwrap();

                // Ajoute le joueur dans le lobby avec ses attributs de base
                player_lobby.0.insert(
                    *client_id,
                    PlayerAttributes {
                        username: "".to_string(),
                        position: spawn,
                        rotation: Quat::IDENTITY,
                        health: 100.,
                        armor: 0.,
                        owned_weapon: HashMap::from([
                            (Weapon::Gun, true),
                            (Weapon::Shotgun, false),
                            (Weapon::Gatling, false),
                            (Weapon::RocketLauncher, false),
                            (Weapon::Bfg, false),
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
                );

                // Notifie les autres clients qu'un nouveau joueur a rejoint
                let message =
                    bincode::serialize(&multiplayer_demo::ServerMessage::PlayerJoin(*client_id))
                        .unwrap();
                server.broadcast_message_except(
                    *client_id,
                    DefaultChannel::ReliableOrdered,
                    message,
                );
            }

            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("❌ Client {client_id} disconnected: {reason}");
                player_lobby.0.remove(client_id);

                let message =
                    bincode::serialize(&multiplayer_demo::ServerMessage::PlayerLeave(*client_id))
                        .unwrap();
                server.broadcast_message(DefaultChannel::ReliableOrdered, message);
            }
        }
    }
}


pub fn receive_shoot_system(
    mut server: ResMut<RenetServer>,
    mut player_lobby: ResMut<PlayerLobby>,
    server_spawns: Res<SpawnSpots>,
) {
    let mut should_broadcast_lobby = false;

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
        {
            if let Ok(shoot) = bincode::deserialize::<PlayerShoot>(&message) {
                for (victim_id, victim_attr) in player_lobby.0.iter_mut() {
                    if *victim_id != client_id {
                        let victim_pos = Vec3::from_array(victim_attr.position.into());
                        let dir =
                            Vec3::from_array(shoot.to.into()) - Vec3::from_array(shoot.from.into());
                        let hit = ray_hits_player(victim_pos, shoot.from.into(), dir);

                        if hit {
                            println!("💥 Client {client_id} a touché {victim_id}");
                            match shoot.weapon{
                                Weapon::Gun => victim_attr.health -= 17.,
                                Weapon::Shotgun => victim_attr.health -= 28.,
                                Weapon::Gatling => victim_attr.health -= 8.,
                                Weapon::RocketLauncher => victim_attr.health -= 400.,
                                Weapon::Bfg => victim_attr.health -= 800.,
                            }

                            let hit_msg = ServerMessage::PlayerHit {
                                new_health: victim_attr.health,
                                client_id: *victim_id,
                            };
                            let msg = bincode::serialize(&hit_msg).unwrap();
                            server.send_message(*victim_id, DefaultChannel::ReliableOrdered, msg);

                            if victim_attr.health <= 0.0 {
                                println!("☠️ Joueur {victim_id} est mort, respawn en cours…");

                                victim_attr.health = 100.;
                                victim_attr.armor = 0.;
                                victim_attr.ammo = HashMap::from([
                                    (Weapon::Gun, 30.),
                                    (Weapon::Shotgun, 15.),
                                    (Weapon::Gatling, 50.),
                                    (Weapon::RocketLauncher, 5.),
                                    (Weapon::Bfg, 1.),
                                ]);
                                victim_attr.actual_weapon = Weapon::Gun;
                                victim_attr.owned_weapon = HashMap::from([
                                    (Weapon::Gun, true),
                                    (Weapon::Shotgun, false),
                                    (Weapon::Gatling, false),
                                    (Weapon::RocketLauncher, false),
                                    (Weapon::Bfg, false),
                                ]);

                                // Respawn à une position aléatoire
                                let mut rng = thread_rng();
                                if let Some((x, y, z)) = server_spawns.0.choose(&mut rng).copied() {
                                    victim_attr.position = [x, y, z].into();
                                    victim_attr.rotation = Quat::IDENTITY;
                                    println!("🔁 Respawn de {victim_id} à [{x}, {y}, {z}]");
                                }

                                // Juste respawné = true pour reset côté client

                                should_broadcast_lobby = true;
                            }
                        }
                    }
                }
            }
        }
    }

    if should_broadcast_lobby {
        let lobby_msg = ServerMessage::LobbySync(player_lobby.0.clone());
        let msg = bincode::serialize(&lobby_msg).unwrap();
        server.broadcast_message(DefaultChannel::Unreliable, msg);
    }
}

/// Détection basique de "hit" (à affiner)
fn ray_hits_player(pos: Vec3, from: [f32; 3], dir: Vec3) -> bool {
    let start = Vec3::from_array(from);
    let max_dist = 100.0;
    let radius = 1.0;

    let to = start + dir.normalize() * max_dist;
    let closest = closest_point_on_line(start, to, pos);
    pos.distance(closest) < radius
}

fn closest_point_on_line(a: Vec3, b: Vec3, p: Vec3) -> Vec3 {
    let ab = b - a;
    let t = ((p - a).dot(ab)) / ab.length_squared();
    a + ab * t.clamp(0.0, 1.0)
}
