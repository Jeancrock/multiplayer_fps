// src/client/main

// Import des modules standard pour l'entrée/sortie et le réseau
use std::{
    io::{self, Write},
    net::{SocketAddrV4, UdpSocket},
    time::SystemTime,
};

// Import des modules Bevy pour l'ECS et le rendu
use bevy::{
    app::{App, Update},
    log::info,
    prelude::*,
    DefaultPlugins,
};
// Import des plugins renet pour la gestion réseau côté client
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};
// Import du plugin de jeu principal
use game::game::GamePlugin;
// Import des structures de données partagées
use multiplayer_demo::PlayerLobby;
// Import des modules renet pour la configuration réseau
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport},
    ClientId, ConnectionConfig, RenetClient,
};
// Import des ressources locales
use resources::SpawnedPlayers;
// Import des systèmes et plugins locaux
use systems::{update_lobby_system, SyncStatePlugin};

// Import des modules internes du client
use crate::{
    resources::{IsSynced, MyClientId, MyUsername},
    systems::{
        handle_lobby_sync_event_system, handle_player_despawn_event_system,
        handle_player_spawn_event_system, receive_message_system, send_message_system,
    },
};

// Déclarations des sous-modules du client
mod components;  // Composants spécifiques au client
mod events;      // Événements personnalisés
pub mod game;    // Module principal du jeu (rendu, input, etc.)
mod resources;   // Ressources locales du client
mod systems;     // Systèmes de gestion réseau et logique

/// Point d'entrée principal du client
/// Initialise la connexion réseau et démarre l'application de jeu
fn main() {
    // --- Configuration de l'adresse IP du serveur ---
    print!("Entrez l'IP du serveur (ex: 192.168.1.10): ");
    io::stdout().flush().unwrap(); // Force l'affichage du prompt
    let mut ipaddr = String::new();
    io::stdin()
        .read_line(&mut ipaddr)
        .expect("Échec lecture IP");
    let ipaddr = ipaddr.trim(); // Suppression des espaces et retours à la ligne

    // --- Configuration du nom d'utilisateur ---
    let mut username = "".to_string();
    while username.trim().is_empty() {
        print!("Entrez votre nom d'utilisateur (ex: NoobMaster69): ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut username)
            .expect("Échec lecture nom");
        username = username.trim().to_string();
    }

    // --- Initialisation de l'application Bevy ---
    let mut app = App::new();

    // --- Configuration réseau du client ---
    let client_id = rand::random::<u64>(); // Génération d'un ID client aléatoire
    // Création et liaison du socket UDP (port 0 = port automatique)
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Échec du bind UDP client");

    // Création de l'adresse du serveur avec le port 5000
    let server_socket = SocketAddrV4::new(ipaddr.parse().expect("Adresse IP invalide"), 5000);

    // Configuration de l'authentification client (non sécurisée pour le développement)
    let authentication = ClientAuthentication::Unsecure {
        server_addr: server_socket.into(), // Adresse du serveur
        client_id,                         // ID unique du client
        user_data: None,                   // Pas de données utilisateur supplémentaires
        protocol_id: 0,                    // ID du protocole réseau
    };

    // Récupération du timestamp actuel pour la synchronisation
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    // Création du transport réseau client
    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .expect("Échec init transport client!");

    // --- Ajout des plugins réseau à l'application ---
    app.add_plugins(RenetClientPlugin);      // Plugin client renet
    app.add_plugins(NetcodeClientPlugin);    // Plugin de transport réseau

    // --- Insertion des ressources réseau et locales ---
    app.insert_resource(RenetClient::new(ConnectionConfig::default())); // Client renet
    app.insert_resource(transport);                                      // Transport réseau
    app.insert_resource(PlayerLobby::default());                        // Lobby des joueurs
    app.insert_resource(MyClientId(ClientId::from_raw(client_id)));     // ID du client local
    app.insert_resource(MyUsername::new(username.clone()));             // Nom d'utilisateur
    app.insert_resource(SpawnedPlayers::default());                     // Joueurs spawnés localement
    app.insert_resource(IsSynced(false)); // Flag de synchronisation (bloque les messages tant que non synchronisé)

    // --- Ajout des plugins de jeu et de rendu ---
    app.add_plugins((
        SyncStatePlugin, // Plugin de gestion de l'état de synchronisation
        GamePlugin,      // Plugin principal du jeu (rendu, input, logique)
        DefaultPlugins.set(AssetPlugin {
            mode: AssetMode::Unprocessed, // Mode de traitement des assets (non traité pour les performances)
            ..default()
        }),
    ));

    // --- Définition des événements personnalisés ---
    // Ces événements permettent la communication entre les systèmes
    app.add_event::<events::PlayerSpawnEvent>();    // Événement de spawn d'un joueur
    app.add_event::<events::PlayerDespawnEvent>();  // Événement de despawn d'un joueur
    app.add_event::<events::LobbySyncEvent>();      // Événement de synchronisation du lobby

    // --- Ajout des systèmes clients principaux ---
    // Ces systèmes gèrent la logique réseau et la synchronisation
    app.add_systems(Update, receive_message_system);                    // Réception des messages serveur
    app.add_systems(Update, handle_player_spawn_event_system);          // Gestion des spawns de joueurs
    app.add_systems(Update, handle_player_despawn_event_system);        // Gestion des despawns de joueurs
    app.add_systems(Update, handle_lobby_sync_event_system);            // Gestion de la synchronisation du lobby
    app.add_systems(Update, update_lobby_system);                       // Mise à jour du lobby local
    app.add_systems(
        Update,
        send_message_system.run_if(|synced: Res<IsSynced>| synced.0), // Envoi des messages (seulement si synchronisé)
    );

    // --- Affichage du message de confirmation ---
    info!("Client {} started with username '{}'", client_id, username);

    // --- Démarrage de la boucle principale de Bevy ---
    app.run();
}