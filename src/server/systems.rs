// src/server/system.rs

// Import des modules standard pour la gestion des collections et du temps
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

// Import des modules Bevy pour l'ECS et les événements
use bevy::{
    ecs::{
        event::EventReader,
        system::{Res, ResMut},
    },
    log::info,
    math::{Quat, Vec3},
};
// Import des structures de données partagées
use multiplayer_demo::{
    PlayerAttributes, PlayerLobby, PlayerShoot, RecentlyRespawned, ServerMessage, Weapon,
};
// Import pour la génération de nombres aléatoires
use rand::{seq::SliceRandom, thread_rng};
// Import des modules renet pour la gestion réseau
use renet::{ClientId, DefaultChannel, RenetServer, ServerEvent};

// Import des modules locaux
use crate::{resources::SpawnSpots, SERVER_ADDR};

/// Système de configuration initiale du serveur
/// Affiche un message de confirmation du démarrage du serveur
pub fn setup_system() {
    info!("Server started on {}", SERVER_ADDR);
}

/// Système d'envoi des messages de synchronisation du lobby
/// Envoie l'état actuel du lobby à tous les clients connectés
/// 
/// # Arguments
/// * `server` - Référence mutable au serveur renet
/// * `player_lobby` - Référence au lobby des joueurs
pub fn send_message_system(mut server: ResMut<RenetServer>, player_lobby: Res<PlayerLobby>) {
    let channel = DefaultChannel::Unreliable; // Canal non fiable pour les mises à jour fréquentes
    let lobby = player_lobby.0.clone(); // Copie du lobby pour l'envoi
    let event = ServerMessage::LobbySync(lobby); // Création du message de synchronisation
    let message = bincode::serialize(&event).unwrap(); // Sérialisation du message
    server.broadcast_message(channel, message); // Envoi à tous les clients
}

