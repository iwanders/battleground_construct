use battleground_construct::display;
use battleground_construct::display::EffectId;
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

pub struct ParticleEmitter {
    // renderable: three_d::Gm<ParticleSystem, FireworksMaterial>,
    renderable: three_d::Gm<ParticleSystem, three_d::PhysicalMaterial>,
    particles: three_d::renderer::geometry::Particles,
    emit_period: f32,
    last_emit: f32,
    color: three_d::Color,
    fade_time: f32,
    spawn_times: Vec<f32>,
    
}

impl ParticleEmitter {
    pub fn new(
        context: &Context,
        entity_position: Matrix4<f32>,
        time: f32,
        display: &display::primitives::ParticleEmitter,
    ) -> Self {
        println!("New prticles");
        let color = Color {
            r: display.color.r,
            g: display.color.g,
            b: display.color.b,
            a: display.color.a,
        };
        // let mut square = CpuMesh::circle(8);
        // let mut square = CpuMesh::cube();
        let mut square = CpuMesh::square();
        square.transform(&Mat4::from_scale(0.05)).unwrap();
        let mut particles = ParticleSystem::new(context, &Particles::default(), &square);
        particles.acceleration = vec3(0.0, 0.0, 0.0);
        let fireworks_material = FireworksMaterial {
            color: color,
            fade: 0.0,
        };

        let material = three_d::renderer::material::PhysicalMaterial::new(
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
        let mut fireworks = Gm::new(particles, material);
        fireworks.time = time;

        let mut particles : three_d::renderer::geometry::Particles = Default::default();
        particles.start_positions.push(vec3(0.0, 0.0, 0.0));
        particles.start_velocities.push(vec3(0.0, 0.0, 0.0));
        particles.colors = Some(vec![color]);
        let spawn_times = vec![time];
        Self {
            renderable: fireworks,
            emit_period: 0.1,
            last_emit: -1000.0,
            particles,
            fade_time: 1.0,
            spawn_times,
            color
        }
    }
}

impl RenderableEffect for ParticleEmitter {
    fn object(&self) -> Option<&dyn Object> {
        Some(&self.renderable as &dyn Object)
    }

    fn update(&mut self, camera: &Camera, entity_position: Matrix4<f32>, time: f32) {
        // Since our geometry is a square, we always want to view it from the same direction, nomatter how we change the camera.
        if (self.last_emit + self.emit_period) < time {
            self.particles.start_positions.push(entity_position.w.truncate());
            self.particles.start_velocities.push(vec3(0.0, 0.0, 0.0));
            self.particles.colors.as_mut().unwrap().push(self.color);
            self.spawn_times.push(time);
            self.last_emit = time;

            // Update the alphas in all colors.
            for (color, spawn) in self.particles.colors.as_mut().unwrap().iter_mut().zip(self.spawn_times.iter()) {
                let age = time - spawn;
                let ratio_of_age = (1.0 - (age / self.fade_time)).max(0.0) * 255.0;
                let alpha = ratio_of_age as u8;
                println!("Alpha: {alpha}");
                color.a = alpha;
            }

            self.renderable.set_particles(&self.particles);
        }
        // fireworks.set_particles(&Particles {
            // start_positions,
            // start_velocities,
            // colors,
            // ..Default::default()
        // });
        self.renderable.set_transformation(
            Mat4::from_cols(
                camera.view().x,
                camera.view().y,
                camera.view().z,
                vec4(0.0, 0.0, 0.0, 1.0),
            )
            .invert()
            .unwrap(),
        );
        // let f = self.renderable.time / 30.0;
        // self.renderable.material.fade = 1.0 - f * f * f * f;
        // self.renderable.material.fade = 1.0;
        self.renderable.time = time;
    }
}

