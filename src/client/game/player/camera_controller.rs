use bevy::{input::mouse::MouseMotion, prelude::*};
use multiplayer_demo::Player;

/// Composant contrôlant la caméra FPS
#[derive(Component)]
pub struct CameraController {
    pub rotation: Vec2,         // (pitch, yaw)
    pub rotation_lock: f32,     // Limite haut/bas de la caméra (pitch)
    pub sensitivity: f32,       // Sensibilité de la souris
}

/// Met à jour l'orientation de la caméra à partir des mouvements de la souris
pub fn update_camera_controller(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_query: Query<(&mut CameraController, &mut Transform)>,
    mut player_query: Query<&mut Player>,
) {
    // Une seule caméra FPS contrôlée
    if let Ok((mut controller, mut transform)) = camera_query.get_single_mut() {
        // Appliquer tous les mouvements de souris reçus ce frame
        for ev in mouse_motion_events.read() {
            controller.rotation.y -= ev.delta.x * controller.sensitivity; // yaw (gauche/droite)
            controller.rotation.x -= ev.delta.y * controller.sensitivity; // pitch (haut/bas)

            // Clamp vertical (pitch)
            controller.rotation.x = controller
                .rotation
                .x
                .clamp(-controller.rotation_lock, controller.rotation_lock);
        }

        // Création de la rotation finale
        let yaw = Quat::from_axis_angle(Vec3::Y, controller.rotation.y.to_radians());
        let pitch = Quat::from_axis_angle(Vec3::X, controller.rotation.x.to_radians());

        // Rotation caméra = yaw * pitch (ordre important)
        transform.rotation = yaw * pitch;

        // Mise à jour de la rotation du joueur (corps) uniquement avec le yaw (axe Y)
        if let Ok(mut player) = player_query.get_single_mut() {
            player.rotation = yaw;
        }
    }
}
