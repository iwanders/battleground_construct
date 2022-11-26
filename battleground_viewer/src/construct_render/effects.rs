use crate::construct_render::instanced_entity::InstancedEntity;
use battleground_construct::display;
use battleground_construct::display::EffectId;
use rand::rngs::ThreadRng;
use rand::Rng;
use three_d::*;

pub trait RenderableEffect {
    fn object(&self) -> Option<&dyn Object>;
    fn update(&mut self, camera: &Camera, entity_position: Matrix4<f32>, time: f32);
}

#[derive(Clone)]
struct FireworksMaterial {
    pub color: Color,
    pub fade: f32,
}

impl Material for FireworksMaterial {
    fn fragment_shader_source(&self, _use_vertex_colors: bool, _lights: &[&dyn Light]) -> String {
        include_str!("shaders/particles.frag").to_string()
    }
    fn use_uniforms(&self, program: &Program, _camera: &Camera, _lights: &[&dyn Light]) {
        program.use_uniform("color", self.color);
        program.use_uniform("fade", self.fade);
    }
    fn render_states(&self) -> RenderStates {
        RenderStates {
            cull: Cull::Back,
            blend: Blend::Enabled {
                rgb_equation: BlendEquationType::Add,
                alpha_equation: BlendEquationType::Add,
                source_rgb_multiplier: BlendMultiplierType::SrcAlpha,
                source_alpha_multiplier: BlendMultiplierType::Zero,
                destination_rgb_multiplier: BlendMultiplierType::One,
                destination_alpha_multiplier: BlendMultiplierType::One,
            },
            depth_test: DepthTest::LessOrEqual,
            write_mask: WriteMask::COLOR,
        }
    }
    fn material_type(&self) -> MaterialType {
        MaterialType::Transparent
    }
}

type LifeTime = f32;

struct Particle {
    pos: Mat4,
    color: Color,
    spawn_time: f32,
    expiry_time: f32,
}

impl Particle {
    pub fn new(pos: Mat4, color: Color, spawn_time: f32, expiry_time: f32) -> Self {
        Particle {
            pos,
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
    renderable: InstancedEntity<PhysicalMaterial>,

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
        display: &display::primitives::ParticleEmitter,
    ) -> Self {
        let color = Color {
            r: display.color.r,
            g: display.color.g,
            b: display.color.b,
            a: display.color.a,
        };
        let mut square = CpuMesh::circle(8);
        // let mut square = CpuMesh::square();
        square.transform(&Mat4::from_scale(display.size)).unwrap();

        let z = FireworksMaterial {
            color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 254,
            },
            fade: 1.0,
        };
        // renderable.set_material(z);

        let mut material = three_d::renderer::material::PhysicalMaterial::new(
            &context,
            &CpuMaterial {
                albedo: Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 254,
                },
                ..Default::default()
            },
        );

        let mut renderable = InstancedEntity::new(context, &square, material);

        Self {
            renderable: renderable,

            next_spawn_time: 0.0,
            spawn_interval: 0.1,
            spawn_jitter: 0.01,
            lifetime: 1.0,
            lifetime_jitter: 0.3,

            velocity: vec3(0.0, 0.0, 0.0),

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

    fn update(&mut self, camera: &Camera, entity_position: Matrix4<f32>, time: f32) {
        // Spawn new particles.
        while (self.next_spawn_time < time) {
            use rand_distr::StandardNormal;
            let spawn_val: f32 = self.rng.sample(StandardNormal);
            self.next_spawn_time += self.spawn_interval + spawn_val * self.spawn_jitter;
            // spawn particle.
            let pos = entity_position;
            let color = self.color;
            let spawn_time = time;
            let lifetime_val: f32 = self.rng.sample(StandardNormal);
            let expiry_time = time + self.lifetime + self.lifetime_jitter * lifetime_val;
            self.particles
                .push(Particle::new(pos, color, spawn_time, expiry_time));
        }

        // Update position and alphas
        for particle in self.particles.iter_mut() {
            if self.fade_alpha_to_lifetime {
                let ratio_of_age =
                    (time - particle.spawn_time) / (particle.expiry_time - particle.spawn_time);
                let ratio_of_age = (1.0 - (ratio_of_age).max(0.0)) * 255.0;
                let alpha = ratio_of_age as u8;
                particle.color.a = alpha;
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

        // drop all transparent things.
        self.particles = self
            .particles
            .drain(..)
            .filter(|p| !p.expired(time))
            .collect::<_>();

        let p = self
            .particles
            .iter()
            .map(|p| (&p.pos, &p.color))
            .collect::<Vec<_>>();

        self.renderable.set_instances(&p[..]);
    }
}
