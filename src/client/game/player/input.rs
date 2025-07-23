use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PlayerInput {
    pub movement: Vec2,
    pub jump: bool,
    pub crouch: bool,      // Ajout de l'état accroupi
    pub crouch_speed: f32, // Ajout de la vitesse accroupi
    pub run: f32,          // Vitesse de course
}
