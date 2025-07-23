
use bevy::ecs::system::Resource;
use renet::ClientId;

#[derive(Resource)]
pub struct MyClientId(pub ClientId);