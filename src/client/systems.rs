// src/client/system.rs

// Import des modules Bevy pour l'ECS, les événements et le rendu
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
// Import du module de physique pour les collisions
use bevy_rapier3d::prelude::Collider;
// Import des structures de données partagées
use multiplayer_demo::{PlayerAttributes, PlayerEntity, PlayerLobby, PlayerStats, ServerMessage};
// Import des modules renet pour la gestion réseau
use renet::{ClientId, DefaultChannel, RenetClient};

// Import des modules locaux
use crate::{
    events::{LobbySyncEvent, PlayerDespawnEvent, PlayerSpawnEvent},
    game::player::player_shooting::Shootable,
    resources::{IsSynced, MyUsername},
    MyClientId,
};

/// Système d'envoi des messages au serveur
/// Envoie les informations du joueur local (position, rotation, état) au serveur
///
/// # Arguments
/// * `client` - Référence mutable au client renet
/// * `query` - Requête pour récupérer les attributs et la transformation du joueur local
/// * `username` - Référence au nom d'utilisateur local
pub fn send_message_system(
    mut client: ResMut<RenetClient>,
    query: Query<(&PlayerAttributes, &Transform)>,
    username: Res<MyUsername>,
) {
    // Récupération des données du joueur local
    if let Ok((player, transform)) = query.get_single() {
        // Construction du message de synchronisation
        let player_sync = PlayerAttributes {
            username: username.0.clone(), // Nom d'utilisateur local
            rotation: player.rotation,    // Rotation du joueur
            position: [
                transform.translation.x,
                transform.translation.y - 0.7, // Ajustement de la hauteur (centre du joueur)
                transform.translation.z,
            ]
            .into(),
            health: player.health,                     // Points de vie
            armor: player.armor,                       // Points d'armure
            velocity: player.velocity,                 // Vélocité
            owned_weapon: player.owned_weapon.clone(), // Armes possédées
            actual_weapon: player.actual_weapon,       // Arme actuellement équipée
            ammo: player.ammo.clone(),                 // Munitions
            entities: player.entities.clone(),         // Entités 3D des armes
        };

        // Sérialisation et envoi du message
        let message = bincode::serialize(&player_sync).unwrap();
        client.send_message(DefaultChannel::Unreliable, message);
    }
}

