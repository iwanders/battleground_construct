use crate::construct_render::render::{RenderPass, RenderableGeometry};
use crate::construct_render::util::ColorConvert;
use battleground_construct::components::velocity::velocity_on_body;
use battleground_construct::display;
use battleground_construct::util::cgmath::prelude::*;
use rand::rngs::ThreadRng;
use rand::Rng;
use three_d::*;

pub trait RetainedEffect: RenderableGeometry {
    fn update(
        &mut self,
        effect_type: &display::primitives::EffectType,
        camera: &Camera,
        entity_position: Matrix4<f32>,
        time: f32,
    );

    fn as_renderable_geometry(&self) -> &dyn RenderableGeometry;
    fn as_renderable_geometry_mut(&mut self) -> &mut dyn RenderableGeometry;
}

#[derive(Debug, Copy, Clone)]
struct Particle {
    pos: Mat4,
    vel: Vec3,
    color: Color,
    spawn_time: f32,
    expiry_time: f32,
    distance: f32,
    expiry_distance: f32,
}

impl Particle {
    pub fn new(
        pos: Mat4,
        vel: Vec3,
        color: Color,
        spawn_time: f32,
        expiry_time: f32,
        expiry_distance: f32,
    ) -> Self {
        Particle {
            pos,
            vel,
            color,
            spawn_time,
            expiry_time,
            distance: 0.0,
            expiry_distance,
        }
    }

    pub fn expired(&self, time: f32) -> bool {
        self.expiry_time < time
    }
}

/*
This thing could do with some work... overall it works, but it could be better;
- functions for how to spawn / emit particles / velocity / duration of spawning etc.
- functions that affect the behaviour of particles, both for velocity and acceleration.
*/
pub struct ParticleEmitter {
    participates_in_pass: fn(RenderPass) -> bool,
    renderable: Gm<InstancedMesh, ColorMaterial>,

    /// Keep track of the last update time, to integrate velocity.
    last_time: f32,

    /// Last spawn time.
    next_spawn_time: f32,

    /// Whether to force emitting to true next cycle, then set this flag to false.
    emit_next_cycle: bool,

    /// Interval at which to spawn particles.
    spawn_interval: f32,
    spawn_jitter: f32,

    /// Lifetime of each particle.
    lifetime: f32,
    lifetime_jitter: f32, // gaussian

    max_distance: f32,

    /// Color of each particle.
    color: three_d::Color,

    // acceleration: Vec3,
    /// Initial velocity of each particle.
    velocity: Vec3,
    velocity_jitter: Vec3,

    velocity_clamp_x: Option<(f32, f32)>,
    velocity_clamp_z: Option<(f32, f32)>,

    reflect_on_floor: bool,

    /// Whether to fade the alpha to the lifetime.
    fade_alpha_to_lifetime: bool,

    /// Make particles always face the camera
    face_camera: bool,

    /// Actual container of particles.
    particles: Vec<Particle>,

    rng: ThreadRng,
}

