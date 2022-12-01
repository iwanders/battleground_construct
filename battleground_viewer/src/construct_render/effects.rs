use crate::construct_render::instanced_entity::InstancedEntity;
use crate::construct_render::util::ColorConvert;
use battleground_construct::display;
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


struct DestructorParticle {
    pos: Mat4,
    vel: Vec3,
    color: Color,
}


pub struct Deconstructor {
    renderable: InstancedEntity<three_d::renderer::material::PhysicalMaterial>,
    particles: Vec<DestructorParticle>,
}

impl Deconstructor {
    pub fn new(
        context: &Context,
        entity_position: Matrix4<f32>,
        time: f32,
        elements: &[display::primitives::Element],
    ) -> Self {
        /* now, here, we do the thing, we slice the primitives...
            How do we slice a cube?
            +------------------------
            |       |
            |   O   |
            |       |
            +-------+--------------
            |
            |
        */
        let edge_x = 0.05;
        let edge_y = 0.05;
        let edge_z = 0.05;
        let mut renderable =
            InstancedEntity::<three_d::renderer::material::PhysicalMaterial>::new_physical(
                &context,
                &CpuMesh::cube(),
            );

        let material = three_d::renderer::material::PhysicalMaterial::new(
            &context,
            &CpuMaterial {
                albedo: Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 128,
                },
                ..Default::default()
            },
        );
        renderable.gm_mut().material = material;
        let mut particles = vec![];

        for el in elements.iter() {
            match el.primitive {
                battleground_construct::display::primitives::Primitive::Cuboid(c) => {
                    // add the original!
                    let mut c = c;
                    c.width = 0.12;
                    let half_width = c.width / 2.0;
                    let half_length = c.length / 2.0;
                    let half_height = c.height / 2.0;
                    particles.push(DestructorParticle{
                        pos: entity_position * el.transform * Mat4::from_nonuniform_scale(half_width, half_length, half_height),
                        color: Color::new(0, 0, 255, 128),
                        vel: vec3(0.0, 0.0, 0.0),
                    });
                    // calculate the translations from the center of the cuboid.
                    let offset_x = c.width.rem_euclid(edge_x) / 2.0;
                    let offset_y = c.length.rem_euclid(edge_y) / 2.0;
                    let offset_z = c.height.rem_euclid(edge_z) / 2.0;
                    let chunks_x = (c.width / edge_x).ceil() as usize;
                    let chunks_y = (c.length / edge_y).ceil() as usize;
                    let chunks_z = (c.height / edge_z).ceil() as usize;
                    for x in 0..chunks_x {
                        for y in 0..chunks_y {
                            for z in 0..chunks_z {
                                /**/
                                let p = vec3(x as f32 * edge_x + offset_x - half_width, y as f32 * edge_y + offset_y - half_length, z as f32 * edge_z + offset_z - half_height);
                                let t = 0.001;
                                let p = p + vec3(x as f32 * t, y as f32 * t, z as f32 * t);
                                let sx = if (x == 0 || x == chunks_x - 1) { offset_x } else {edge_x / 2.0};
                                let sy = if (y == 0 || y == chunks_y - 1) { offset_y } else {edge_y / 2.0};
                                let sz = if (z == 0 || z == chunks_z - 1) { offset_z } else {edge_z / 2.0};
                                let transformation = Mat4::from_translation(p) * Mat4::from_nonuniform_scale(sx, sy, sz);
                                

                                particles.push(DestructorParticle{
                                    pos: entity_position * el.transform * transformation,
                                    color: Color::new(255, 0, 0, 30),
                                    vel: vec3(0.0, 0.0, 0.0),
                                });
                            }
                        }
                    }
                }
                battleground_construct::display::primitives::Primitive::Sphere(_) => todo!(),
                battleground_construct::display::primitives::Primitive::Cylinder(_) => todo!(),
            }
        }
        let p = particles
            .iter()
            .map(|p| (&p.pos, &p.color))
            .collect::<Vec<_>>();

        renderable.set_instances(&p);
        Deconstructor { renderable, particles }
    }
}

impl RenderableEffect for Deconstructor {
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
        // self.renderable.set_instances(&[(
            // &Mat4::from_translation(vec3(0.0, 0.0, 0.0)),
            // &Color::new_opaque(255, 0, 0),
        // )]);
    }
}
