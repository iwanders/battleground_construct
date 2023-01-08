use crate::construct_render::render::{GeometryRef, RenderPass, RenderableGeometry};
use battleground_construct::components::velocity::velocity_on_body;
use battleground_construct::display;
use battleground_construct::util::cgmath::prelude::*;
use rand::Rng;
use three_d::*;

use super::RetainedEffect;

// use battleground_construct::util::cgmath::InvertHomogeneous;
use battleground_construct::util::cgmath::ToHomogenous;
// use battleground_construct::util::cgmath::ToQuaternion;
use battleground_construct::util::cgmath::ToRotationH;
struct DestructorParticle {
    pos: Mat4,
    vel: Vec3,
    color: Color,
    scale: Vec3,
    traveled: f32, // accumulating distance traveled
}

pub struct Deconstructor {
    participates_in_pass: fn(RenderPass) -> bool,
    renderable: Gm<InstancedMesh, PhysicalMaterial>,
    particles: Vec<DestructorParticle>,

    /// Keep track of the last update time, to integrate velocity.
    last_time: f32,

    /// Max traveled to fully have faded.
    max_traveled: f32,
}

impl Deconstructor {
    pub fn new(
        context: &Context,
        _effect_position: Matrix4<f32>,
        time: f32,
        elements: &[(display::primitives::Element, Twist<f32>, Mat4)],
        impacts: &[(Mat4, f32)],
    ) -> Self {
        let edge_x = 0.05;
        let edge_y = edge_x;
        let edge_z = edge_x;
        // let edge_z = 1.0;
        let material = three_d::renderer::material::PhysicalMaterial::new_transparent(
            context,
            &CpuMaterial {
                albedo: Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
                ..Default::default()
            },
        );

        let instances: three_d::renderer::geometry::Instances = Default::default();
        let renderable = Gm::new(
            InstancedMesh::new(context, &instances, &CpuMesh::cube()),
            material,
        );

        let mut particles = vec![];

        let mut rng = rand::thread_rng();
        use rand_distr::StandardNormal;

        let mut rand_f32 = || rng.sample::<f32, StandardNormal>(StandardNormal);

        const DO_FULL_EXPLOSION: bool = true;

        for (el, entity_twist, entity_pose) in elements.iter() {
            struct Fragment {
                pos: Vector3<f32>,
                scale: Vector3<f32>,
            }

            fn generate_fragments(
                x_range: std::ops::Range<isize>,
                y_range: std::ops::Range<isize>,
                z_range: std::ops::Range<isize>,
                edge_x: f32,
                edge_y: f32,
                edge_z: f32,
                fun: &dyn Fn(Vec3) -> bool,
            ) -> Vec<Fragment> {
                let mut fragments = vec![];
                for x in x_range.start..x_range.end {
                    for y in y_range.start..y_range.end {
                        for z in z_range.start..z_range.end {
                            let x_start = x as f32 * edge_x;
                            let x_end = (x + 1) as f32 * edge_x;

                            let y_start = y as f32 * edge_y;
                            let y_end = (y + 1) as f32 * edge_y;

                            let z_start = z as f32 * edge_z;
                            let z_end = (z + 1) as f32 * edge_z;

                            let p = vec3(
                                (x_end - x_start) / 2.0 + x_start,
                                (y_end - y_start) / 2.0 + y_start,
                                (z_end - z_start) / 2.0 + z_start,
                            );
                            if fun(p) {
                                let sx = edge_x / 2.0;
                                let sy = edge_y / 2.0;
                                let sz = edge_z / 2.0;
                                fragments.push(Fragment {
                                    pos: p,
                                    scale: vec3(sx, sy, sz),
                                });
                            }
                        }
                    }
                }
                fragments
            }

            let center_element = entity_pose * el.transform;
            let fragments = match el.primitive {
                battleground_construct::display::primitives::Primitive::Cuboid(c) => {
                    let mut fragments = vec![];
                    let half_length = c.length / 2.0;
                    let half_width = c.width / 2.0;
                    let half_height = c.height / 2.0;

                    // calculate the translations from the center of the cuboid.
                    let chunks_x = (((c.length / 2.0) / edge_x) as isize) + 1;
                    let chunks_y = (((c.width / 2.0) / edge_y) as isize) + 1;
                    let chunks_z = (((c.height / 2.0) / edge_z) as isize) + 1;
                    for x in -chunks_x..chunks_x {
                        for y in -chunks_y..chunks_y {
                            for z in -chunks_z..chunks_z {
                                let x_start = (x as f32 * edge_x).clamp(-half_length, half_length);
                                let x_end =
                                    ((x + 1) as f32 * edge_x).clamp(-half_length, half_length);

                                let y_start = (y as f32 * edge_y).clamp(-half_width, half_width);
                                let y_end =
                                    ((y + 1) as f32 * edge_y).clamp(-half_width, half_width);

                                let z_start = (z as f32 * edge_z).clamp(-half_height, half_height);
                                let z_end =
                                    ((z + 1) as f32 * edge_z).clamp(-half_height, half_height);

                                if x_start == x_end || y_start == y_end || z_start == z_end {
                                    continue;
                                }

                                let p = vec3(
                                    (x_end - x_start) / 2.0 + x_start,
                                    (y_end - y_start) / 2.0 + y_start,
                                    (z_end - z_start) / 2.0 + z_start,
                                );
                                let sx = (x_end - x_start) / 2.0;
                                let sy = (y_end - y_start) / 2.0;
                                let sz = (z_end - z_start) / 2.0;
                                // let fragment_pos = Mat4::from_translation(p);
                                fragments.push(Fragment {
                                    pos: p,
                                    scale: vec3(sx, sy, sz),
                                });
                            }
                        }
                    }

                    fragments
                }
                battleground_construct::display::primitives::Primitive::Sphere(sphere) => {
                    let radius = sphere.radius;

                    let chunks_x = ((radius / edge_x) as isize) + 1;
                    let chunks_y = ((radius / edge_y) as isize) + 1;
                    let chunks_z = ((radius / edge_z) as isize) + 1;
                    generate_fragments(
                        -chunks_x..chunks_x,
                        -chunks_y..chunks_y,
                        -chunks_z..chunks_z,
                        edge_x,
                        edge_y,
                        edge_z,
                        &|p: Vec3| p.euclid_norm() <= radius,
                    )
                }
                battleground_construct::display::primitives::Primitive::Cylinder(cylinder) => {
                    let radius = cylinder.radius;
                    let height = cylinder.height;

                    let chunks_x = ((height / edge_x) as isize) + 1;
                    let chunks_y = ((radius / edge_y) as isize) + 1;
                    let chunks_z = ((radius / edge_z) as isize) + 1;
                    generate_fragments(
                        0..chunks_x,
                        -chunks_y..chunks_y,
                        -chunks_z..chunks_z,
                        edge_x,
                        edge_y,
                        edge_z,
                        &|p: Vec3| vec3(0.0, p.y, p.z).euclid_norm() <= radius,
                    )
                }
                battleground_construct::display::primitives::Primitive::Line(_) => todo!(),
                battleground_construct::display::primitives::Primitive::Cone(cone) => {
                    let radius = cone.radius;
                    let height = cone.height;
                    let chunks_x = ((height / edge_x) as isize) + 1;
                    let chunks_y = ((radius / edge_y) as isize) + 1;
                    let chunks_z = ((radius / edge_z) as isize) + 1;
                    generate_fragments(
                        0..chunks_x,
                        -chunks_y..chunks_y,
                        -chunks_z..chunks_z,
                        edge_x,
                        edge_y,
                        edge_z,
                        &|p: Vec3| {
                            vec3(0.0, p.y, p.z).euclid_norm() <= (radius * (1.0 - (p.x / height)))
                        },
                    )
                }
                battleground_construct::display::primitives::Primitive::Circle(_) => {
                    // skip circle for now, they're infinitesimally thin.
                    vec![]
                }
                battleground_construct::display::primitives::Primitive::ExtrudedRectangle(_) => {
                    // Skip for now, will do later if this feels necessary
                    vec![]
                }
            };

            for fragment in fragments {
                let fragment_world_pos =
                    entity_pose * el.transform * Mat4::from_translation(fragment.pos);

                // Start velocity calculation, initialise with zero.
                let mut vel = vec3(0.0, 0.0, 0.0);

                // Add the velocity on this body.
                vel += velocity_on_body((*entity_twist).into(), fragment_world_pos - entity_pose).v;

                // Add outward from the body center.
                let cube_world = fragment_world_pos;
                let dir = cube_world.w.truncate() - center_element.w.truncate();
                let pos = (fragment_world_pos).to_rotation_h();
                if DO_FULL_EXPLOSION {
                    vel += (dir.to_h() * pos).w.truncate() * 0.1;
                }

                // Add some random jitter, such that it looks prettier.
                if DO_FULL_EXPLOSION {
                    vel += vec3(rand_f32(), rand_f32(), rand_f32()) * 0.1;
                }

                // Then, add velocities away from the impacts.
                if DO_FULL_EXPLOSION {
                    for (impact_location, magnitude) in impacts.iter() {
                        let p1 = impact_location.to_translation();
                        let p0 = fragment_world_pos.to_translation();
                        let rotation =
                            Quat::from_arc(vec3(1.0, 0.0, 0.0), (p0 - p1).normalize(), None);
                        let d = (p1.distance2(p0)).sqrt();
                        let mag = magnitude * (1.0 / (d * d));
                        if DO_FULL_EXPLOSION {
                            vel += (rotation * vec3(1.0, 0.0, 0.0)) * mag;
                        }
                    }
                }

                let material =
                    if let display::primitives::Material::FlatMaterial(material) = el.material {
                        material
                    } else {
                        panic!("unsupported material");
                    };
                particles.push(DestructorParticle {
                    pos: fragment_world_pos,
                    color: Color::new(material.color.r, material.color.g, material.color.b, 128),
                    vel,
                    scale: fragment.scale,
                    traveled: 0.0,
                });
            }
        }
        // let particles = vec![particles.pop().unwrap()];

        Deconstructor {
            participates_in_pass: |pass| pass == RenderPass::BaseScene,
            last_time: time,
            renderable,
            particles,
            max_traveled: 10.0,
        }
    }
}