impl ParticleEmitter {
    pub fn new(
        context: &Context,
        entity_position: Matrix4<f32>,
        time: f32,
        display: &display::primitives::ParticleType,
    ) -> Self {
        let mut p_color: Color = Color::default();
        let p_size: f32;
        let lifetime = 0.4;
        let spawn_interval = 0.01;
        let mut initial_particle_velocity = vec3(0.0, 0.0, 0.0);
        let mut velocity_jitter = vec3(0.1, 0.1, 0.1);
        let mut next_spawn_time = time;
        let mut emit_next_cycle = false;
        let mut reflect_on_floor = false;
        let mut velocity_clamp_x = None;
        let velocity_clamp_z = None;
        let mut initial_particles = vec![];
        let mut max_distance: f32 = 1000000.0; // max distance the particle can travel before fading / expiring.
        let mut rng = rand::thread_rng();

        match display {
            display::primitives::ParticleType::BulletTrail { color, size } => {
                p_color = color.to_color();
                p_color.a = 128;
                p_size = *size;
            }
            display::primitives::ParticleType::BulletImpact {
                color,
                size,
                velocity,
            } => {
                p_color = color.to_color();
                p_color.a = 128;
                p_size = *size;
                initial_particle_velocity += *velocity * 0.1;
                next_spawn_time -= 20.0 * spawn_interval;
                velocity_jitter *= 3.0;
                emit_next_cycle = true;
                reflect_on_floor = true;
            }
            display::primitives::ParticleType::MuzzleFlash { color, size } => {
                p_color = color.to_color();
                p_color.a = 255;
                p_size = *size;
                initial_particle_velocity += vec3(10.0, 0.0, 0.0);
                next_spawn_time -= 150.0 * spawn_interval;
                velocity_jitter = vec3(7.0, 0.5, 0.5);
                // clamp x to avoid particles going backwards
                velocity_clamp_x = Some((0.0, 1000.0));
                emit_next_cycle = true;
                reflect_on_floor = true;
                max_distance = 2.5;
            }

            display::primitives::ParticleType::Explosion { radius } => {
                p_size = 0.03;
                let color_yellow = (0.75, 0.5, 0.0, 0.5); // pretty yellow.
                let color_red = (1.0, 0.3, 0.0, 0.5); // More red
                let c_interp = |p: f32, c1: (f32, f32, f32, f32), c2: (f32, f32, f32, f32)| {
                    let r = ((c1.0 * p + c2.0 * (1.0 - p)) * 255.0) as u8;
                    let g = ((c1.1 * p + c2.1 * (1.0 - p)) * 255.0) as u8;
                    let b = ((c1.2 * p + c2.2 * (1.0 - p)) * 255.0) as u8;
                    let a = ((c1.3 * p + c2.3 * (1.0 - p)) * 255.0) as u8;
                    Color { r, g, b, a }
                };
                let lifetime_jitter = 0.3;
                let lifetime = 0.4;
                let velocity_jitter = vec3(5.0, 3.0, 3.0);
                for _i in 0..1500 {
                    use rand_distr::StandardNormal;
                    // let spawn_val: f32 = rng.sample(StandardNormal);
                    let color_val: f32 = rng.gen();
                    // spawn particle.
                    let pos = entity_position;
                    let mut color = c_interp(color_val, color_yellow, color_red);
                    color.a /= 2;
                    let lifetime_val: f32 = rng.sample(StandardNormal);
                    let expiry_time = time + lifetime + lifetime_jitter * lifetime_val;

                    let v0: f32 = rng.sample(StandardNormal);
                    let v1: f32 = rng.sample(StandardNormal);
                    let v2: f32 = rng.sample(StandardNormal);
                    let vel = vec3(
                        v0 * velocity_jitter[0],
                        v1 * velocity_jitter[1],
                        (v2 * velocity_jitter[2]).abs(),
                    );

                    initial_particles.push(Particle::new(
                        pos,
                        vel,
                        color,
                        time,
                        expiry_time,
                        *radius,
                    ));
                }
                for _i in 0..1000 {
                    let color_val: f32 = rng.gen();
                    let pos = entity_position;
                    let color = c_interp(color_val, color_yellow, color_red);
                    let expiry_time = time + 10.0;

                    let direction: f32 = rng.gen();
                    let pvel: f32 = rng.gen();
                    let pvel = pvel * 5.0 + 0.75;
                    let direction = direction * std::f32::consts::PI * 2.0;
                    let vel = vec3(direction.cos() * pvel, direction.sin() * pvel, 0.0);

                    initial_particles.push(Particle::new(
                        pos,
                        vel,
                        color,
                        time,
                        expiry_time,
                        *radius,
                    ));
                }
            }
        }
        let color = p_color;
        let size = p_size;

        let mut square = CpuMesh::circle(8);
        // let mut square = CpuMesh::square();
        square.transform(&Mat4::from_scale(size)).unwrap();

        // renderable.set_material(z);

        // let mut material = three_d::renderer::material::PhysicalMaterial::new(
        let material = three_d::renderer::material::ColorMaterial::new_transparent(
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
        let renderable = Gm::new(InstancedMesh::new(context, &instances, &square), material);

        Self {
            participates_in_pass: |pass| pass == RenderPass::BaseScene,
            last_time: time,
            renderable,

            emit_next_cycle,
            next_spawn_time,
            spawn_interval,
            spawn_jitter: 0.00,
            lifetime,
            lifetime_jitter: 0.0,
            max_distance,

            velocity: initial_particle_velocity,
            velocity_jitter,
            velocity_clamp_x,
            velocity_clamp_z,
            reflect_on_floor,

            fade_alpha_to_lifetime: true,
            face_camera: true,

            particles: initial_particles,
            color,

            rng,
        }
    }
}

impl RetainedEffect for ParticleEmitter {
    fn as_renderable_geometry(&self) -> &dyn RenderableGeometry {
        self as &dyn RenderableGeometry
    }

