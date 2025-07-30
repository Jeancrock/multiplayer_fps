// Import des modules standard pour la gestion des collections et du réseau
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

// Import des modules Bevy pour l'ECS et la gestion des systèmes
use bevy::prelude::IntoSystemConfigs;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::{
    app::{App, Startup, Update},
    asset::AssetPlugin,
    log::LogPlugin,
    MinimalPlugins,
};

// Import des modules renet pour la gestion du réseau multijoueur
use bevy_renet::{transport::NetcodeServerPlugin, RenetServerPlugin};
// Import des structures de données partagées entre client et serveur
use multiplayer_demo::{PlayerLobby, RecentlyRespawned};
use renet::{
    transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    ConnectionConfig, RenetServer,
};

// Import des modules locaux du serveur
use resources::SpawnSpots;
use systems::{
    handle_events_system, receive_message_system, receive_shoot_system, send_message_system,
    setup_system,
};

/// Adresse et port sur lesquels le serveur écoute les connexions
/// "0.0.0.0" signifie que le serveur écoute sur toutes les interfaces réseau
const SERVER_ADDR: &str = "0.0.0.0:5000";

// Déclaration des modules locaux
mod resources;
mod systems;

/// Énumération définissant les ensembles de systèmes du serveur
/// Permet d'organiser l'exécution des systèmes dans un ordre spécifique
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ServerSystemSet {
    Events,  // Systèmes de gestion des événements réseau
    Receive, // Systèmes de réception des messages clients
    Send,    // Systèmes d'envoi des messages aux clients
}

/// Point d'entrée principal du serveur
/// Initialise et démarre le serveur de jeu multijoueur
fn main() {
    // Création d'une nouvelle application Bevy
    let mut app = App::new();

    // Ajout des plugins nécessaires au fonctionnement du serveur
    app.add_plugins((
        MinimalPlugins,        // Plugins minimaux de Bevy (pas de rendu)
        AssetPlugin::default(), // Plugin de gestion des assets
        LogPlugin::default(),   // Plugin de logging
        RenetServerPlugin,      // Plugin serveur pour renet
        NetcodeServerPlugin,    // Plugin de transport réseau
    ));

    // Création et insertion du serveur renet avec la configuration par défaut
    let server = RenetServer::new(ConnectionConfig::default());
    app.insert_resource(server);

    // Insertion des ressources globales du serveur
    app.insert_resource(PlayerLobby(HashMap::default())); // Lobby des joueurs connectés
    app.insert_resource(SpawnSpots::new());               // Points de spawn du niveau
    app.insert_resource(RecentlyRespawned::default());    // Gestion des respawns récents

    // Configuration de l'adresse réseau du serveur
    let server_addr = SERVER_ADDR.parse().unwrap();
    
    // Création et liaison du socket UDP pour la communication réseau
    let socket = UdpSocket::bind(server_addr).expect("Échec du bind de l'UDP socket");

    // Configuration du serveur réseau avec les paramètres de connexion
    let server_config = ServerConfig {
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(), // Timestamp actuel pour la synchronisation
        max_clients: 10, // Nombre maximum de clients connectés simultanément
        protocol_id: 0,  // ID du protocole réseau
        public_addresses: vec![server_addr], // Adresses publiques du serveur
        authentication: ServerAuthentication::Unsecure, // Pas d'authentification (développement)
    };

    // Création du transport réseau avec la configuration définie
    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Échec de l'initialisation du transport réseau");
    app.insert_resource(transport);

    // Affichage de l'adresse IP locale pour faciliter les tests
    match local_ip_address::local_ip() {
        Ok(ip) => println!("🌐 IP locale du serveur : {}", ip),
        Err(e) => eprintln!("❌ Erreur IP locale : {}", e),
    }

    // Ajout du système de configuration initiale (exécuté au démarrage)
    app.add_systems(Startup, setup_system);

    // Définition de l'intervalle de mise à jour fixe (60 FPS)
    let fixed_interval = Duration::from_secs_f32(1.0 / 60.0);

    // Configuration de l'ordre d'exécution des ensembles de systèmes
    app.configure_sets(
        Update,
        (
            ServerSystemSet::Events,                    // D'abord les événements
            ServerSystemSet::Receive.after(ServerSystemSet::Events), // Puis la réception
            ServerSystemSet::Send.after(ServerSystemSet::Receive),   // Enfin l'envoi
        ),
    );

    // Ajout des systèmes de mise à jour avec leur ordre d'exécution
    app.add_systems(
        Update,
        (
            // Système de gestion des événements réseau (connexions/déconnexions)
            handle_events_system.in_set(ServerSystemSet::Events),
            // Système de réception des messages des clients
            receive_message_system.in_set(ServerSystemSet::Receive),
            // Système de traitement des tirs des clients
            receive_shoot_system.in_set(ServerSystemSet::Receive),
            // Système d'envoi des messages aux clients
            send_message_system.in_set(ServerSystemSet::Send),
        )
            .into_configs() // Conversion en configurations de systèmes
            .run_if(on_timer(fixed_interval)), // Exécution à intervalle fixe
    );

    // Démarrage de la boucle principale du serveur
    app.run();
}
