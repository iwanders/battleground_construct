use crate::construct_render::instanced_entity::InstancedEntity;
use crate::construct_render::util::ColorConvert;
use battleground_construct::display;
use battleground_construct::util::cgmath::prelude::*;
use rand::rngs::ThreadRng;
use rand::Rng;
use three_d::*;

pub trait RenderableEffect {
    fn object(&self) -> Option<&dyn Object>;
    fn update(
        &mut self,
        effect_type: &display::primitives::EffectType,
        camera: &Camera,
        entity_position: Matrix4<f32>,
        time: f32,
    );
}

struct Particle {
    pos: Mat4,
    vel: Vec3,
    color: Color,
    spawn_time: f32,
    expiry_time: f32,
}

impl Particle {
    pub fn new(pos: Mat4, vel: Vec3, color: Color, spawn_time: f32, expiry_time: f32) -> Self {
        Particle {
            pos,
            vel,
            color,
            spawn_time,
            expiry_time,
        }
    }

    pub fn expired(&self, time: f32) -> bool {
        self.expiry_time < time
    }
}

pub struct ParticleEmitter {
    renderable: InstancedEntity<three_d::renderer::material::ColorMaterial>,

    /// Keep track of the last update time, to integrate velocity.
    last_time: f32,

    /// Last spawn time.
    next_spawn_time: f32,

    /// Interval at which to spawn particles.
    spawn_interval: f32,
    spawn_jitter: f32,

    /// Lifetime of each particle.
    lifetime: f32,
    lifetime_jitter: f32, // gaussian

    /// Color of each particle.
    color: three_d::Color,

