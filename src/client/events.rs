// Import des modules nécessaires pour les événements et les types de données
use bevy::ecs::event::Event;
use multiplayer_demo::PlayerAttributes;
use renet::ClientId;

/// Événement déclenché quand un nouveau joueur doit être spawné
/// Contient l'ID du client qui doit être créé dans la scène 3D
/// Utilisé pour synchroniser l'apparition des joueurs entre les clients
#[derive(Event)]
pub struct PlayerSpawnEvent(pub ClientId);

/// Événement déclenché quand un joueur doit être supprimé de la scène
/// Contient l'ID du client qui doit être retiré de la scène 3D
/// Utilisé pour synchroniser la déconnexion des joueurs entre les clients
#[derive(Event)]
pub struct PlayerDespawnEvent(pub ClientId);

/// Événement déclenché quand le lobby des joueurs doit être synchronisé
/// Contient une HashMap complète de tous les joueurs connectés avec leurs attributs
/// Utilisé pour maintenir la cohérence des données entre le serveur et les clients
#[derive(Event)]
pub struct LobbySyncEvent(pub std::collections::HashMap<ClientId, PlayerAttributes>);
