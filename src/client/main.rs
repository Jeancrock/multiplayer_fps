use std::{
    io::{self, Write},
    net::{SocketAddrV4, UdpSocket},
    time::SystemTime,
};

use crate::{
    resources::MyClientId,
    systems::{
        handle_lobby_sync_event_system, handle_player_despawn_event_system,
        handle_player_spawn_event_system, receive_message_system, send_message_system,
    },
};
use bevy::{
    app::{App, Update},
    log::info,
    prelude::*,
    DefaultPlugins,
};
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};
use multiplayer_demo::{PlayerHitEvent, PlayerLobby};
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport},
    ClientId, ConnectionConfig, RenetClient,
};

mod components;
mod events;
pub mod game;
mod resources;
mod systems;

fn main() {
    print!("Entrez l'IP du serveur (ex: 192.168.1.10): ");
    io::stdout().flush().unwrap();

    let mut ipaddr = String::new();
    io::stdin().read_line(&mut ipaddr).expect("Échec lecture");
    let ipaddr = ipaddr.trim();

    let mut app = App::new();

    // base plugins
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);
    app.add_event::<PlayerHitEvent>();
    // renet client
    let client = RenetClient::new(ConnectionConfig::default());
    app.insert_resource(client);

    let client_id = rand::random::<u64>();
    app.insert_resource(MyClientId(ClientId::from_raw(client_id)));

    // Adresse serveur dans le réseau local
    let server_socket = SocketAddrV4::new(ipaddr.parse().expect("Adresse IP invalide"), 5000);

    let authentication = ClientAuthentication::Unsecure {
        server_addr: std::net::SocketAddr::V4(server_socket),
        client_id,
        user_data: None,
        protocol_id: 0,
    };

    // Bind sur une adresse locale automatique
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Échec du bind UDP client");

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    app.insert_resource(transport);

    // game build

    app.add_plugins((
        game::game::GamePlugin,
        DefaultPlugins.set(AssetPlugin {
            mode: AssetMode::Unprocessed,
            ..default()
        }),
    ));
    app.insert_resource(PlayerLobby::default());
    // game events
    app.add_event::<events::PlayerSpawnEvent>();
    app.add_event::<events::PlayerDespawnEvent>();
    app.add_event::<events::LobbySyncEvent>();

    // game systems
    app.add_systems(Update, send_message_system);
    app.add_systems(Update, receive_message_system);
    app.add_systems(Update, handle_player_spawn_event_system);
    app.add_systems(Update, handle_player_despawn_event_system);
    app.add_systems(Update, handle_lobby_sync_event_system);
    info!("Client {} started", client_id);

    app.run();
}
