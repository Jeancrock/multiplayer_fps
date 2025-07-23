use std::collections::HashMap;

use bevy::{
    asset::AssetServer,
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    prelude::default,
    scene::SceneBundle,
    transform::components::Transform,
};
use bevy_rapier3d::prelude::Collider;
use multiplayer_demo::{
    Player, PlayerAttributes, PlayerEntity, PlayerStats, ServerMessage, Weapon
};
use renet::{DefaultChannel, RenetClient};

use crate::{
    events::{LobbySyncEvent, PlayerDespawnEvent, PlayerSpawnEvent},
    game::player::player_shooting::Shootable,
    MyClientId,
};

pub fn send_message_system(
    mut client: ResMut<RenetClient>,
    query: Query<(&Player, &Transform)>,
) {
    if let Ok((player, transform)) = query.get_single() {
        let player_sync = PlayerAttributes {
            position: [
                transform.translation.x,
                transform.translation.y - 0.7,
                transform.translation.z,
            ],
            rotation: player.rotation,
            health: player.health,
            armor: player.armor,
            owned_weapon: player.owned_weapon.clone(),
            actual_weapon: player.actual_weapon,
            ammo: player.ammo.clone(),
        };

        let message = bincode::serialize(&player_sync).unwrap();
        client.send_message(DefaultChannel::Unreliable, message);
    }
}

pub fn receive_message_system(
    mut client: ResMut<RenetClient>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut despawn_events: EventWriter<PlayerDespawnEvent>,
    mut lobby_sync_events: EventWriter<LobbySyncEvent>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        let server_message: ServerMessage = bincode::deserialize(&message).unwrap();

        match server_message {
            ServerMessage::PlayerJoin(client_id) => {
                info!("Client connected: {}", client_id);
                spawn_events.send(PlayerSpawnEvent(client_id));
            }
            ServerMessage::PlayerLeave(client_id) => {
                info!("Client disconnected: {}", client_id);
                despawn_events.send(PlayerDespawnEvent(client_id));
            }
            ServerMessage::LobbySync(map) => {
                lobby_sync_events.send(LobbySyncEvent(map));
            }
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        if let Ok(ServerMessage::LobbySync(map)) = bincode::deserialize(&message) {
            println!("{:?}", map);
            println!();
            lobby_sync_events.send(LobbySyncEvent(map));
        }
    }
}

pub fn handle_player_spawn_event_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
) {
    for event in spawn_events.read() {
        info!("Handling player spawn event: {:?}", event.0);
        let client_id = event.0;
        commands.spawn((
            SceneBundle {
                scene: asset_server.load("models/guntest.glb#Scene0"),
                ..default()
            },
            PlayerStats {
                health: 100.,
                armor: 0.,
                owned_weapon: HashMap::from([
                    (Weapon::Gun, true),
                    (Weapon::Shotgun, true),
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
            Collider::cylinder(1.5, 0.5),
            PlayerEntity(client_id),
            Shootable,
        ));
    }
}

pub fn handle_player_despawn_event_system(
    mut commands: Commands,
    mut despawn_events: EventReader<PlayerDespawnEvent>,
    query: Query<(Entity, &PlayerEntity)>,
) {
    for event in despawn_events.read() {
        info!("Handling player despawn event: {:?}", event.0);
        let client_id = event.0;

        for (entity, player_entity) in query.iter() {
            if player_entity.0 == client_id {
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}
pub fn handle_lobby_sync_event_system(
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut sync_events: EventReader<LobbySyncEvent>,
    mut query: Query<(&PlayerEntity, &mut Transform, Option<&mut PlayerStats>)>,
    my_client_id: Res<MyClientId>,
) {
    if let Some(event) = sync_events.read().last() {
        for (client_id, player_sync) in event.0.iter() {
            if *client_id == my_client_id.0 {
                continue;
            }

            let mut found = false;

            for (player_entity, mut transform, stats_opt) in query.iter_mut() {
                if *client_id == player_entity.0 {
                    transform.translation = player_sync.position.into();

                    transform.rotation = player_sync.rotation.into();
                    if let Some(mut stats) = stats_opt {
                        stats.health = player_sync.health;
                        stats.armor = player_sync.armor;
                        stats.owned_weapon = player_sync.owned_weapon.clone();
                        stats.actual_weapon = player_sync.actual_weapon.clone();
                        stats.ammo = player_sync.ammo.clone();
                    }

                    found = true;
                    break;
                }
            }

            if !found {
                spawn_events.send(PlayerSpawnEvent(*client_id));
            }
        }
    }
}
