use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
};

use super::{crosshair, hud};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, (crosshair::spawn_crosshair, hud::setup_hud_ui,hud::setup_timer))
            .add_systems(
                Update,
                (
                    hud::update_fps_ui,
                    hud::update_game_time_ui,
                    hud::update_player_health_ui,
                    hud::update_player_armor_ui,
                    hud::update_player_ammo_ui,
                    hud::update_player_weapon_ui,
                    hud::update_head,
                    hud::update_head_backgroung,
                ),
            );
    }
}