    fn as_renderable_geometry_mut(&mut self) -> &mut dyn RenderableGeometry {
        self as &mut dyn RenderableGeometry
    }

    fn update(
        &mut self,
        effect_type: &display::primitives::EffectType,
        camera: &Camera,
        entity_position: Matrix4<f32>,
        time: f32,
    ) {
        // let emitting = if let display::primitives::EffectType::ParticleEmitter{emitting,..} = effect_type {*emitting} else {false};
        let emitting =
            if let display::primitives::EffectType::ParticleEmitter { emitting, .. } = *effect_type
            {
                emitting
            } else {
                panic!("Called renderable effect with wrong effect type");
            };
        let dt = time - self.last_time;

        // Drop particles that are expired.
        self.particles = self
            .particles
            .drain(..)
            .filter(|p| !p.expired(time))
            .collect::<_>();

        // Spawn new particles.
        while (self.next_spawn_time < time) && (emitting || self.emit_next_cycle) {
            use rand_distr::StandardNormal;
            let spawn_val: f32 = self.rng.sample(StandardNormal);
            self.next_spawn_time += self.spawn_interval + spawn_val * self.spawn_jitter;
            // spawn particle.
            let pos = entity_position;
            let color = self.color;
            let spawn_time = time;
            let lifetime_val: f32 = self.rng.sample(StandardNormal);
            let expiry_time = time + self.lifetime + self.lifetime_jitter * lifetime_val;

            let v0: f32 = self.rng.sample(StandardNormal);
            let v1: f32 = self.rng.sample(StandardNormal);
            let v2: f32 = self.rng.sample(StandardNormal);
            let mut v_initial = self.velocity
                + vec3(
                    v0 * self.velocity_jitter[0],
                    v1 * self.velocity_jitter[1],
                    v2 * self.velocity_jitter[2],
                );
            if let Some(velocity_clamp_x) = self.velocity_clamp_x {
                v_initial.x = v_initial.x.clamp(velocity_clamp_x.0, velocity_clamp_x.1);
            }
            if let Some(velocity_clamp_z) = self.velocity_clamp_z {
                v_initial.z = v_initial.z.clamp(velocity_clamp_z.0, velocity_clamp_z.1);
            }
            let vel = (entity_position.to_rotation_h() * v_initial.extend(0.0)).truncate();

            self.particles.push(Particle::new(
                pos,
                vel,
                color,
                spawn_time,
                expiry_time,
                self.max_distance,
            ));
        }
        self.emit_next_cycle = false;

        // Update position and alphas
        for particle in self.particles.iter_mut() {
            // Always update position.
            let before = particle.pos.to_translation();
            particle.pos.w += particle.vel.extend(0.0) * dt;
            let after = particle.pos.to_translation();
            let d = after.distance2(before).sqrt();
            particle.distance += d;

            if self.reflect_on_floor {
                if particle.pos.w.z < 0.0 {
                    if particle.vel.z < 0.0 {
                        particle.vel.z = particle.vel.z.abs();
                    }
                    particle.pos.w.z = 0.0;
                }
            }

            if self.fade_alpha_to_lifetime {
                let ratio_of_age =
                    (time - particle.spawn_time) / (particle.expiry_time - particle.spawn_time);
                let ratio_of_distance = (particle.distance / particle.expiry_distance).min(1.0);
                let max_ratio = ratio_of_distance.max(ratio_of_age);
                let alpha_scaled = (1.0 - max_ratio) * (self.color.a as f32);
                particle.color.a = alpha_scaled as u8;
            }

            if self.face_camera {
                particle.pos = Mat4::from_translation(particle.pos.w.truncate())
                    * Mat4::from_cols(
                        camera.view().x,
                        camera.view().y,
                        camera.view().z,
                        vec4(0.0, 0.0, 0.0, 1.0),
                    )
                    .invert()
                    .unwrap();
            }
        }

        let mut transforms = Vec::with_capacity(self.particles.len());
        let mut colors = Vec::with_capacity(self.particles.len());
        for p in &self.particles {
            transforms.push(p.pos);
            colors.push(p.color);
        }

        let instances = three_d::renderer::geometry::Instances {
            transformations: transforms,
            colors: Some(colors),
            ..Default::default()
        };
        self.renderable.geometry.set_instances(&instances);
        self.last_time = time;
    }
}

impl RenderableGeometry for ParticleEmitter {
    fn objects(&self, pass: RenderPass) -> Vec<&dyn Object> {
        if (self.participates_in_pass)(pass) {
            vec![&self.renderable]
        } else {
            vec![]
        }
    }