impl RetainedEffect for Deconstructor {
    fn as_renderable_geometry(&self) -> &dyn RenderableGeometry {
        self as &dyn RenderableGeometry
    }

    fn as_renderable_geometry_mut(&mut self) -> &mut dyn RenderableGeometry {
        self as &mut dyn RenderableGeometry
    }

    fn update(
        &mut self,
        _effect_type: &display::primitives::EffectType,
        _camera: &Camera,
        _entity_position: Matrix4<f32>,
        time: f32,
    ) {
        let dt = time - self.last_time;

        for particle in self.particles.iter_mut() {
            // println!();
            // println!("Pre pose: {:?}", particle.pos);
            // Always update position.
            let before = particle.pos.to_translation();
            particle.pos.w += particle.vel.extend(0.0) * dt;
            let after = particle.pos.to_translation();
            let d = after.distance2(before).sqrt();
            particle.traveled += d;
            let ratio_of_lifetime = 1.0 - (particle.traveled / self.max_traveled).min(1.0);
            particle.color.a = std::cmp::min(particle.color.a, (255f32 * ratio_of_lifetime) as u8);

            let accel = -9.81 * 0.25;
            // let accel = -9.81 * 0.0;
            let gravity = vec3(0.0f32, 0.0, accel).to_h();
            let rot = particle.pos.to_rotation_h();
            particle.vel += (gravity * rot).w.truncate() * dt;
            if particle.pos.w[2] <= 0.0 {
                particle.vel[0] *= 0.5;
                particle.vel[1] *= 0.5;
                particle.vel[2] = -particle.vel[2] * 0.5;
                particle.color.a = particle.color.a.saturating_sub(20);
            }
        }

        self.particles = self
            .particles
            .drain(..)
            .filter(|p| p.color.a != 0)
            .collect::<_>();

        self.last_time = time;
    }
}

impl RenderableGeometry for Deconstructor {
    fn objects(&self, pass: RenderPass) -> Vec<&dyn Object> {
        if (self.participates_in_pass)(pass) {
            vec![&self.renderable]
        } else {
            vec![]
        }
    }

    fn geometries(&self, pass: RenderPass) -> Vec<GeometryRef> {
        if (self.participates_in_pass)(pass) {
            vec![GeometryRef::InstancedMesh(&self.renderable.geometry)]
        } else {
            vec![]
        }
    }

    fn finish_scene(&mut self, _context: &Context) {
        let (transforms, colors) = self
            .particles
            .iter()
            .map(|p| {
                (
                    p.pos * Mat4::from_nonuniform_scale(p.scale.x, p.scale.y, p.scale.z),
                    p.color,
                )
            })
            .unzip();
        self.renderable.geometry.set_instances(&Instances {
            transformations: transforms,
            colors: Some(colors),
            ..Default::default()
        });
    }
}
