# Multiplayer FPS Game

## Description du projet

Ce projet est un jeu de tir à la première personne (FPS) multijoueur développé en Rust avec le moteur de jeu Bevy. Il permet à plusieurs joueurs de se connecter à un serveur et de s'affronter dans un environnement 3D.

## Architecture du projet

Le projet est divisé en deux parties principales :
- **Serveur** : Gère la logique de jeu, les connexions clients, et la synchronisation des données
- **Client** : Interface utilisateur, rendu 3D, et gestion des entrées utilisateur

## Fonctionnalités principales

- **Multijoueur en temps réel** : Support de plusieurs joueurs simultanés
- **Système d'armes** : Plusieurs types d'armes avec des caractéristiques différentes
- **Physique 3D** : Collisions, gravité, et mouvement réaliste
- **Système de santé et d'armure** : Gestion des dégâts et de la mort
- **Réseau optimisé** : Communication client/serveur via UDP avec renet

## Installation et configuration

### Prérequis

- Rust (version 2021 ou plus récente)
- Git LFS pour la gestion des assets volumineux

### Configuration de Git LFS avant de récupérer le répo

```bash
# Installation de Git LFS
sudo apt install git-lfs

# Initialisation de Git LFS
git lfs install

# Pour gérer les fichiers lourds sur git
git lfs track "fichiers lourds"

# Pour tracker les fichiers lourds
git add .gitattributes
git commit -m "Configuration Git LFS"
```

### Compilation et exécution après la récupération du répo

```bash
# Installation des recquis et compilation du projet
./install.sh

# Lancement du serveur
# L'adresse ip du server servira pour les clients
./server.sh


# Lancement du client (dans un autre terminal)
# L'adresse ip du server sera demandé puis vous devrez entrer un nom d'utilisateur
./client.sh
```

## Structure du code

### Serveur (`src/server/`)
- `main.rs` : Point d'entrée du serveur
- `resources.rs` : Ressources partagées (points de spawn, etc.)
- `systems.rs` : Systèmes de logique serveur (gestion des connexions, tirs, etc.)

### Client (`src/client/`)
- `main.rs` : Point d'entrée du client
- `resources.rs` : Ressources locales du client
- `systems.rs` : Systèmes de gestion réseau et synchronisation
- `events.rs` : Événements personnalisés
- `game/` : Module principal du jeu (rendu, input, logique de jeu)

### Bibliothèque partagée (`src/lib.rs`)
- Structures de données communes entre client et serveur
- Messages réseau
- Types d'armes et attributs de joueur

## Technologies utilisées

- **Bevy** : Moteur de jeu ECS (Entity Component System)
- **Renet** : Bibliothèque réseau pour les jeux multijoueurs
- **Rapier3D** : Moteur de physique 3D
- **Serde** : Sérialisation/désérialisation des données
- **Bincode** : Sérialisation binaire pour les messages réseau

## Développement

Le projet utilise une architecture modulaire avec :
- Séparation claire entre client et serveur
- Système d'événements pour la communication entre systèmes
- Ressources partagées pour la gestion d'état
- Plugins Bevy pour l'organisation du code


## Licence

Ce projet est sous licence libre. Voir le fichier LICENSE pour plus de détails.