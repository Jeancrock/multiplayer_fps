use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowMode, WindowResolution},
};

use crate::game::cursor::*;
pub struct WindowSettingsPlugin;
impl Plugin for WindowSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(cursor::CursorPlugin)
            .add_systems(PreStartup, init_window);
    }
}

fn init_window(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = window_query.get_single_mut() {
        let width = window.width();
        let height = window.height();
        window.resolution = WindowResolution::new(height, width);
        window.mode = WindowMode::BorderlessFullscreen;
        println!("WINDOW WIDTH: {}", window.width());
    }
}
