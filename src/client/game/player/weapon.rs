use bevy::prelude::*;

use bevy::input::mouse::MouseWheel;
use bevy::input::mouse::MouseScrollUnit;
use multiplayer_demo::PlayerAttributes;
use multiplayer_demo::Weapon;

pub fn switch_weapon(
    mut player_query: Query<&mut PlayerAttributes>,
    mut scroll_events: EventReader<MouseWheel>, // Pour détecter les événements de la molette
    query_visibility: Query<&mut Visibility, With<Handle<Scene>>>,
) {
    if let Ok(mut player) = player_query.get_single_mut() {
        let mut prev = false;

        // On itère sur les événements de la molette
        let _value = update_weapon_visibility(&player, query_visibility);
        for event in scroll_events.read() {
            // Vérifie si la molette a été défilée vers le haut ou vers le bas
            match event.unit {
                MouseScrollUnit::Line | MouseScrollUnit::Pixel => {
                    if event.y > 0.0 {
                        // Molette défilée vers le bas (changer vers l'arme précédente)
                        prev = true;
                    } else if event.y < 0.0 {
                        // Molette défilée vers le haut (changer vers l'arme suivante)
                        prev = false;
                    }

                    if let Some(new_weapon) = get_next_weapon(&player, prev) {
                        // Mise à jour de l'arme actuelle du joueur
                        player.actual_weapon = new_weapon;
                    }
                }
            }
        }
    }
}

// Fonction qui cherche la prochaine arme disponible
pub fn get_next_weapon(player: &PlayerAttributes, prev: bool) -> Option<Weapon> {
    let mut all_weapons = vec![
        Weapon::Gun,
        Weapon::Shotgun,
        Weapon::Gatling,
        Weapon::RocketLauncher,
        Weapon::Bfg,
    ];

    if prev {
        all_weapons = vec![
            Weapon::Bfg,
            Weapon::RocketLauncher,
            Weapon::Gatling,
            Weapon::Shotgun,
            Weapon::Gun,
        ];
    }

    let current_weapon_index = all_weapons
        .iter()
        .position(|weapon| *weapon == player.actual_weapon)
        .unwrap_or(0);

    // Cherche l'arme suivante qui est possédée
    for i in 1..all_weapons.len() {
        let next_index = (current_weapon_index + i) % all_weapons.len();
        if let Some(owned) = player.owned_weapon.get(&all_weapons[next_index]) {
            if *owned {
                return Some(all_weapons[next_index]);
            }
        }
    }
    None
}

// Fonction pour mettre à jour la visibilité des armes du joueur
pub fn update_weapon_visibility(
    player: &PlayerAttributes,
    mut query_visibility: Query<&mut Visibility, With<Handle<Scene>>>,
) {
    // Parcourt toutes les armes du joueur
    for (&weapon, &entity) in &player.entities {
        if let Ok(mut visibility) = query_visibility.get_mut(entity) {
            // Si c'est l'arme actuelle, elle doit être visible
            if weapon == player.actual_weapon {
                *visibility = Visibility::Visible;
            } else {
                // Sinon, elle doit être cachée
                *visibility = Visibility::Hidden;
            }
        }
    }
}