/// Système de réception des messages des clients
/// Traite les mises à jour de position et d'état envoyées par les clients
/// 
/// # Arguments
/// * `server` - Référence mutable au serveur renet
/// * `player_lobby` - Référence mutable au lobby des joueurs
/// * `recently_respawned` - Référence mutable à la liste des joueurs récemment respawnés
pub fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut player_lobby: ResMut<PlayerLobby>,
    mut recently_respawned: ResMut<RecentlyRespawned>,
) {
    // Parcours de tous les clients connectés
    for client_id in server.clients_id() {
        // Tentative de réception d'un message du client
        if let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            // Vérification si le client a récemment respawné (pour éviter les glitches de position)
            let skip_position = recently_respawned
                .0
                .get(&client_id)
                .map(|&t| t.elapsed() < Duration::from_millis(500))
                .unwrap_or(false);

            // Recherche du joueur dans le lobby
            if let Some(existing) = player_lobby.0.get_mut(&client_id) {
                // Tentative de désérialisation des attributs du joueur
                if let Ok(player_update) = bincode::deserialize::<PlayerAttributes>(&message) {
                    // Mise à jour des attributs du joueur
                    existing.username = player_update.username;
                    if !skip_position {
                        existing.position = player_update.position; // Mise à jour de la position seulement si pas de respawn récent
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

    // Nettoyage automatique des entrées expirées dans la liste des respawns récents
    // Supprime les entrées de plus de 500ms pour économiser la mémoire
    recently_respawned
        .0
        .retain(|_, &mut t| t.elapsed() < Duration::from_millis(500));
}

/// Système de gestion des événements réseau (connexions/déconnexions)
/// Traite les événements de connexion et déconnexion des clients
/// 
/// # Arguments
/// * `server` - Référence mutable au serveur renet
/// * `server_events` - Lecteur d'événements serveur
/// * `player_lobby` - Référence mutable au lobby des joueurs
/// * `spawn_spots` - Référence aux points de spawn
/// * `recently_respawned` - Référence mutable à la liste des respawns récents
pub fn handle_events_system(
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    mut player_lobby: ResMut<PlayerLobby>,
    spawn_spots: Res<SpawnSpots>,
    mut recently_respawned: ResMut<RecentlyRespawned>,
) {
    // Parcours de tous les événements serveur
    for event in server_events.read() {
        match event {
            // Événement de connexion d'un nouveau client
            ServerEvent::ClientConnected { client_id } => {
                println!("✅ Client {client_id} connected");

                // Création du joueur pour le nouveau client
                spawn_player(
                    &mut server,
                    &mut player_lobby,
                    &spawn_spots,
                    *client_id,
                    &mut recently_respawned,
                );
            }

            // Événement de déconnexion d'un client
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("❌ Client {client_id} disconnected: {reason}");
                // Suppression du joueur du lobby
                despawn_player(&mut server, &mut player_lobby, *client_id);
            }
        }
    }
}

/// Fonction pour créer un nouveau joueur dans le lobby
/// Initialise les attributs du joueur et l'ajoute au lobby
/// 
/// # Arguments
/// * `server` - Référence mutable au serveur renet
/// * `player_lobby` - Référence mutable au lobby des joueurs
/// * `spawn_spots` - Référence aux points de spawn
/// * `client_id` - ID du client à créer
/// * `recently_respawned` - Référence mutable à la liste des respawns récents
fn spawn_player(
    server: &mut RenetServer,
    player_lobby: &mut PlayerLobby,
    spawn_spots: &SpawnSpots,
    client_id: ClientId,
    recently_respawned: &mut ResMut<RecentlyRespawned>,
) {
    let mut rng = thread_rng(); // Générateur de nombres aléatoires
    // Sélection aléatoire d'un point de spawn
    let Some(&spawn) = spawn_spots.0.choose(&mut rng) else {
        eprintln!("⚠️ No spawn spots available for client {client_id}");
        return;
    };

    // Ajout du joueur au lobby avec des attributs par défaut
    player_lobby
        .0
        .insert(client_id, default_player_attributes(spawn));

    // Envoi d'un message de notification de connexion à tous les autres clients
    let message = bincode::serialize(&ServerMessage::PlayerJoin(client_id)).unwrap();
    server.broadcast_message_except(client_id, DefaultChannel::ReliableOrdered, message);

    // Marquage du client comme récemment respawné pour éviter les glitches
    recently_respawned.0.insert(client_id, Instant::now());
}

/// Fonction pour supprimer un joueur du lobby
/// Retire le joueur du lobby et notifie les autres clients
/// 
/// # Arguments
/// * `server` - Référence mutable au serveur renet
/// * `player_lobby` - Référence mutable au lobby des joueurs
/// * `client_id` - ID du client à supprimer
fn despawn_player(server: &mut RenetServer, player_lobby: &mut PlayerLobby, client_id: ClientId) {
    // Suppression du joueur du lobby
    player_lobby.0.remove(&client_id);

    // Envoi d'un message de notification de déconnexion à tous les clients
    let message = bincode::serialize(&ServerMessage::PlayerLeave(client_id)).unwrap();
    server.broadcast_message(DefaultChannel::ReliableOrdered, message);
}

/// Fonction pour créer des attributs par défaut pour un nouveau joueur
/// Initialise un joueur avec des valeurs de base (santé, armes, munitions, etc.)
/// 
/// # Arguments
/// * `spawn` - Position de spawn du joueur
/// 
/// # Returns
/// * `PlayerAttributes` - Attributs par défaut du joueur
fn default_player_attributes(spawn: (f32, f32, f32)) -> PlayerAttributes {
    PlayerAttributes {
        username: "".to_string(), // Nom d'utilisateur vide par défaut
        position: spawn, // Position de spawn fournie
        rotation: Quat::IDENTITY, // Rotation neutre
        health: 100., // Santé maximale
        armor: 0., // Pas d'armure par défaut
        velocity: Vec3::ZERO, // Vélocité nulle
        // Initialisation des armes possédées (seulement le pistolet par défaut)
        owned_weapon: HashMap::from([
            (Weapon::Gun, true),
            (Weapon::Shotgun, false),
            (Weapon::Gatling, false),
            (Weapon::RocketLauncher, false),
            (Weapon::Bfg, false),
        ]),
        actual_weapon: Weapon::Gun, // Arme actuellement équipée
        // Initialisation des munitions pour chaque arme
        ammo: HashMap::from([
            // 100 bullets pour une seule arme sur 5 jouables
            (Weapon::Gun, 999.),
            // (Weapon::Gun, 30.),
            (Weapon::Shotgun, 15.),
            (Weapon::Gatling, 50.),
            (Weapon::RocketLauncher, 5.),
            (Weapon::Bfg, 1.),
        ]),
        entities: HashMap::new(), // Pas d'entités 3D par défaut
    }
}

/// Système de traitement des tirs des clients
/// Vérifie les collisions entre les tirs et les joueurs, applique les dégâts
/// 
/// # Arguments
/// * `server` - Référence mutable au serveur renet
/// * `player_lobby` - Référence mutable au lobby des joueurs
/// * `spawn_spots` - Référence aux points de spawn
/// * `recently_respawned` - Référence mutable à la liste des respawns récents
pub fn receive_shoot_system(
    mut server: ResMut<RenetServer>,
    mut player_lobby: ResMut<PlayerLobby>,
    spawn_spots: Res<SpawnSpots>,
    mut recently_respawned: ResMut<RecentlyRespawned>,
) {
    let mut should_broadcast_lobby = false; // Flag pour indiquer si le lobby doit être synchronisé
    let mut hits_to_apply = vec![]; // Liste des impacts à traiter

    // Parcours de tous les clients pour vérifier leurs tirs
    for client_id in server.clients_id() {
        // Traitement de tous les messages de tir du client
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
        {
            // Tentative de désérialisation du message de tir
            if let Ok(shoot) = bincode::deserialize::<PlayerShoot>(&message) {
                // Vérification de collision avec tous les autres joueurs
                for (victim_id, victim_attr) in player_lobby.0.iter() {
                    if *victim_id != client_id { // Pas de tir sur soi-même
                        let victim_pos = Vec3::from_array(victim_attr.position.into());
                        let dir = Vec3::from_array(shoot.to.into()) - Vec3::from_array(shoot.from.into());
                        // Vérification si le tir touche le joueur
                        let hit = ray_hits_player(victim_pos, shoot.from.into(), dir);

                        if hit {
                            println!("💥 Client {client_id} a touché {victim_id}");

                            // Calcul des dégâts selon l'arme utilisée
                            let damage = match shoot.weapon {
                                Weapon::Gun => 17.,
                                Weapon::Shotgun => 28.,
                                Weapon::Gatling => 8.,
                                Weapon::RocketLauncher => 400.,
                                Weapon::Bfg => 800.,
                            };

                            // Ajout de l'impact à la liste de traitement
                            hits_to_apply.push((client_id, *victim_id, damage));
                        }
                    }
                }
            }
        }
    }

    // Application des dégâts et gestion des morts
    for (shooter_id, victim_id, damage) in hits_to_apply {
        if let Some(victim_attr) = player_lobby.0.get_mut(&victim_id) {
            // Application des dégâts
            victim_attr.health -= damage;

            // Envoi du message de dégâts au joueur touché
            let hit_msg = ServerMessage::PlayerHit {
                new_health: victim_attr.health,
                client_id: victim_id,
            };
            let msg = bincode::serialize(&hit_msg).unwrap();
            server.send_message(victim_id, DefaultChannel::ReliableOrdered, msg);

            // Vérification si le joueur est mort
            if victim_attr.health <= 0.0 {
                if let Some(attr) = player_lobby.0.get_mut(&shooter_id) {
                    if let Some(ammo) = attr.ammo.get_mut(&attr.actual_weapon) {
                        // Ajout de munitions selon l'arme utilisée après un frag
                        let add_ammo = match attr.actual_weapon {
                            Weapon::Gun => 15.,
                            Weapon::Shotgun => 6.,
                            Weapon::Gatling => 30.,
                            Weapon::RocketLauncher => 1.,
                            Weapon::Bfg => 1.,
                        };
                        *ammo += add_ammo;
                        
                    }
                }
                // Suppression du joueur mort
                despawn_player(&mut server, &mut player_lobby, victim_id);
                // Respawn du joueur
                spawn_player(
                    &mut server,
                    &mut player_lobby,
                    &spawn_spots,
                    victim_id,
                    &mut recently_respawned,
                );
                // Envoi du message de mort au joueur
                if let Some(victim_attr) = player_lobby.0.get_mut(&victim_id) {
                    let death_msg = ServerMessage::PlayerDeath {
                        dead: victim_id,
                        attr: victim_attr.clone(),
                    };
                    let death_msg_bytes = bincode::serialize(&death_msg).unwrap();
                    server.send_message(
                        victim_id,
                        DefaultChannel::ReliableOrdered,
                        death_msg_bytes.clone(),
                    );
                    println!("server player_lobby : {:?}", player_lobby);
                }
                should_broadcast_lobby = true; // Le lobby doit être synchronisé
            }
        }
    }

    // Synchronisation du lobby si nécessaire
    if should_broadcast_lobby {
        let lobby_msg = ServerMessage::LobbySync(player_lobby.0.clone());
        let msg = bincode::serialize(&lobby_msg).unwrap();
        server.broadcast_message(DefaultChannel::Unreliable, msg);
    }
}

/// Fonction pour vérifier si un rayon touche un joueur
/// Utilise une détection de collision basée sur la distance au rayon
/// 
/// # Arguments
/// * `pos` - Position du joueur
/// * `from` - Point de départ du tir
/// * `dir` - Direction du tir
/// 
/// # Returns
/// * `bool` - True si le joueur est touché, False sinon
fn ray_hits_player(pos: Vec3, from: [f32; 3], dir: Vec3) -> bool {
    let start = Vec3::from_array(from);
    let max_dist = 100.0; // Distance maximale du tir
    let radius = 1.0; // Rayon de collision du joueur

    let to = start + dir.normalize() * max_dist; // Point de fin du rayon
    let closest = closest_point_on_line(start, to, pos); // Point le plus proche sur le rayon
    pos.distance(closest) < radius // Vérification de la collision
}

/// Fonction pour trouver le point le plus proche sur une ligne
/// Calcule la projection orthogonale d'un point sur une ligne
/// 
/// # Arguments
/// * `a` - Point de départ de la ligne
/// * `b` - Point de fin de la ligne
/// * `p` - Point à projeter
/// 
/// # Returns
/// * `Vec3` - Point le plus proche sur la ligne
fn closest_point_on_line(a: Vec3, b: Vec3, p: Vec3) -> Vec3 {
    let ab = b - a; // Vecteur directeur de la ligne
    let t = ((p - a).dot(ab)) / ab.length_squared(); // Paramètre de projection
    a + ab * t.clamp(0.0, 1.0) // Point projeté (clampé entre a et b)
}
