use bevy::prelude::*;

pub fn spawn_crosshair(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1, // rend après la caméra 3D
            ..default()
        },
        ..default()
    });

    let crosshair_size = 2.0;

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center, // Centrage horizontal
                align_items: AlignItems::Center,         // Centrage vertical
                ..default()
            },
            inherited_visibility: InheritedVisibility::VISIBLE,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage::solid_color(Color::srgb(1., 0., 0.)),
                style: Style {
                    width: Val::Px(crosshair_size),
                    height: Val::Px(crosshair_size),
                    ..default()
                },
                inherited_visibility: InheritedVisibility::VISIBLE,
                ..default()
            });
        });
}