    fn geometries(&self, pass: RenderPass) -> Vec<&InstancedMesh> {
        if (self.participates_in_pass)(pass) {
            vec![&self.renderable.geometry]
        } else {
            vec![]
        }
    }

    fn prepare_frame(&mut self) {}

    fn finish_frame(&mut self, context: &Context) {}
}

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
                                let x_start =
                                    (x as f32 * edge_x).max(-half_length).min(half_length);
                                let x_end =
                                    ((x + 1) as f32 * edge_x).max(-half_length).min(half_length);

                                let y_start = (y as f32 * edge_y).max(-half_width).min(half_width);
                                let y_end =
                                    ((y + 1) as f32 * edge_y).max(-half_width).min(half_width);

                                let z_start =
                                    (z as f32 * edge_z).max(-half_height).min(half_height);
                                let z_end =
                                    ((z + 1) as f32 * edge_z).max(-half_height).min(half_height);

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

        // Apply the scale to the transforms.
        let scaled_pos: Vec<Mat4> = self
            .particles
            .iter()
            .map(|p| p.pos * Mat4::from_nonuniform_scale(p.scale.x, p.scale.y, p.scale.z))
            .collect();
        // println!("scaled_pos pose: {:?}", scaled_pos);

        let mut transforms = Vec::with_capacity(self.particles.len());
        let mut colors = Vec::with_capacity(self.particles.len());
        for i in 0..self.particles.len() {
            transforms.push(scaled_pos[i]);
            colors.push(self.particles[i].color);
        }

        let instances = three_d::renderer::geometry::Instances {
            transformations: transforms,
            colors: Some(colors),
            ..Default::default()
        };
        self.renderable.geometry.set_instances(&instances);
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

    fn geometries(&self, pass: RenderPass) -> Vec<&InstancedMesh> {
        if (self.participates_in_pass)(pass) {
            vec![&self.renderable.geometry]
        } else {
            vec![]
        }
    }

    fn prepare_frame(&mut self) {}

    fn finish_frame(&mut self, context: &Context) {}
}
