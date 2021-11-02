use bevy::prelude::*;

fn setup(mut commands: Commands) {
    commands.spawn().insert(Counter(0));
}

struct Counter(u32);

fn increment(mut query: Query<&mut Counter>) {
    for mut counter in query.iter_mut() {
        counter.0 += 1;
        println!("{}", counter.0)
    }
}

fn main() {
    AppBuilder::default()
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup.system())
    .add_system(increment.system())
    .run();
}
