use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PrimaryWindow,
};
use multiplayer_demo::{Player, PlayerLobby, Weapon};
use rand::Rng;

use crate::resources::MyClientId;

#[derive(Resource)]
pub struct HeadUpdateTimer(Timer);

// Composant pour identifier l'élément UI contenant le texte des FPS
#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct GameTimeText;

#[derive(Component)]
pub struct BorderColorEdit;

#[derive(Component)]
pub struct PlayerLifeText;

#[derive(Component)]
pub struct PlayerArmorText;

#[derive(Component)]
pub struct PlayerHead;

#[derive(Component)]
pub struct PlayerHeadBackground;

#[derive(Component)]
pub struct WeaponUi {
    pub weapon: Weapon,
}
#[derive(Component)]
pub struct PlayerAmmoText;

// Système pour configurer l'UI avec le compteur de FPS et les autres éléments
pub fn setup_hud_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();

    // *********************FPS*************************
    commands
        .spawn(TextBundle {
            text: Text::from_section(
                "FPS : ".to_string(),
                TextStyle {
                    font: asset_server.load("fonts/AmazDooMLeft.ttf"),
                    font_size: 60.0,
                    color: Color::Srgba(Srgba::new(0.737, 0.024, 0.012, 0.9)), // Couleur du texte
                },
            ),
            style: Style {
                left: Val::Px(10.),
                top: Val::Px(10.),
                ..default()
            },
            inherited_visibility: InheritedVisibility::VISIBLE,
            ..default()
        })
        .insert(FpsText); // Ajoute le composant FpsText pour identifier le texte

    // **********************Temps de jeu******************************
    commands
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center, // Centrage horizontal du conteneur
                align_items: AlignItems::Center,         // Centrage vertical du conteneur
                position_type: PositionType::Absolute, // Position absolue pour contrôle plus précis
                top: Val::Px(10.),
                width: Val::Percent(100.),
                ..default()
            },
            inherited_visibility: InheritedVisibility::VISIBLE,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        "00.00",
                        TextStyle {
                            font: asset_server.load("fonts/AmazDooMLeft.ttf"),
                            font_size: 60.,
                            color: Color::Srgba(Srgba::new(0.737, 0.024, 0.012, 0.9)), // Couleur du texte
                        },
                    ),
                    style: Style {
                        align_self: AlignSelf::Center, // Centrage du texte à l'intérieur du conteneur
                        ..default()
                    },
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    ..default()
                })
                .insert(GameTimeText); // Ajout du composant GameTimeText pour identifier ce texte
        });

    // ******************Barre du bas************************
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(90.),
                width: Val::Percent(100.), // Conteneur principal qui prend 100% de la largeur de l'écran
                height: Val::Percent(10.), // Hauteur du conteneur
                ..default()
            },
            inherited_visibility: InheritedVisibility::VISIBLE,
            ..default()
        })
        .with_children(|parent| {
            //**************************health************************** */
            parent
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        width: Val::Percent(22.), // Largeur de l'image à 22% de la largeur de l'écran
                        height: Val::Percent(100.), // Hauteur de 100% du conteneur parent
                        ..default()
                    },
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    ..default()
                })
                .insert(UiImage::new(asset_server.load("textures/hud5.png"))) // Charge l'image de fond
                .with_children(|child| {
                    child
                        .spawn(TextBundle {
                            text: Text::from_section(
                                "", // Exemple de valeur des PV
                                TextStyle {
                                    font: asset_server.load("fonts/DooM.ttf"),
                                    font_size: ((window.height() / 10.) * 65.) / 100., // Taille initiale du texte
                                    color: Color::Srgba(Srgba::new(0.737, 0.024, 0.012, 0.9)), // Couleur du texte
                                },
                            ),
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            ..default()
                        })
                        .insert(PlayerLifeText); // Composant pour les PV
                    child.spawn(TextBundle {
                        text: Text::from_section(
                            "HEALTH", // Exemple de valeur des PV
                            TextStyle {
                                font: asset_server.load("fonts/DooM.ttf"),
                                font_size: ((window.height() / 10.) * 30.) / 100., // Taille initiale du texte
                                color: Color::Srgba(Srgba::new(1., 1., 1., 0.5)), // Couleur du texte
                            },
                        ),
                        inherited_visibility: InheritedVisibility::VISIBLE,
                        ..default()
                    });
                });

            //*********************************ARMOR***************************
            parent
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        width: Val::Percent(22.), // Largeur de l'image à 22% de la largeur de l'écran
                        height: Val::Percent(100.), // Hauteur de 100% du conteneur parent
                        ..default()
                    },
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    ..default()
                })
                .insert(UiImage::new(asset_server.load("textures/hud5.png"))) // Charge l'image de fond
                .with_children(|child| {
                    child
                        .spawn(TextBundle {
                            text: Text::from_section(
                                "", // Exemple de valeur d'armure
                                TextStyle {
                                    font: asset_server.load("fonts/DooM.ttf"),
                                    font_size: ((window.height() / 10.) * 65.) / 100., // Taille initiale du texte
                                    color: Color::Srgba(Srgba::new(0.737, 0.024, 0.012, 0.9)), // Couleur du texte
                                },
                            ),
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            ..default()
                        })
                        .insert(PlayerArmorText); // Composant pour les PV
                    child.spawn(TextBundle {
                        text: Text::from_section(
                            "ARMOR", // Exemple de valeur des PV
                            TextStyle {
                                font: asset_server.load("fonts/DooM.ttf"),
                                font_size: ((window.height() / 10.) * 30.) / 100., // Taille initiale du texte
                                color: Color::Srgba(Srgba::new(1., 1., 1., 0.5)), // Couleur du texte
                            },
                        ),
                        inherited_visibility: InheritedVisibility::VISIBLE,
                        ..default()
                    });
                });

            // ********************* HEAD ***********************
            parent
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center, // Centre horizontalement et verticalement
                        width: Val::Percent(12.),                // Largeur relative
                        height: Val::Percent(100.),              // Hauteur du conteneur parent
                        ..default()
                    },
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    background_color: BackgroundColor(Color::BLACK), // Applique un fond noir
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(ImageBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                width: Val::Px(100.), // Largeur fixe
                                height: Val::Auto,    // Ajustement automatique de la hauteur
                                ..default()
                            },
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            image: UiImage::new(asset_server.load("textures/100head1.png")), // Charge l'image
                            ..default()
                        })
                        .insert(PlayerHeadBackground);
                    parent
                        .spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(100.), // Largeur fixe
                                height: Val::Auto,    // Ajustement automatique de la hauteur
                                ..default()
                            },
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            image: UiImage::default(), // Charge l'image
                            ..default()
                        })
                        .insert(PlayerHead);
                });
            // *************** WEAPONS *********************************
            parent
                .spawn(NodeBundle {
                    style: Style {
                        padding: UiRect {
                            left: Val::Percent(1.),
                            right: Val::Percent(1.),
                            top: Val::Percent(1.),
                            bottom: Val::Percent(1.),
                        },
                        display: Display::Flex,
                        flex_wrap: FlexWrap::Wrap, // Permet de revenir à la ligne lorsque la largeur est dépassée
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center, // Centre horizontalement et verticalement
                        width: Val::Percent(22.), // Largeur de l'image à 22% de la largeur de l'écran
                        height: Val::Percent(100.), // Hauteur de 100% du conteneur parent
                        ..default()
                    },
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    ..default()
                })
                .insert(UiImage::new(asset_server.load("textures/hud5.png"))) // Charge l'image de fond
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                border: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                margin: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                width: Val::Percent(30.),
                                height: Val::Percent(45.),
                                ..default()
                            },
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            ..default()
                        })
                        .insert(UiImage::solid_color(Color::NONE)) // Charge l'image de fond
                        .insert(BorderColorEdit) // Ajout du composant GameTimeText pour identifier ce texte
                        .insert(WeaponUi {
                            weapon: Weapon::Gun,
                        });

                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                border: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                margin: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                width: Val::Percent(30.),
                                height: Val::Percent(45.),
                                ..default()
                            },
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            ..default()
                        })
                        .insert(UiImage::solid_color(Color::srgba(0., 0., 0., 0.))) // Charge l'image de fond
                        .insert(UiImage::solid_color(Color::NONE)) // Charge l'image de fond
                        .insert(BorderColorEdit)
                        .insert(WeaponUi {
                            weapon: Weapon::Shotgun,
                        }); // Ajout du composant GameTimeText pour identifier ce texte

                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                border: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                margin: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                width: Val::Percent(30.),
                                height: Val::Percent(45.),
                                ..default()
                            },
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            ..default()
                        })
                        .insert(UiImage::solid_color(Color::NONE)) // Charge l'image de fond
                        .insert(BorderColorEdit)
                        .insert(WeaponUi {
                            weapon: Weapon::Gatling,
                        });

                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                border: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                margin: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                width: Val::Percent(30.),
                                height: Val::Percent(45.),
                                ..default()
                            },
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            ..default()
                        })
                        .insert(UiImage::solid_color(Color::NONE)) // Charge l'image de fond
                        .insert(BorderColorEdit)
                        .insert(WeaponUi {
                            weapon: Weapon::RocketLauncher,
                        });
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                border: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                margin: UiRect {
                                    left: Val::Percent(1.),
                                    right: Val::Percent(1.),
                                    top: Val::Percent(1.),
                                    bottom: Val::Percent(1.),
                                },
                                width: Val::Percent(30.),
                                height: Val::Percent(45.),
                                ..default()
                            },
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            ..default()
                        })
                        .insert(UiImage::solid_color(Color::NONE)) // Charge l'image de fond
                        .insert(BorderColorEdit)
                        .insert(WeaponUi {
                            weapon: Weapon::Bfg,
                        });
                });

            // **************************AMMO***************************
            parent
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        width: Val::Percent(22.), // Largeur de l'image à 22% de la largeur de l'écran
                        height: Val::Percent(100.), // Hauteur de 100% du conteneur parent
                        ..default()
                    },
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    ..default()
                })
                .insert(UiImage::new(asset_server.load("textures/hud5.png"))) // Charge l'image de fond
                .with_children(|child| {
                    child
                        .spawn(TextBundle {
                            text: Text::from_section(
                                "", // Exemple de valeur d'armure
                                TextStyle {
                                    font: asset_server.load("fonts/DooM.ttf"),
                                    font_size: ((window.height() / 10.) * 65.) / 100., // Taille initiale du texte
                                    color: Color::Srgba(Srgba::new(0.737, 0.024, 0.012, 0.9)), // Couleur du texte
                                },
                            ),
                            inherited_visibility: InheritedVisibility::VISIBLE,
                            ..default()
                        })
                        .insert(PlayerAmmoText); // Composant pour les PV
                    child.spawn(TextBundle {
                        text: Text::from_section(
                            "AMMO", // Exemple de valeur des PV
                            TextStyle {
                                font: asset_server.load("fonts/DooM.ttf"),
                                font_size: ((window.height() / 10.) * 30.) / 100., // Taille initiale du texte
                                color: Color::Srgba(Srgba::new(1., 1., 1., 0.5)), // Couleur du texte
                            },
                        ),
                        inherited_visibility: InheritedVisibility::VISIBLE,
                        ..default()
                    });
                });
        });
}

