use crate::construct_render::instanced_entity::InstancedEntity;
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
    renderable: InstancedEntity<FireworksMaterial>,
    // particles: three_d::renderer::geometry::Particles,
    emit_period: f32,
    last_emit: f32,
    color: three_d::Color,
    fade_time: f32,
    particles: Vec<(Mat4, Color, f32)>,
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
        // let mut square = CpuMesh::circle(8);
        let mut square = CpuMesh::square();
        square.transform(&Mat4::from_scale(display.size)).unwrap();


        let z = FireworksMaterial{color: Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 254,
                }, fade: 1.0};
        // renderable.set_material(z);


        let mut renderable = InstancedEntity::<FireworksMaterial>::new(context, &square, z);

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
        Self {
            renderable: renderable,
            emit_period: 0.03,
            last_emit: -1000.0,
            fade_time: 1.0,
            particles: vec![],
            color,
        }
    }
}

impl RenderableEffect for ParticleEmitter {
    fn object(&self) -> Option<&dyn Object> {
        Some(self.renderable.object())
    }

    fn update(&mut self, camera: &Camera, entity_position: Matrix4<f32>, time: f32) {
        // Since our geometry is a square, we always want to view it from the same direction, nomatter how we change the camera.
        if (self.last_emit + self.emit_period) < time {
            self.particles.push((entity_position, self.color, time));
            self.last_emit = time;

            // Update the alphas in all colors.
            for (pos, color, spawn) in self.particles.iter_mut() {
                let age = time - *spawn;
                let ratio_of_age = (1.0 - (age / self.fade_time)).max(0.0) * 255.0;
                let alpha = ratio_of_age as u8;
                color.a = alpha;

                *pos = Mat4::from_translation(pos.w.truncate())
                    * Mat4::from_cols(
                        camera.view().x,
                        camera.view().y,
                        camera.view().z,
                        vec4(0.0, 0.0, 0.0, 1.0),
                    )
                    .invert()
                    .unwrap();
            }

            // drop all transparent things.
            self.particles = self
                .particles
                .iter()
                .filter(|(_p, color, _s)| color.a != 0)
                .map(|x| *x)
                .collect::<_>();

            let p = self
                .particles
                .iter()
                .map(|(p, c, s)| (p, c))
                .collect::<Vec<_>>();

            self.renderable.set_instances(&p[..]);
        }
    }
}