/// Système de réception des messages du serveur
/// Traite tous les messages reçus du serveur et déclenche les événements appropriés
///
/// # Arguments
/// * `client` - Référence mutable au client renet
/// * `spawn_events` - Écrivain d'événements de spawn de joueurs
/// * `despawn_events` - Écrivain d'événements de despawn de joueurs
/// * `lobby_sync_events` - Écrivain d'événements de synchronisation du lobby
/// * `sync_state` - Référence mutable à l'état de synchronisation
/// * `my_id` - Référence à l'ID du client local
/// * `player_query` - Requête pour modifier les attributs et la transformation du joueur local
pub fn receive_message_system(
    mut client: ResMut<RenetClient>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut despawn_events: EventWriter<PlayerDespawnEvent>,
    mut lobby_sync_events: EventWriter<LobbySyncEvent>,
    mut sync_state: ResMut<SyncState>,
    my_id: Res<MyClientId>,
    mut player_query: Query<(&mut PlayerAttributes, &mut Transform)>,
) {
    // Traitement des messages fiables (canal ReliableOrdered)
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        if let Ok(server_message) = bincode::deserialize::<ServerMessage>(&message) {
            match server_message {
                // Message de connexion d'un nouveau joueur
                ServerMessage::PlayerJoin(client_id) => {
                    info!("Client connected: {}", client_id);
                    // Mise à jour de l'état de synchronisation si c'est la première connexion
                    if !sync_state.is_connected {
                        sync_state.is_connected = true;
                        sync_state.client_id = Some(client_id);
                        info!("Client ID enregistré dans SyncState: {:?}", client_id);
                    }
                    // Déclenchement de l'événement de spawn
                    spawn_events.send(PlayerSpawnEvent(client_id));
                }
                // Message de déconnexion d'un joueur
                ServerMessage::PlayerLeave(client_id) => {
                    info!("Client disconnected: {}", client_id);
                    // Déclenchement de l'événement de despawn
                    despawn_events.send(PlayerDespawnEvent(client_id));
                }
                // Message de synchronisation du lobby
                ServerMessage::LobbySync(map) => {
                    lobby_sync_events.send(LobbySyncEvent(map));
                }
                // Message de dégâts reçus par le joueur local
                ServerMessage::PlayerHit {
                    new_health,
                    client_id,
                } => {
                    // Vérification que c'est bien le joueur local qui a été touché
                    if Some(client_id) == sync_state.client_id {
                        if let Ok((mut player, _)) = player_query.get_single_mut() {
                            player.health = new_health; // Mise à jour de la santé
                            info!("🔥 Dégât reçu ! Nouvelle vie : {}", new_health);
                        }
                    }
                }
                // Message de mort d'un joueur
                ServerMessage::PlayerDeath {
                    dead: client_id,
                    attr:p_attr,
                } => {
                    // Si c'est le joueur local qui est mort, mise à jour de sa position
                    if my_id.0 == client_id {
                        if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
                            *player = p_attr;
                            let over_the_ground = 3.; // Hauteur au-dessus du sol
                            transform.translation = Vec3::new(
                                player.position.0,
                                player.position.1 + over_the_ground,
                                player.position.2
                            );
                        }
                    }

                    info!("Mort reçue pour client {}", client_id);
                    // Déclenchement de l'événement de despawn
                    despawn_events.send(PlayerDespawnEvent(client_id));
                }
            }
        }
    }

    // Traitement des messages non fiables (canal Unreliable)
    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        if let Ok(ServerMessage::LobbySync(map)) = bincode::deserialize(&message) {
            lobby_sync_events.send(LobbySyncEvent(map));
        }
    }
}

/// Système de mise à jour du lobby local
/// Met à jour le lobby avec les données reçues du serveur
///
/// # Arguments
/// * `lobby` - Référence mutable au lobby des joueurs
/// * `lobby_sync_events` - Lecteur d'événements de synchronisation du lobby
pub fn update_lobby_system(
    mut lobby: ResMut<PlayerLobby>,
    mut lobby_sync_events: EventReader<LobbySyncEvent>,
) {
    for event in lobby_sync_events.read() {
        lobby.0 = event.0.clone(); // Mise à jour complète du lobby
    }
}

/// Système de gestion des événements de spawn de joueurs
/// Crée les entités 3D pour les nouveaux joueurs qui se connectent
///
/// # Arguments
/// * `commands` - Commandes Bevy pour créer des entités
/// * `asset_server` - Référence au serveur d'assets
/// * `spawn_events` - Lecteur d'événements de spawn
pub fn handle_player_spawn_event_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
) {
    for event in spawn_events.read() {
        let client_id = event.0;
        info!("Tentative de spawn du joueur : {:?}", client_id);

        // Création de l'entité joueur avec le modèle 3D et les composants nécessaires
        commands.spawn((
            SceneBundle {
                scene: asset_server.load("models/guntest.glb#Scene0"), // Modèle 3D du joueur
                ..default()
            },
            Collider::cylinder(1.5, 0.5), // Collision cylindrique pour le joueur
            PlayerEntity(client_id),      // Composant d'identification du joueur
            Shootable,                    // Composant pour permettre les tirs
        ));

        info!("✅ Joueur {:?} spawné avec succès", client_id);
    }
}