// Système pour mettre à jour le texte des FPS
pub fn update_fps_ui(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average())
    {
        for mut text in query.iter_mut() {
            if !text.sections.is_empty() {
                text.sections[0].value = format!("FPS : {:.1}", fps);
            }
        }
    }
}

// Système pour mettre à jour le texte du temps de jeu
pub fn update_game_time_ui(time: Res<Time>, mut query: Query<&mut Text, With<GameTimeText>>) {
    for mut text in query.iter_mut() {
        let minutes = (time.elapsed_seconds() / 60.) as u32;
        let seconds = (time.elapsed_seconds() % 60.) as u32;
        text.sections[0].value = format!("{:02}:{:02}", minutes, seconds); // "MM:SS"
    }
}

pub fn update_player_health_ui(
    mut text_query: Query<&mut Text, With<PlayerLifeText>>,
    lobby: Res<PlayerLobby>,
    my_id: Res<MyClientId>,
) {
    if let Some(attr) = lobby.0.get(&my_id.0) {
        for mut text in text_query.iter_mut() {
            text.sections[0].value = format!("{:.0}%", attr.health);
        }
    }
}

// Système pour mettre à jour la vie du joueur
pub fn update_player_armor_ui(
    mut text_query: Query<&mut Text, With<PlayerArmorText>>, // Texte de la vie du joueur
    lobby: Res<PlayerLobby>,
    my_id: Res<MyClientId>,
) {
    // Récupère la vie du joueur (on suppose qu'il n'y a qu'un seul joueur)
    if let Some(attr) = lobby.0.get(&my_id.0) {
        // Mise à jour du texte UI avec la vie actuelle du joueur
        for mut text in text_query.iter_mut() {
            text.sections[0].value = format!(
                "{:.0}%", // Affiche la vie sans décimales
                attr.armor
            );
        }
    }
}

