use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::{Duration, Instant, SystemTime};

use bevy::ecs::system::IntoSystem;
use bevy::prelude::IntoSystemConfigs;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::{
    app::{App, Startup, Update},
    asset::AssetPlugin,
    log::LogPlugin,
    MinimalPlugins,
};

use bevy_renet::{transport::NetcodeServerPlugin, RenetServerPlugin};
use multiplayer_demo::{PlayerLobby, RecentlyRespawned}; // ⬅️ Ajout ici
use renet::{
    transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    ConnectionConfig, RenetServer,
};

use resources::SpawnSpots;
use systems::{
    handle_events_system, receive_message_system, receive_shoot_system, send_message_system,
    setup_system,
};

const SERVER_ADDR: &str = "0.0.0.0:5000";

mod resources;
mod systems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ServerSystemSet {
    Events,
    Receive,
    Send,
}

fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        LogPlugin::default(),
        RenetServerPlugin,
        NetcodeServerPlugin,
    ));

    let server = RenetServer::new(ConnectionConfig::default());
    app.insert_resource(server);

    app.insert_resource(PlayerLobby(HashMap::default()));
    app.insert_resource(SpawnSpots::new());
    app.insert_resource(RecentlyRespawned::default()); // ⬅️ Ressource ajoutée ici

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

    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Échec de l'initialisation du transport réseau");
    app.insert_resource(transport);

    match local_ip_address::local_ip() {
        Ok(ip) => println!("🌐 IP locale du serveur : {}", ip),
        Err(e) => eprintln!("❌ Erreur IP locale : {}", e),
    }

    app.add_systems(Startup, setup_system);

    let fixed_interval = Duration::from_secs_f32(1.0 / 60.0);

    app.configure_sets(
        Update,
        (
            ServerSystemSet::Events,
            ServerSystemSet::Receive.after(ServerSystemSet::Events),
            ServerSystemSet::Send.after(ServerSystemSet::Receive),
        ),
    );

    app.add_systems(
        Update,
        (
            handle_events_system.in_set(ServerSystemSet::Events),
            receive_message_system.in_set(ServerSystemSet::Receive),
            receive_shoot_system.in_set(ServerSystemSet::Receive),
            send_message_system.in_set(ServerSystemSet::Send),
        )
            .into_configs()
            .run_if(on_timer(fixed_interval)),
    );

    app.run();
}
