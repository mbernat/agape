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
        .insert_resource(DebugLines {
            depth_test: true,
            ..Default::default()
        })
        .insert_resource(Rays { lines: vec![] })
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
    lines: Vec<Line>,
}

struct Line {
    start: Vec3,
    end: Vec3,
    step: u8,
}

// TODOs:
// 1. hit the world (first a triangle, then a mesh, then the scene)
// 2. get the hit point data and draw them (point + normal?)
// 3. reflect the ray
// 4. refract the ray
// 5. multisampling

struct Hit {
    pos: Vec3,
    norm: Vec3,
}

struct Sphere {
    pos: Vec3,
    radius: f32,
}

fn hit_sphere(s: &Sphere, ray: &Ray) -> Option<Hit> {
    let oc = ray.origin - s.pos;
    let a = ray.direction.length_squared();
    let b = 2. * oc.dot(ray.direction);
    let c = oc.length_squared() - s.radius.powi(2);
    let disc = b.powi(2) - 4. * a * c;
    if disc < 0. {
        return None;
    }
    let tmid = -b / (2. * a);
    let td = disc.sqrt() / (2. * a);
    let t = if tmid - td >= 1e-3 {
        Some(tmid - td)
    } else if tmid + td >= 1e-3 {
        Some(tmid + td)
    } else {
        None
    };
    let pos = ray.origin + t? * ray.direction;
    let norm = (pos - s.pos).normalize();
    Some(Hit { pos, norm })
}

// Needs input from the scene; for now hard-coding simple shapes
fn hit(spheres: &Vec<Sphere>, ray: &Ray) -> Option<Hit> {
    hit_sphere(&spheres[0], ray)
}

// Needs input from the material, for now doing a simple reflection
fn generate_rays(ray: &Ray, hit: &Hit) -> Vec<Ray> {
    // This dot product should have be negative, if all went well
    let parallel = ray.direction.dot(hit.norm) * hit.norm;
    vec![Ray {
        origin: hit.pos,
        direction: ray.direction - 2. * parallel,
    }]
}

fn process_rays(
    ray_query: Query<&Ray, With<Initial>>,
    scene_query: Query<(&HitSphere, &Transform)>,
    mut rays: ResMut<Rays>,
) {
    let mut spheres: Vec<Sphere> = vec![];
    for (hs, trans) in scene_query.iter() {
        let sphere = Sphere {
            pos: trans.translation,
            radius: hs.radius,
        };
        spheres.push(sphere);
    }

    let mut old_rays: Vec<Ray> = ray_query.iter().map(Clone::clone).collect();
    rays.lines = vec![];
    for step in 0..3 {
        let mut new_rays = vec![];
        for ray in old_rays {
            if let Some(hit) = hit(&spheres, &ray) {
                let line = Line {
                    start: ray.origin,
                    end: hit.pos,
                    step,
                };
                rays.lines.push(line);
                let extra_rays = generate_rays(&ray, &hit);
                new_rays.extend(extra_rays);
            } else {
                let start = ray.origin;
                let end = ray.origin + 1000. * ray.direction.normalize();
                rays.lines.push(Line { start, end, step });
            }
        }
        old_rays = new_rays.clone();
    }
}

struct MoveRight;
struct HitSphere {
    radius: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    wireframe_config.global = true;

    let purple = materials.add(Color::rgba(0.7, 0.5, 0.9, 1.).into());
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
            material: purple.clone(),
            transform: Transform::from_xyz(0., 0., 0.),
            ..Default::default()
        })
        .insert(Wireframe);

    let radius = 1.;
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius,
                subdivisions: 5,
            })),
            material: purple.clone(),
            transform: Transform::from_xyz(1., 3., -0.5),
            ..Default::default()
        })
        .insert(HitSphere { radius })
        .insert(MoveRight);

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0., 5., 10.).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(2., 2., 4.),
        ..Default::default()
    });

    let steps = 20;
    let inc = -0.01;
    for x in 0..steps {
        for y in 0..steps {
            for z in 0..steps {
                let s: Vec3 = (-0.5, 0., 0.).into();
                let d: Vec3 = (x as f32, y as f32, z as f32).into();
                commands
                    .spawn()
                    .insert(Ray {
                        origin: s + inc * d,
                        direction: (1., 1., 0.).into(),
                    })
                    .insert(Initial);
            }
        }
    }
}

fn draw_rays(mut lines: ResMut<DebugLines>, rays: Res<Rays>, query: Query<&Ray>) {
    for line in rays.lines.iter() {
        let duration = 0.;
        let alpha = if line.step > 0 { 0.2 } else { 0.005 };
        lines.line_colored(
            line.start,
            line.end,
            duration,
            Color::rgba(0., 1., 0., alpha),
        );
    }
}

fn increment(mut query: Query<&mut Transform, With<MoveRight>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x += 0.002;
        //transform.rotate(Quat::from_axis_angle(Vec3::splat(1.), 0.01));
    }
}