pub fn update_player_ammo_ui(
    player_query: Query<&Player>, // Récupère le joueur
    mut text_query: Query<&mut Text, With<PlayerAmmoText>>, // Texte des munitions
) {
    // Récupère le joueur (on suppose qu'il n'y a qu'un seul joueur)
    if let Some(player) = player_query.iter().next() {
        // Accède aux munitions de l'arme actuelle via la HashMap
        if let Some(ammo) = player.ammo.get(&player.actual_weapon) {
            // Mise à jour du texte UI avec les munitions actuelles
            for mut text in text_query.iter_mut() {
                text.sections[0].value = format!("{:.0}", ammo);
            }
        }
    }
}

pub fn update_player_weapon_border_ui(
    player_query: Query<&Player>,
    mut color_query: Query<(&WeaponUi, &mut BorderColor), With<BorderColorEdit>>,
) {
    // Récupère le joueur (on suppose qu'il n'y a qu'un seul joueur)
    if let Some(player) = player_query.iter().next() {
        // Récupère l'arme actuelle directement sans Option
        let actual_weapon = player.actual_weapon;

        // Pour chaque bordure avec une arme associée
        for (weapon_ui, mut border_color) in color_query.iter_mut() {
            // Vérifie si l'arme actuelle est celle associée à la texture de l'arme
            if actual_weapon == weapon_ui.weapon {
                // Met la bordure en rouge si c'est l'arme actuelle
                border_color.0 = Color::Srgba(Srgba::new(0.737, 0.024, 0.012, 0.9));
            } else {
                // Met la bordure en noir sinon
                border_color.0 = Color::BLACK;
            }
        }
    }
}
pub fn update_player_weapon_ui(
    player_query: Query<&Player>,
    mut iu_weapon_query: Query<
        (
            &WeaponUi,
            &mut BorderColor,
            &mut BackgroundColor,
            &mut UiImage,
        ),
        With<BorderColorEdit>,
    >,
    asset_server: Res<AssetServer>,
) {
    if let Some(player) = player_query.iter().next() {
        let actual_weapon = player.actual_weapon;

        // Mise à jour de la couleur de la bordure
        update_weapon_border_color(actual_weapon, &mut iu_weapon_query);

        // Mise à jour de la couleur de fond
        update_weapon_color_image(player, &mut iu_weapon_query, asset_server);

        // Mise à jour de la couleur de fond
        update_weapon_background_color(player, &mut iu_weapon_query);
    }
}

