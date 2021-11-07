use bevy::prelude::*;
use bevy::render::wireframe::*;
use bevy::wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

fn main() {
    AppBuilder::default()
        .insert_resource(Msaa { samples: 8 })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                features: vec![WgpuFeature::NonFillPolygonMode],
            },
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_plugin(DebugLinesPlugin)
        .insert_resource(DebugLines { depth_test: true, ..Default::default()})
        .insert_resource(Rays { lines: vec![]})
        .add_startup_system(setup.system())
        .add_system(increment.system())
        // TODO make the ray system into a plugin
        // TODO order the systems properly
        .add_system(process_rays.system())
        .add_system(draw_rays.system())
        .run();
}

#[derive(Clone)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

struct Initial;

struct Rays {
    lines: Vec<Line>
}

struct Line {
    start: Vec3,
    end: Vec3
}

// TODOs:
// 1. hit the world (first a triangle, then a mesh, then the scene)
// 2. get the hit point data and draw them (point + normal?)
// 3. reflect the ray
// 4. refract the ray
// 5. multisampling

struct Hit {
    p: Vec3,
    n: Vec3,
}

// Needs input from the scene; for now hard-coding simple shapes
fn hit(ray: &Ray) -> Option<Hit> {
    None
}

// Needs input from the material
fn generate_rays(ray: &Ray, hit: &Hit) -> Vec<Ray> {
    vec![]
}

fn process_rays(query: Query<&Ray, With<Initial>>, mut rays: ResMut<Rays>) {
    let mut old_rays: Vec<Ray> = query.iter().map(Clone::clone).collect();
    rays.lines = vec![];
    for _step in 0..3 {
        let mut new_rays = vec![];
        for ray in old_rays {
            if let Some(hit) = hit(&ray) {
                let line = Line { start: ray.origin, end: hit.p};
                rays.lines.push(line);
                let extra_rays = generate_rays(&ray, &hit);                
                new_rays.extend(extra_rays);
            } else {
                let start = ray.origin;
                let end = ray.origin + 1000. * ray.direction.normalize();
                rays.lines.push(Line {start, end});
            }
        }
        old_rays = new_rays.clone();
    }
}

struct MoveRight;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    wireframe_config.global = true;

    let purple = materials.add(Color::rgb(0.5, 0.2, 0.7).into());
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
            material: purple.clone(),
            transform: Transform::from_xyz(-5., 0., 0.),
            ..Default::default()
        })
        .insert(MoveRight);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 2.0 })),
            material: purple,
            transform: Transform::from_xyz(0., 0., 0.),
            ..Default::default()
        })
        .insert(Wireframe);

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0., 5., 10.).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(2., 2., 4.),
        ..Default::default()
    });

    for i in 0..10 {
        commands.spawn().insert(Ray {
            origin: (-1. + 0.1 * (i as f32), -1., -1.).into(),
            direction: Vec3::splat(1.),
        }).insert(Initial);
    }
}

fn draw_rays(mut lines: ResMut<DebugLines>, rays: Res<Rays>, query: Query<&Ray>) {
    for line in rays.lines.iter() {
        let duration = 0.;
        lines.line(line.start, line.end, duration);
    }
}

fn increment(mut query: Query<&mut Transform, With<MoveRight>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x += 0.02;
        transform.rotate(Quat::from_axis_angle(Vec3::splat(1.), 0.01));
    }
}
