// src/client/system.rs

use bevy::{
    app::{App, Plugin, Update},
    asset::AssetServer,
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::info,
    math::Vec3,
    prelude::default,
    scene::SceneBundle,
    transform::components::Transform,
};
use bevy_rapier3d::prelude::Collider;
use multiplayer_demo::{PlayerAttributes, PlayerEntity, PlayerLobby, PlayerStats, ServerMessage};
use renet::{ClientId, DefaultChannel, RenetClient};

use crate::{
    events::{LobbySyncEvent, PlayerDespawnEvent, PlayerSpawnEvent},
    game::player::player_shooting::Shootable,
    resources::{IsSynced, MyUsername},
    MyClientId,
};

pub fn send_message_system(
    mut client: ResMut<RenetClient>,
    query: Query<(&PlayerAttributes, &Transform)>,
    username: Res<MyUsername>,
) {
    if let Ok((player, transform)) = query.get_single() {
        // Construct message
        let player_sync = PlayerAttributes {
            username: username.0.clone(),
            rotation: player.rotation,
            position: [
                transform.translation.x,
                transform.translation.y - 0.7,
                transform.translation.z,
            ]
            .into(),
            health: player.health,
            armor: player.armor,
            velocity: player.velocity,
            owned_weapon: player.owned_weapon.clone(),
            actual_weapon: player.actual_weapon,
            ammo: player.ammo.clone(),
            entities: player.entities.clone(),
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
    mut sync_state: ResMut<SyncState>,
    my_id: Res<MyClientId>,
    mut player_query: Query<(&mut PlayerAttributes, &mut Transform)>, // mutable Transform
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        if let Ok(server_message) = bincode::deserialize::<ServerMessage>(&message) {
            match server_message {
                ServerMessage::PlayerJoin(client_id) => {
                    info!("Client connected: {}", client_id);
                    if !sync_state.is_connected {
                        sync_state.is_connected = true;
                        sync_state.client_id = Some(client_id);
                        info!("Client ID enregistré dans SyncState: {:?}", client_id);
                    }
                    spawn_events.send(PlayerSpawnEvent(client_id));
                }
                ServerMessage::PlayerLeave(client_id) => {
                    info!("Client disconnected: {}", client_id);
                    despawn_events.send(PlayerDespawnEvent(client_id));
                }
                ServerMessage::LobbySync(map) => {
                    lobby_sync_events.send(LobbySyncEvent(map));
                }
                ServerMessage::PlayerHit {
                    new_health,
                    client_id,
                } => {
                    if Some(client_id) == sync_state.client_id {
                        if let Ok((mut player, _)) = player_query.get_single_mut() {
                            player.health = new_health;
                            info!("🔥 Dégât reçu ! Nouvelle vie : {}", new_health);
                        }
                    }
                }
                ServerMessage::PlayerDeath {
                    dead: client_id,
                    new_position: position,
                } => {
                    // Modifier la position si le joueur est celui local (à discuter selon logique)
                    if my_id.0 == client_id {
                        if let Ok((mut _player, mut transform)) = player_query.get_single_mut() {
                            // player.position = position;
                            let over_the_ground = 3.;
                            transform.translation =
                                Vec3::new(position.0, position.1 + over_the_ground, position.2);
                        }
                    }

                    info!("Mort reçue pour client {}", client_id);
                    despawn_events.send(PlayerDespawnEvent(client_id));
                }
            }
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        if let Ok(ServerMessage::LobbySync(map)) = bincode::deserialize(&message) {
            lobby_sync_events.send(LobbySyncEvent(map));
        }
    }
}

pub fn update_lobby_system(
    mut lobby: ResMut<PlayerLobby>,
    mut lobby_sync_events: EventReader<LobbySyncEvent>,
) {
    for event in lobby_sync_events.read() {
        lobby.0 = event.0.clone();
    }
}

pub fn handle_player_spawn_event_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
) {
    for event in spawn_events.read() {
        let client_id = event.0;
        info!("Tentative de spawn du joueur : {:?}", client_id);

        // ✅ Vérifie que les données du joueur sont bien disponibles
        commands.spawn((
            SceneBundle {
                scene: asset_server.load("models/guntest.glb#Scene0"),
                ..default()
            },
            Collider::cylinder(1.5, 0.5),
            PlayerEntity(client_id),
            Shootable,
        ));

        info!("✅ Joueur {:?} spawné avec succès", client_id);
    }
}

pub fn handle_player_despawn_event_system(
    mut commands: Commands,
    mut despawn_events: EventReader<PlayerDespawnEvent>,
    query: Query<(Entity, &PlayerEntity)>,
) {
    for event in despawn_events.read() {
        info!("Joueur déconnecté :: {:?}", event.0);
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
    mut is_synced: ResMut<IsSynced>,
) {
    if let Some(event) = sync_events.read().last() {
        for (client_id, player_sync) in event.0.iter() {
            let mut found = false;

            for (player_entity, mut transform, stats_opt) in query.iter_mut() {
                if *client_id == player_entity.0 {
                    transform.translation = player_sync.position.into();
                    transform.rotation = player_sync.rotation.into();

                    if let Some(mut stats) = stats_opt {
                        stats.health = player_sync.health;
                        stats.armor = player_sync.armor;
                        stats.owned_weapon = player_sync.owned_weapon.clone();
                        stats.actual_weapon = player_sync.actual_weapon;
                        stats.ammo = player_sync.ammo.clone();
                    }

                    found = true;
                    break;
                }
            }

            // ✅ Ne spawn PAS le joueur local ici (il est géré ailleurs)
            if !found && *client_id != my_client_id.0 {
                spawn_events.send(PlayerSpawnEvent(*client_id));
            }
        }

        is_synced.0 = true;
    }
}

#[derive(Resource, Default, Debug)]
pub struct SyncState {
    pub is_connected: bool,
    pub is_synced: bool,
    pub client_id: Option<ClientId>,
}

pub struct SyncStatePlugin;

impl Plugin for SyncStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SyncState>()
            .add_systems(Update, check_lobby_sync_system);
    }
}

fn check_lobby_sync_system(
    mut sync_state: ResMut<SyncState>,
    lobby: Res<PlayerLobby>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
) {
    if sync_state.is_connected && !sync_state.is_synced {
        if let Some(client_id) = sync_state.client_id {
            if lobby.0.contains_key(&client_id) {
                sync_state.is_synced = true;
                info!("Lobby synchronisé, client présent dans PlayerLobby");

                // 🔥 On déclenche le spawn du joueur local ici
                spawn_events.send(PlayerSpawnEvent(client_id));
            }
        }
    }
}
