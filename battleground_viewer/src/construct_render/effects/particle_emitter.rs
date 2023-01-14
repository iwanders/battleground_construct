use crate::construct_render::render::{GeometryRef, RenderPass, RenderableGeometry};
use crate::construct_render::util::ColorConvert;
use battleground_construct::display;
use battleground_construct::util::cgmath::prelude::*;
use rand::rngs::ThreadRng;
use rand::Rng;
use three_d::*;

use super::RetainedEffect;

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

    /// Initial velocity of each particle.
    velocity: Vec3,
    velocity_jitter: Vec3,
    global_acceleration: Option<Vec3>,
    drag: Option<Vec3>,

    velocity_clamp_x: Option<(f32, f32)>,
    velocity_clamp_z: Option<(f32, f32)>,

    reflect_on_floor: bool,

    /// Whether to fade the alpha to the lifetime.
    fade_alpha_to_lifetime: bool,

    /// Make particles always face the camera
    face_camera: bool,
    view_matrix: Mat4,

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
        let mut global_acceleration = None;
        let mut drag = None;
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

            display::primitives::ParticleType::Firework { radius, color } => {
                p_size = 0.03;
                global_acceleration = Some(vec3(0.0, 0.0, -9.81));
                fn color_to_tuple(
                    color: &battleground_construct::display::Color,
                    change: f32,
                ) -> (f32, f32, f32, f32) {
                    (
                        (color.r as f32 + change).clamp(0.0, 1.0),
                        (color.g as f32 + change).clamp(0.0, 1.0),
                        (color.b as f32 + change).clamp(0.0, 1.0),
                        0.25,
                    )
                }
                let color_yellow = color_to_tuple(color, 0.1); // pretty yellow.
                let color_red = color_to_tuple(color, -0.1); // More light
                let c_interp = |p: f32, c1: (f32, f32, f32, f32), c2: (f32, f32, f32, f32)| {
                    let r = ((c1.0 * p + c2.0 * (1.0 - p)) * 255.0) as u8;
                    let g = ((c1.1 * p + c2.1 * (1.0 - p)) * 255.0) as u8;
                    let b = ((c1.2 * p + c2.2 * (1.0 - p)) * 255.0) as u8;
                    let a = ((c1.3 * p + c2.3 * (1.0 - p)) * 255.0) as u8;
                    Color { r, g, b, a }
                };
                let lifetime_jitter = 0.3;
                let lifetime = 0.8;
                drag = Some(vec3(5.0, 5.0, 5.0));
                let velocity_jitter = vec3(1.5, 1.5, 1.5) * *radius;
                for _i in 0..1500 {
                    use rand_distr::StandardNormal;
                    // let spawn_val: f32 = rng.sample(StandardNormal);
                    let color_val: f32 = rng.gen();
                    // spawn particle.
                    let pos = entity_position;
                    let color = c_interp(color_val, color_yellow, color_red);
                    let lifetime_val: f32 = rng.sample(StandardNormal);
                    let expiry_time = time + lifetime + lifetime_jitter * lifetime_val;

                    let v0: f32 = rng.sample(StandardNormal);
                    let v1: f32 = rng.sample(StandardNormal);
                    let v2: f32 = rng.sample(StandardNormal);
                    let vel = vec3(
                        v0 * velocity_jitter[0],
                        v1 * velocity_jitter[1],
                        v2 * velocity_jitter[2] + 0.5 * *radius,
                    );
                    let expiry_distance = 100.0;
                    initial_particles.push(Particle::new(
                        pos,
                        vel,
                        color,
                        time,
                        expiry_time,
                        expiry_distance,
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
            global_acceleration,
            drag,

            reflect_on_floor,

            fade_alpha_to_lifetime: true,
            face_camera: true,
            view_matrix: Mat4::identity(),

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
            // Update velocity with acceleration if that's self.
            if let Some(accel) = self.global_acceleration {
                let gravity = accel.to_h();
                let rot = particle.pos.to_rotation_h();
                particle.vel += (gravity * rot).w.truncate() * dt;
            }

            if let Some(drag) = self.drag {
                particle.vel.x -= particle.vel.x * drag.x * dt;
                particle.vel.y -= particle.vel.y * drag.y * dt;
                particle.vel.z -= particle.vel.z * drag.z * dt;
            }

            // Always update position.
            let before = particle.pos.to_translation();
            particle.pos.w += particle.vel.extend(0.0) * dt;
            let after = particle.pos.to_translation();
            let d = after.distance2(before).sqrt();
            particle.distance += d;

            if self.reflect_on_floor && particle.pos.w.z < 0.0 {
                if particle.vel.z < 0.0 {
                    particle.vel.z = particle.vel.z.abs();
                }
                particle.pos.w.z = 0.0;
            }

            if self.fade_alpha_to_lifetime {
                let ratio_of_age =
                    (time - particle.spawn_time) / (particle.expiry_time - particle.spawn_time);
                let ratio_of_distance = (particle.distance / particle.expiry_distance).min(1.0);
                let max_ratio = ratio_of_distance.max(ratio_of_age);
                let alpha_scaled = (1.0 - max_ratio) * (self.color.a as f32);
                particle.color.a = alpha_scaled as u8;
            }
        }

        if self.face_camera {
            self.view_matrix = *camera.view();
        }
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

    fn geometries(&self, pass: RenderPass) -> Vec<GeometryRef> {
        if (self.participates_in_pass)(pass) {
            vec![GeometryRef::InstancedMesh(&self.renderable.geometry)]
        } else {
            vec![]
        }
    }

    fn finish_scene(&mut self, _context: &Context) {
        let (transforms, colors) = if self.face_camera {
            let inv_view_matrix = Mat4::from_cols(
                self.view_matrix.x,
                self.view_matrix.y,
                self.view_matrix.z,
                vec4(0.0, 0.0, 0.0, 1.0),
            )
            .invert()
            .unwrap();
            self.particles
                .iter()
                .map(|p| {
                    (
                        Mat4::from_translation(p.pos.w.truncate()) * inv_view_matrix,
                        p.color,
                    )
                })
                .unzip()
        } else {
            self.particles.iter().map(|p| (p.pos, p.color)).unzip()
        };
        self.renderable.geometry.set_instances(&Instances {
            transformations: transforms,
            colors: Some(colors),
            ..Default::default()
        });
    }
}
