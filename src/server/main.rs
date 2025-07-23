use bevy::time::common_conditions::on_timer;
use bevy::{
    app::{App, Startup, Update},
    asset::{AssetApp, AssetPlugin},
    log::LogPlugin,
    scene::Scene,
    MinimalPlugins,
};
use std::time::Duration;
use bevy::prelude::*;
use bevy_renet::{transport::NetcodeServerPlugin, RenetServerPlugin};
use multiplayer_demo::PlayerLobby;
use renet::{
    transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    ConnectionConfig, RenetServer,
};
use std::{collections::HashMap, net::UdpSocket, time::SystemTime};
use systems::{handle_events_system, receive_message_system, send_message_system, setup_system};

mod resources;
mod systems;

const SERVER_ADDR: &str = "0.0.0.0:5000"; // Permet d'écouter toutes les IP locales

fn main() {
    let mut app = App::new();

    // base plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Scene>();
    app.add_plugins(LogPlugin::default());
    app.add_plugins(RenetServerPlugin);

    // renet server
    let server = RenetServer::new(ConnectionConfig::default());
    app.insert_resource(server);

    app.add_plugins(NetcodeServerPlugin);
    let server_addr = SERVER_ADDR.parse().unwrap();
    let socket = UdpSocket::bind(server_addr).expect("Échec du bind de l'UDP socket");
    let server_config = ServerConfig {
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        max_clients: 10,
        protocol_id: 0,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };
    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    app.insert_resource(transport);

    match local_ip_address::local_ip() {
        Ok(ip) => println!("Mon IP locale est : {}", ip),
        Err(e) => eprintln!("Erreur : {}", e),
    }
    // game systems
    app.insert_resource(PlayerLobby(HashMap::default()));

    app.add_systems(Startup, setup_system);

    let fixed_interval = Duration::from_secs_f32(1.0 / 60.0);

    app.add_systems(Update, send_message_system.run_if(on_timer(fixed_interval)));
    app.add_systems(
        Update,
        receive_message_system.run_if(on_timer(fixed_interval)),
    );
    app.add_systems(
        Update,
        handle_events_system.run_if(on_timer(fixed_interval)),
    );

    app.run();
}
