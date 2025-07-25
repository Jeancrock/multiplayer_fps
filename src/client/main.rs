// src/client/main

use std::{
    io::{self, Write},
    net::{SocketAddrV4, UdpSocket},
    time::SystemTime,
};

use bevy::{
    app::{App, Update},
    log::info,
    prelude::*,
    DefaultPlugins,
};
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};
use game::game::GamePlugin;
use multiplayer_demo::PlayerLobby;
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport},
    ClientId, ConnectionConfig, RenetClient,
};
use resources::SpawnedPlayers;
use systems::{update_lobby_system, SyncStatePlugin};

// Modules internes
use crate::{
    resources::{IsSynced, MyClientId, MyUsername},
    systems::{
        handle_lobby_sync_event_system, handle_player_despawn_event_system,
        handle_player_spawn_event_system, receive_message_system, send_message_system,
    },
};

// Déclarations des sous-modules
mod components;
mod events;
pub mod game;
mod resources;
mod systems;

fn main() {
    // --- IP du serveur ---
    print!("Entrez l'IP du serveur (ex: 192.168.1.10): ");
    io::stdout().flush().unwrap();
    let mut ipaddr = String::new();
    io::stdin()
        .read_line(&mut ipaddr)
        .expect("Échec lecture IP");
    let ipaddr = ipaddr.trim();

    // --- Nom du joueur ---
    let mut username = "".to_string();
    while username.trim().is_empty() {
        print!("Entrez votre nom d'utilisateur (ex: NoobMaster69): ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut username)
            .expect("Échec lecture nom");
        username = username.trim().to_string();
    }

    // --- Initialisation de l'app Bevy ---
    let mut app = App::new();

    // --- Initialisation réseau ---
    let client_id = rand::random::<u64>();
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Échec du bind UDP client");

    let server_socket = SocketAddrV4::new(ipaddr.parse().expect("Adresse IP invalide"), 5000);

    let authentication = ClientAuthentication::Unsecure {
        server_addr: server_socket.into(),
        client_id,
        user_data: None,
        protocol_id: 0,
    };

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .expect("Échec init transport client!");

    // --- Ajout des plugins réseau ---
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);

    // --- Insertion des ressources réseau ---
    app.insert_resource(RenetClient::new(ConnectionConfig::default()));
    app.insert_resource(transport);
    app.insert_resource(PlayerLobby::default());
    app.insert_resource(MyClientId(ClientId::from_raw(client_id)));
    app.insert_resource(MyUsername::new(username.clone()));
    app.insert_resource(SpawnedPlayers::default());
    app.insert_resource(IsSynced(false)); // blocage messages tant que non synchronisé

    // --- Plugins et config assets ---
    app.add_plugins((
        SyncStatePlugin,
        GamePlugin,
        DefaultPlugins.set(AssetPlugin {
            mode: AssetMode::Unprocessed,
            ..default()
        }),
    ));

    // --- Définition des événements personnalisés ---
    app.add_event::<events::PlayerSpawnEvent>();
    app.add_event::<events::PlayerDespawnEvent>();
    app.add_event::<events::LobbySyncEvent>();

    // --- Systèmes clients principaux ---
    app.add_systems(Update, receive_message_system);
    app.add_systems(Update, handle_player_spawn_event_system);
    app.add_systems(Update, handle_player_despawn_event_system);
    app.add_systems(Update, handle_lobby_sync_event_system);
    app.add_systems(Update, update_lobby_system);
    app.add_systems(
        Update,
        send_message_system.run_if(|synced: Res<IsSynced>| synced.0),
    );

    // --- Message de confirmation ---
    info!("Client {} started with username '{}'", client_id, username);

    // --- Démarrage Bevy ---
    app.run();
}