    // acceleration: Vec3,
    /// Initial velocity of each particle.
    velocity: Vec3,
    velocity_jitter: Vec3,

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
        _entity_position: Matrix4<f32>,
        time: f32,
        display: &display::primitives::ParticleType,
    ) -> Self {
        let mut p_color: Color;
        let p_size: f32;
        let lifetime = 0.4;
        let spawn_interval = 0.01;

        match display {
            display::primitives::ParticleType::BulletTrail { color, size } => {
                p_color = color.to_color();
                p_color.a = 128;
                p_size = *size;
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
            &context,
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

        let renderable = InstancedEntity::new(context, &square, material);

        Self {
            last_time: time,
            renderable: renderable,

            next_spawn_time: time,
            spawn_interval: spawn_interval,
            spawn_jitter: 0.00,
            lifetime: lifetime,
            lifetime_jitter: 0.0,

            velocity: vec3(0.0, 0.0, 0.0),
            velocity_jitter: vec3(0.1, 0.1, 0.1),

            fade_alpha_to_lifetime: true,
            face_camera: true,

            particles: vec![],
            color,

            rng: rand::thread_rng(),
        }
    }
}

impl RenderableEffect for ParticleEmitter {
    fn object(&self) -> Option<&dyn Object> {
        Some(self.renderable.object())
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
        let dt = self.last_time - time;

        // Drop particles that are expired.
        self.particles = self
            .particles
            .drain(..)
            .filter(|p| !p.expired(time))
            .collect::<_>();

        // Spawn new particles.
        while (self.next_spawn_time < time) && emitting {
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
            let vel = self.velocity
                + vec3(
                    v0 * self.velocity_jitter[0],
                    v1 * self.velocity_jitter[1],
                    v2 * self.velocity_jitter[2],
                );

            self.particles
                .push(Particle::new(pos, vel, color, spawn_time, expiry_time));
        }

        // Update position and alphas
        for particle in self.particles.iter_mut() {
            // Always update position.
            particle.pos.w = particle.pos.w + particle.vel.extend(0.0) * dt;

            if self.fade_alpha_to_lifetime {
                let ratio_of_age =
                    (time - particle.spawn_time) / (particle.expiry_time - particle.spawn_time);
                let alpha_scaled = (1.0 - ratio_of_age) * (self.color.a as f32);
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

        let p = self
            .particles
            .iter()
            .map(|p| (&p.pos, &p.color))
            .collect::<Vec<_>>();

        self.renderable.set_instances(&p[..]);
        self.last_time = time;
    }
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
    renderable: InstancedEntity<three_d::renderer::material::PhysicalMaterial>,
    particles: Vec<DestructorParticle>,

    /// Keep track of the last update time, to integrate velocity.
    last_time: f32,

    /// Max traveled to fully have faded.
    max_traveled: f32,
}

impl Deconstructor {
    pub fn new(
        context: &Context,
        entity_position: Matrix4<f32>,
        time: f32,
        elements: &[(display::primitives::Element, Twist<f32>)],
        impacts: &[(Mat4, f32)],
    ) -> Self {
        let edge_x = 0.05;
        let edge_y = edge_x;
        let edge_z = edge_x;
        let mut renderable =
            InstancedEntity::<three_d::renderer::material::PhysicalMaterial>::new_physical(
                &context,
                &CpuMesh::cube(),
            );

        let material = three_d::renderer::material::PhysicalMaterial::new_transparent(
            &context,
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
        renderable.gm_mut().material = material;
        let mut particles = vec![];

        let mut rng = rand::thread_rng();
        use rand_distr::StandardNormal;

        let mut rand_f32 = move || rng.sample::<f32, StandardNormal>(StandardNormal);

        let do_full_explosion = true;

        for (el, twist) in elements.iter() {
            match el.primitive {
                battleground_construct::display::primitives::Primitive::Cuboid(c) => {
                    let half_length = c.length / 2.0;
                    let half_width = c.width / 2.0;
                    let half_height = c.height / 2.0;

                    // calculate the translations from the center of the cuboid.
                    let chunks_x = (((c.length / 2.0) / edge_x) as isize) + 1;
                    let chunks_y = (((c.width / 2.0) / edge_y) as isize) + 1;
                    let chunks_z = (((c.height / 2.0) / edge_z) as isize) + 1;
                    let center_world = entity_position * el.transform;
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
                                let fragment_pos = Mat4::from_translation(p);

                                let fragment_world_pos =
                                    entity_position * el.transform * fragment_pos;

                                // Start velocity calculation, initialise with zero.
                                let mut vel = vec3(0.0, 0.0, 0.0);

                                // Add base body velocity.
                                vel = (twist.v * 1.0) + vel;

                                // Add outward from the body center.
                                let cube_world = fragment_world_pos;
                                let dir = cube_world.w.truncate() - center_world.w.truncate();
                                let pos = (fragment_world_pos).to_rotation_h();
                                if do_full_explosion {
                                    vel = vel + (dir.to_h() * pos).w.truncate() * 0.1;
                                }

                                // Add some random jitter, such that it looks prettier.
                                if do_full_explosion {
                                    vel = vel + vec3(rand_f32(), rand_f32(), rand_f32()) * 0.1;
                                }

                                // Then, add velocities away from the impacts.
                                for (impact_location, magnitude) in impacts.iter() {
                                    let p1 = impact_location.to_translation();
                                    let p0 = fragment_world_pos.to_translation();
                                    let rotation = Quat::from_arc(
                                        vec3(1.0, 0.0, 0.0),
                                        (p0 - p1).normalize(),
                                        None,
                                    );
                                    let d = (p1.distance2(p0)).sqrt();
                                    let mag = magnitude * (1.0 / (d * d));
                                    if do_full_explosion {
                                        vel += (rotation * vec3(1.0, 0.0, 0.0)) * mag;
                                    }
                                }

                                particles.push(DestructorParticle {
                                    pos: fragment_world_pos,
                                    color: Color::new(el.color.r, el.color.g, el.color.b, 128),
                                    vel,
                                    scale: vec3(sx, sy, sz),
                                    traveled: 0.0,
                                });
                            }
                        }
                    }
                }
                battleground_construct::display::primitives::Primitive::Sphere(_) => todo!(),
                battleground_construct::display::primitives::Primitive::Cylinder(_) => todo!(),
                battleground_construct::display::primitives::Primitive::Line(_) => todo!(),
            }
        }
        // let particles = vec![particles.pop().unwrap()];

        Deconstructor {
            last_time: time,
            renderable,
            particles,
            max_traveled: 10.0,
        }
    }
}

impl RenderableEffect for Deconstructor {
    fn object(&self) -> Option<&dyn Object> {
        Some(self.renderable.object())
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
            let rot = particle.pos.to_rotation_h();
            let before = particle.pos.to_translation();
            particle.pos.w = particle.pos.w + particle.vel.extend(0.0) * dt;
            let after = particle.pos.to_translation();
            let d = after.distance2(before).sqrt();
            particle.traveled += d;
            let ratio_of_lifetime = 1.0 - (particle.traveled / self.max_traveled).min(1.0);
            particle.color.a =
                std::cmp::min(particle.color.a, (255 as f32 * ratio_of_lifetime) as u8);

            let accel = -9.81 * 0.25;
            let gravity = vec3(0.0f32, 0.0, accel).to_h();
            particle.vel += (gravity * rot).w.truncate() * dt;
            if particle.pos.w[2] <= 0.0 {
                particle.vel[0] = particle.vel[0] * 0.5;
                particle.vel[1] = particle.vel[1] * 0.5;
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

        let p = (0..self.particles.len())
            .map(|i| (&scaled_pos[i], &self.particles[i].color))
            .collect::<Vec<_>>();

        self.renderable.set_instances(&p);
        self.last_time = time;
    }
}
