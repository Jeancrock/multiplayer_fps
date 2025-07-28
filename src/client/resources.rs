// src/client/resources.rs

use std::collections::HashSet;

use bevy::{asset::Handle, ecs::system::Resource, render::texture::Image};
use renet::ClientId;

#[derive(Resource)]
pub struct MyClientId(pub ClientId);

#[derive(Resource)]
pub struct MyUsername(pub String);

impl MyUsername {
    pub fn new(username: String) -> Self {
        return Self(username);
    }
}

#[derive(Resource, Default)]
pub struct IsSynced(pub bool);

#[derive(Resource)]
pub struct SkyCubeMap {
    pub image: Handle<Image>,
    pub loaded: bool,
}
#[allow(dead_code)]
#[derive(Resource, Default)]
pub struct SpawnedPlayers(pub HashSet<ClientId>);
