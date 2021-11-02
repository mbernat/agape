use bevy::prelude::*;

struct Square;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(1., 1.)),
            material: materials.add(ColorMaterial::color([0.5, 0.2, 0.5, 1.0].into())),
            transform: Transform::from_translation([0., 0., -100.].into()),
            ..Default::default()
        })
        .insert(Square);
    commands.spawn_bundle(PerspectiveCameraBundle::new_3d());
}

fn increment(mut query: Query<&mut Transform, With<Square>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x += 0.1;
    }
}

fn main() {
    AppBuilder::default()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(increment.system())
        .run();
}
