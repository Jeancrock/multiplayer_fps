// src/server/main.rs

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::{
    app::{App, Startup, Update},
    asset::AssetPlugin,
    log::LogPlugin,
    MinimalPlugins,
};
use bevy_renet::{transport::NetcodeServerPlugin, RenetServerPlugin};
use multiplayer_demo::PlayerLobby;
use renet::{
    transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    ConnectionConfig, RenetServer,
};
use resources::SpawnSpots;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};
use systems::{handle_events_system, receive_message_system, receive_shoot_system, send_message_system, setup_system};

// Permet au serveur d'écouter sur toutes les interfaces réseau locales
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

    // Plugins minimaux (pas besoin du moteur complet Bevy côté serveur)
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(LogPlugin::default());
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(NetcodeServerPlugin);

    // Initialisation du serveur Renet (logique réseau bas niveau)
    let server = RenetServer::new(ConnectionConfig::default());
    app.insert_resource(server);

    // Initialisation des ressources globales (état des joueurs, spawn points, etc.)
    app.insert_resource(PlayerLobby(HashMap::default()));
    app.insert_resource(SpawnSpots::new());

    // Configuration et binding du transport réseau (UDP)
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

    // Affichage de l'IP locale pour connexion client facile
    match local_ip_address::local_ip() {
        Ok(ip) => println!("🌐 IP locale du serveur : {}", ip),
        Err(e) => eprintln!("❌ Erreur IP locale : {}", e),
    }

    // Système de setup au démarrage
    app.add_systems(Startup, setup_system);

    // Systèmes de jeu à exécuter 60 fois par seconde
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
        handle_events_system
            .run_if(on_timer(fixed_interval))
            .in_set(ServerSystemSet::Events),
    );
    app.add_systems(
        Update,
        receive_message_system
            .run_if(on_timer(fixed_interval))
            .in_set(ServerSystemSet::Receive),
    );
    app.add_systems(
        Update,
        send_message_system
            .run_if(on_timer(fixed_interval))
            .in_set(ServerSystemSet::Send),
    );
    app.add_systems(
        Update,
        receive_shoot_system
            .run_if(on_timer(fixed_interval))
            .in_set(ServerSystemSet::Send),
    );
    app.run();
}