/// Système de gestion des événements de despawn de joueurs
/// Supprime les entités 3D des joueurs qui se déconnectent
///
/// # Arguments
/// * `commands` - Commandes Bevy pour supprimer des entités
/// * `despawn_events` - Lecteur d'événements de despawn
/// * `query` - Requête pour trouver les entités joueurs
pub fn handle_player_despawn_event_system(
    mut commands: Commands,
    mut despawn_events: EventReader<PlayerDespawnEvent>,
    query: Query<(Entity, &PlayerEntity)>,
) {
    for event in despawn_events.read() {
        info!("Joueur déconnecté :: {:?}", event.0);
        let client_id = event.0;

        // Recherche et suppression de l'entité correspondante
        for (entity, player_entity) in query.iter() {
            if player_entity.0 == client_id {
                commands.entity(entity).despawn(); // Suppression de l'entité
                break;
            }
        }
    }
}

/// Système de gestion des événements de synchronisation du lobby
/// Met à jour les positions et états des joueurs existants et spawn les nouveaux
///
/// # Arguments
/// * `spawn_events` - Écrivain d'événements de spawn
/// * `sync_events` - Lecteur d'événements de synchronisation
/// * `query` - Requête pour modifier les entités joueurs existantes
/// * `my_client_id` - Référence à l'ID du client local
/// * `is_synced` - Référence mutable au flag de synchronisation
pub fn handle_lobby_sync_event_system(
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut sync_events: EventReader<LobbySyncEvent>,
    mut query: Query<(&PlayerEntity, &mut Transform, Option<&mut PlayerStats>)>,
    my_client_id: Res<MyClientId>,
    mut is_synced: ResMut<IsSynced>,
) {
    if let Some(event) = sync_events.read().last() {
        // Parcours de tous les joueurs dans le lobby
        for (client_id, player_sync) in event.0.iter() {
            let mut found = false;

            // Mise à jour des joueurs existants
            for (player_entity, mut transform, stats_opt) in query.iter_mut() {
                if *client_id == player_entity.0 {
                    // Mise à jour de la position et rotation
                    transform.translation = player_sync.position.into();
                    transform.rotation = player_sync.rotation.into();

                    // Mise à jour des statistiques si disponibles
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

            // Spawn des nouveaux joueurs (sauf le joueur local)
            if !found && *client_id != my_client_id.0 {
                spawn_events.send(PlayerSpawnEvent(*client_id));
            }
        }

        // Marquage de la synchronisation comme terminée
        is_synced.0 = true;
    }
}

/// Ressource pour gérer l'état de synchronisation du client
/// Suit l'état de connexion et de synchronisation avec le serveur
#[derive(Resource, Default, Debug)]
pub struct SyncState {
    pub is_connected: bool,          // Si le client est connecté au serveur
    pub is_synced: bool,             // Si le lobby est synchronisé
    pub client_id: Option<ClientId>, // ID du client local (une fois assigné)
}

/// Plugin pour gérer l'état de synchronisation
/// Initialise la ressource SyncState et ajoute le système de vérification
pub struct SyncStatePlugin;

impl Plugin for SyncStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SyncState>()
            .add_systems(Update, check_lobby_sync_system);
    }
}

/// Système de vérification de la synchronisation du lobby
/// Vérifie si le client local est présent dans le lobby et déclenche le spawn si nécessaire
///
/// # Arguments
/// * `sync_state` - Référence mutable à l'état de synchronisation
/// * `lobby` - Référence au lobby des joueurs
/// * `spawn_events` - Écrivain d'événements de spawn
fn check_lobby_sync_system(
    mut sync_state: ResMut<SyncState>,
    lobby: Res<PlayerLobby>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
) {
    // Vérification si le client est connecté mais pas encore synchronisé
    if sync_state.is_connected && !sync_state.is_synced {
        if let Some(client_id) = sync_state.client_id {
            // Vérification si le client local est présent dans le lobby
            if lobby.0.contains_key(&client_id) {
                sync_state.is_synced = true;
                info!("Lobby synchronisé, client présent dans PlayerLobby");

                // Déclenchement du spawn du joueur local
                spawn_events.send(PlayerSpawnEvent(client_id));
            }
        }
    }
}
