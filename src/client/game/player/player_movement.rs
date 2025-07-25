use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use multiplayer_demo::Player;

use super::{camera_controller::CameraController, input::*};

/// Système qui lit les touches clavier et met à jour les intentions de mouvement du joueur
pub fn update_movement_input(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<PlayerInput>) {
    input.movement = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        input.movement.x -= 1.;
    }
    if keys.pressed(KeyCode::KeyA) {
        input.movement.y -= 1.;
    }
    if keys.pressed(KeyCode::KeyS) {
        input.movement.x += 1.;
    }
    if keys.pressed(KeyCode::KeyD) {
        input.movement.y += 1.;
    }

    input.jump = keys.pressed(KeyCode::Space);

    if keys.pressed(KeyCode::ControlLeft) {
        input.crouch = true;
        input.crouch_speed = 0.6;
    } else {
        input.crouch = false;
        input.crouch_speed = 1.0;
    }

    if keys.pressed(KeyCode::ShiftLeft) {
        input.run = 2.0 * input.crouch_speed;
    } else {
        input.run = 1.0 * input.crouch_speed;
    }
}

/// Système qui applique les déplacements physiques du joueur à chaque tick fixe (physique)
pub fn update_movement(
    time: Res<Time<Fixed>>,
    input: Res<PlayerInput>,
    mut camera_query: Query<&mut Transform, (With<Camera>, With<CameraController>)>,
    camera_controller_query: Query<&CameraController>,
    mut player_query: Query<(
        &mut Player,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
) {
    // Récupère le contrôleur de caméra (orientation du joueur)
    let Ok(camera_controller) = camera_controller_query.get_single() else {
        return;
    };

    let yaw_radians = camera_controller.rotation.y.to_radians();

    // Calcule les vecteurs avant/droite en fonction du yaw
    let forward = Vec2::new(yaw_radians.sin(), yaw_radians.cos());
    let right = Vec2::new(forward.y, -forward.x);

    // Vitesse de transition accroupi / debout
    let crouch_transition_speed = 4.0;

    for (mut player, mut controller, controller_output) in player_query.iter_mut() {
        // Gestion du saut si le joueur est au sol
        if let Some(output) = controller_output {
            if output.grounded {
                player.velocity.y = 0.0;
                if input.jump {
                    player.velocity.y = player.jump_strength;
                }
            }
        }

        // Mouvement horizontal : si un input est actif, calcule la direction et applique la vitesse
        if let Some(dir) = (forward * input.movement.x + right * input.movement.y).try_normalize() {
            player.velocity.x = dir.x * player.speed * input.run * 2.;
            player.velocity.z = dir.y * player.speed * input.run * 2.;
        } else {
            // Aucun input → stop le mouvement horizontal (empêche la glisse)
            player.velocity.x = 0.0;
            player.velocity.z = 0.0;
        }

        // Gravité (accélération vers le bas)
        player.velocity.y -= player.gravity * time.timestep().as_secs_f32();

        // Application du déplacement basé sur la vélocité calculée
        controller.translation = Some(player.velocity * time.timestep().as_secs_f32());

        // Ajuste la hauteur de la caméra (accroupi / debout)
        let target_camera_y = if input.crouch { 0.5 } else { 1.5 };

        for mut cam_transform in camera_query.iter_mut() {
            let current = cam_transform.translation.y;
            let new = current
                + (target_camera_y - current)
                    * crouch_transition_speed
                    * time.timestep().as_secs_f32();
            cam_transform.translation.y = new - 0.14; // ⚠️ Décalage spécifique à ton modèle 3D
        }
    }
}

// use bevy::prelude::*;
// use bevy_rapier3d::prelude::*;
// use multiplayer_demo::Player;

// use super::{camera_controller::CameraController, input::*};

// pub fn update_movement_input(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<PlayerInput>) {
//     input.movement = Vec2::ZERO;

//     if keys.pressed(KeyCode::KeyW) {
//         input.movement.x += 1.;
//     }
//     if keys.pressed(KeyCode::KeyA) {
//         input.movement.y -= 1.;
//     }
//     if keys.pressed(KeyCode::KeyS) {
//         input.movement.x -= 1.;
//     }
//     if keys.pressed(KeyCode::KeyD) {
//         input.movement.y += 1.;
//     }

//     if keys.pressed(KeyCode::Space) {
//         input.jump = true; // Saut lorsque la touche espace est pressée
//     } else {
//         input.jump = false;
//     }

//     if keys.pressed(KeyCode::ControlLeft) {
//         input.crouch = true; // Accroupissement lorsque la touche Ctrl gauche est pressée
//         input.crouch_speed = 0.6;
//     } else {
//         input.crouch = false; // Normal lorsque Ctrl gauche est relâché
//         input.crouch_speed = 1.;
//     }

//     if keys.pressed(KeyCode::ShiftLeft) {
//         input.run = 2. * input.crouch_speed; // Accélère lorsque Shift gauche est pressé
//     } else {
//         input.run = 1. * input.crouch_speed; // Vitesse normale lorsque Shift gauche est relâché
//     }
// }

// pub fn update_movement(
//     time: Res<Time<Fixed>>,
//     input: Res<PlayerInput>,
//     mut camera_query: Query<&mut Transform, With<Camera>>, // Accès à la caméra
//     camera_controller_query: Query<&CameraController>,
//     mut player_query: Query<(
//         &mut Player,
//         &mut KinematicCharacterController,
//         Option<&KinematicCharacterControllerOutput>,
//     )>,
// ) {
//     let camera = camera_controller_query.get_single().unwrap();

//     // Définir la vitesse de transition de la caméra pour accroupissement
//     let crouch_transition_speed = 4.0; // Ajustez cette valeur pour ralentir ou accélérer la transition

//     for (mut player, mut controller, controller_output) in player_query.iter_mut() {
//         if let Some(output) = controller_output {
//             if output.grounded {
//                 player.velocity = Vec3::ZERO; // Remise à zéro de la vélocité si au sol
//                 if input.jump {
//                     player.velocity.y = player.jump_strength; // Applique une force de saut
//                 }
//             }
//         }

//         // Logique de rotation de la caméra pour le mouvement du joueur
//         let camera_rotation_converted = -camera.rotation.y.to_radians() - 90.0_f32.to_radians();
//         let forward = Vec2::new(
//             f32::cos(camera_rotation_converted),
//             f32::sin(camera_rotation_converted),
//         );
//         let right = Vec2::new(-forward.y, forward.x);

//         if let Some(movement_direction) =
//             (forward * input.movement.x + right * input.movement.y).try_normalize()
//         {
//             player.velocity.x = movement_direction.x * player.speed * input.run * 2.;
//             player.velocity.z = movement_direction.y * player.speed * input.run * 2.;
//         }

//         // Applique la gravité
//         player.velocity.y -= player.gravity * time.timestep().as_secs_f32();

//         // Déplace le joueur
//         controller.translation = Some(player.velocity * time.timestep().as_secs_f32());

//         // Ajuste la hauteur de la caméra en fonction de l'état accroupi
//         let target_camera_height = if input.crouch { 0.5 } else { 1.5 }; // Hauteur accroupie ou normale

//         for mut camera_transform in camera_query.iter_mut() {
//             let current_height = camera_transform.translation.y;
//             let new_height = f32::lerp(
//                 current_height,
//                 target_camera_height,
//                 crouch_transition_speed * time.timestep().as_secs_f32(),
//             );

//             camera_transform.translation.y = new_height - 0.14; // Applique la nouvelle hauteur
//         }
//     }
// }
