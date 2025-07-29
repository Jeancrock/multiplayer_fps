// src/server/system.rs

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use bevy::{
    ecs::{
        event::EventReader,
        system::{Res, ResMut},
    },
    log::info,
    math::{Quat, Vec3},
};
use multiplayer_demo::{
    PlayerAttributes, PlayerLobby, PlayerShoot, RecentlyRespawned, ServerMessage, Weapon,
};
use rand::{seq::SliceRandom, thread_rng};
use renet::{ClientId, DefaultChannel, RenetServer, ServerEvent};

use crate::{resources::SpawnSpots, SERVER_ADDR};

pub fn setup_system() {
    info!("Server started on {}", SERVER_ADDR);
}

pub fn send_message_system(mut server: ResMut<RenetServer>, player_lobby: Res<PlayerLobby>) {
    let channel = DefaultChannel::Unreliable;
    let lobby = player_lobby.0.clone();
    let event = ServerMessage::LobbySync(lobby);
    let message = bincode::serialize(&event).unwrap();
    server.broadcast_message(channel, message);
}

pub fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut player_lobby: ResMut<PlayerLobby>,
    mut recently_respawned: ResMut<RecentlyRespawned>,
) {
    for client_id in server.clients_id() {
        if let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            let skip_position = recently_respawned
                .0
                .get(&client_id)
                .map(|&t| t.elapsed() < Duration::from_millis(500))
                .unwrap_or(false);

            if let Some(existing) = player_lobby.0.get_mut(&client_id) {
                if let Ok(player_update) = bincode::deserialize::<PlayerAttributes>(&message) {
                    existing.username = player_update.username;
                    if !skip_position {
                        existing.position = player_update.position;
                    }
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

    // Nettoyage automatique des entrées expirées
    recently_respawned
        .0
        .retain(|_, &mut t| t.elapsed() < Duration::from_millis(500));
}

pub fn handle_events_system(
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    mut player_lobby: ResMut<PlayerLobby>,
    spawn_spots: Res<SpawnSpots>,
    mut recently_respawned: ResMut<RecentlyRespawned>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("✅ Client {client_id} connected");

                spawn_player(
                    &mut server,
                    &mut player_lobby,
                    &spawn_spots,
                    *client_id,
                    &mut recently_respawned,
                );
            }

            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("❌ Client {client_id} disconnected: {reason}");
                despawn_player(&mut server, &mut player_lobby, *client_id);
            }
        }
    }
}

fn spawn_player(
    server: &mut RenetServer,
    player_lobby: &mut PlayerLobby,
    spawn_spots: &SpawnSpots,
    client_id: ClientId,
    recently_respawned: &mut ResMut<RecentlyRespawned>,
) {
    let mut rng = thread_rng();
    let Some(&spawn) = spawn_spots.0.choose(&mut rng) else {
        eprintln!("⚠️ No spawn spots available for client {client_id}");
        return;
    };

    player_lobby
        .0
        .insert(client_id, default_player_attributes(spawn));

    let message = bincode::serialize(&ServerMessage::PlayerJoin(client_id)).unwrap();
    server.broadcast_message_except(client_id, DefaultChannel::ReliableOrdered, message);

    // Marque ce client comme récemment respawné
    recently_respawned.0.insert(client_id, Instant::now());
}

fn despawn_player(server: &mut RenetServer, player_lobby: &mut PlayerLobby, client_id: ClientId) {
    player_lobby.0.remove(&client_id);

    let message = bincode::serialize(&ServerMessage::PlayerLeave(client_id)).unwrap();
    server.broadcast_message(DefaultChannel::ReliableOrdered, message);
}

fn default_player_attributes(spawn: (f32, f32, f32)) -> PlayerAttributes {
    PlayerAttributes {
        username: "".to_string(),
        position: spawn,
        rotation: Quat::IDENTITY,
        health: 100.,
        armor: 0.,
        velocity: Vec3::ZERO,
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
        entities: HashMap::new(),
    }
}

pub fn receive_shoot_system(
    mut server: ResMut<RenetServer>,
    mut player_lobby: ResMut<PlayerLobby>,
    spawn_spots: Res<SpawnSpots>,
    mut recently_respawned: ResMut<RecentlyRespawned>,
) {
    let mut should_broadcast_lobby = false;
    let mut hits_to_apply = vec![];

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
        {
            if let Ok(shoot) = bincode::deserialize::<PlayerShoot>(&message) {
                for (victim_id, victim_attr) in player_lobby.0.iter() {
                    if *victim_id != client_id {
                        let victim_pos = Vec3::from_array(victim_attr.position.into());
                        let dir =
                            Vec3::from_array(shoot.to.into()) - Vec3::from_array(shoot.from.into());
                        let hit = ray_hits_player(victim_pos, shoot.from.into(), dir);

                        if hit {
                            println!("💥 Client {client_id} a touché {victim_id}");

                            let damage = match shoot.weapon {
                                Weapon::Gun => 17.,
                                Weapon::Shotgun => 28.,
                                Weapon::Gatling => 8.,
                                Weapon::RocketLauncher => 400.,
                                Weapon::Bfg => 800.,
                            };

                            hits_to_apply.push((client_id, *victim_id, damage));
                        }
                    }
                }
            }
        }
    }

    for (_shooter_id, victim_id, damage) in hits_to_apply {
        if let Some(victim_attr) = player_lobby.0.get_mut(&victim_id) {
            victim_attr.health -= damage;

            let hit_msg = ServerMessage::PlayerHit {
                new_health: victim_attr.health,
                client_id: victim_id,
            };
            let msg = bincode::serialize(&hit_msg).unwrap();
            server.send_message(victim_id, DefaultChannel::ReliableOrdered, msg);

            if victim_attr.health <= 0.0 {
                despawn_player(&mut server, &mut player_lobby, victim_id);
                spawn_player(
                    &mut server,
                    &mut player_lobby,
                    &spawn_spots,
                    victim_id,
                    &mut recently_respawned,
                );
                if let Some(victim_attr) = player_lobby.0.get_mut(&victim_id) {
                    let death_msg = ServerMessage::PlayerDeath {
                        dead: victim_id,
                        new_position: victim_attr.position,
                    };
                    let death_msg_bytes = bincode::serialize(&death_msg).unwrap();
                    server.send_message(
                        victim_id,
                        DefaultChannel::ReliableOrdered,
                        death_msg_bytes.clone(),
                    );
                    println!("server player_lobby : {:?}", player_lobby);
                }
                should_broadcast_lobby = true;
            }
        }
    }

    if should_broadcast_lobby {
        let lobby_msg = ServerMessage::LobbySync(player_lobby.0.clone());
        let msg = bincode::serialize(&lobby_msg).unwrap();
        server.broadcast_message(DefaultChannel::Unreliable, msg);
    }
}

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