/// Met à jour la couleur de la bordure en fonction de l'arme actuelle
fn update_weapon_border_color(
    actual_weapon: Weapon,
    iu_weapon_query: &mut Query<
        (
            &WeaponUi,
            &mut BorderColor,
            &mut BackgroundColor,
            &mut UiImage,
        ),
        With<BorderColorEdit>,
    >,
) {
    for (weapon_ui, mut border_color, _, _) in iu_weapon_query.iter_mut() {
        // Mise à jour de la couleur de la bordure
        if actual_weapon == weapon_ui.weapon {
            border_color.0 = Color::Srgba(Srgba::new(0.737, 0.024, 0.012, 0.9));
        // Rouge pour l'arme actuelle
        } else {
            border_color.0 = Color::BLACK; // Noir pour les autres armes
        }
    }
}

/// Met à jour la couleur de fond en fonction de la possession de l'arme
fn update_weapon_color_image(
    player: &Player,
    iu_weapon_query: &mut Query<
        (
            &WeaponUi,
            &mut BorderColor,
            &mut BackgroundColor,
            &mut UiImage,
        ),
        With<BorderColorEdit>,
    >,
    asset_server: Res<AssetServer>,
) {
    for (weapon_ui, _, _, mut image) in iu_weapon_query.iter_mut() {
        // Mise à jour de la couleur de fond
        if *player.owned_weapon.get(&weapon_ui.weapon).unwrap_or(&false) {
            // Si l'arme est possédée, pas de fond
            let weapon_name = format!("{}", weapon_ui.weapon);
            *image = UiImage::new(asset_server.load(format!("hud_weapon/{}.png", weapon_name)));
        } else {
            // Si l'arme n'est pas possédée, fond noir
            *image = UiImage::solid_color(Color::srgb(0., 0., 0.));
        }
    }
}

pub fn update_head_backgroung(
    time: Res<Time>,                    // Le temps pour suivre les mises à jour
    mut timer: ResMut<HeadUpdateTimer>, // Le timer qui gère les mises à jour toutes les secondes
    asset_server: Res<AssetServer>,     // Ressource partagée pour charger des assets
    mut image_query: Query<&mut UiImage, With<PlayerHead>>, // Composant pour l'animation du visage
    lobby: Res<PlayerLobby>,
    my_id: Res<MyClientId>,
) {
    // Avancer le timer
    if timer.0.tick(time.delta()).finished() {
        if let Some(attr) = lobby.0.get(&my_id.0) {
            for mut image in image_query.iter_mut() {
                let head_image = match attr.health {
                    h if h >= 80.0 => format!("100head1.png"),
                    h if h < 80.0 && h >= 60.0 => format!("80head2.png"),
                    h if h < 60.0 && h >= 40.0 => format!("60head2.png"),
                    h if h < 40.0 && h >= 20.0 => format!("40head2.png"),
                    h if h == 0.0 => format!("0head.png"),
                    _ => format!("20head2.png"),
                };

                *image = UiImage::new(asset_server.load(format!("textures/{}", head_image)));
            }
        }
    }
}

pub fn update_head(
    time: Res<Time>,                    // Le temps pour suivre les mises à jour
    mut timer: ResMut<HeadUpdateTimer>, // Le timer qui gère les mises à jour toutes les secondes
    asset_server: Res<AssetServer>,     // Ressource partagée pour charger des assets
    mut image_query: Query<&mut UiImage, With<PlayerHead>>, // Composant pour l'animation du visage
    lobby: Res<PlayerLobby>,
    my_id: Res<MyClientId>,
) {
    // Avancer le timer
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        if let Some(attr) = lobby.0.get(&my_id.0) {
            for mut image in image_query.iter_mut() {
                let random_number = rng.gen_range(1..=3); // Génère un nombre entre 1 et 3
                let head_image = match attr.health {
                    h if h >= 80.0 => format!("100head{}.png", random_number),
                    h if h < 80.0 && h >= 60.0 => format!("80head{}.png", random_number),
                    h if h < 60.0 && h >= 40.0 => format!("60head{}.png", random_number),
                    h if h < 40.0 && h >= 20.0 => format!("40head{}.png", random_number),
                    h if h == 0.0 => format!("0head.png"),
                    _ => format!("20head{}.png", random_number),
                };

                *image = UiImage::new(asset_server.load(format!("textures/{}", head_image)));
            }
        }
    }
}

/// Met à jour la couleur de fond en fonction de la possession de l'arme
fn update_weapon_background_color(
    player: &Player,
    iu_weapon_query: &mut Query<
        (
            &WeaponUi,
            &mut BorderColor,
            &mut BackgroundColor,
            &mut UiImage,
        ),
        With<BorderColorEdit>,
    >,
) {
    for (weapon_ui, _, mut background_color, _) in iu_weapon_query.iter_mut() {
        // Mise à jour de la couleur de fond
        if *player.owned_weapon.get(&weapon_ui.weapon).unwrap_or(&false) {
            // Si l'arme est possédée, pas de fond
            *background_color = BackgroundColor(Color::NONE);
        } else {
            // Si l'arme n'est pas possédée, fond noir
            *background_color = BackgroundColor(Color::BLACK);
        }
    }
}
pub fn setup_timer(mut commands: Commands) {
    // Initialisation du timer pour 1 seconde
    commands.insert_resource(HeadUpdateTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
}
