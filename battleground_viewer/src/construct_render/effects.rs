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
    renderable: three_d::Gm<ParticleSystem, FireworksMaterial>,
}

impl ParticleEmitter {
    pub fn new(
        context: &Context,
        entity_position: Matrix4<f32>,
        time: f32,
        display: &display::primitives::ParticleEmitter,
    ) -> Self {
        println!("New prticles");
        let color = Color::new_opaque(255, 255, 178);
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
        let mut fireworks = Gm::new(particles, fireworks_material);
        fireworks.time = time;
        let mut rng = rand::thread_rng();

        let explosion_speed = 0.5;
        let start_position = entity_position.w.truncate();
        let start_positions = (0..300).map(|_| start_position).collect();
        let colors = Some(
            (0..300)
                .map(|_| {
                    Color::new_opaque(
                        (rng.gen::<f32>() * 100.0 - 50.0) as u8,
                        (rng.gen::<f32>() * 100.0 - 50.0) as u8,
                        (rng.gen::<f32>() * 100.0 - 50.0) as u8,
                    )
                })
                .collect(),
        );
        let R = 0.5;
        let mut start_velocities = Vec::new();
        for _ in 0..300 {
            let theta = rng.gen::<f32>() * 2.0 - 1.0;
            let phi = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
            let explosion_direction = vec3(
                R * theta.acos().sin() * phi.cos(),
                R * theta.acos().sin() * phi.sin(),
                R * theta,
            );
            start_velocities
                .push((rng.gen::<f32>() * 0.2 + 0.9) * explosion_speed * explosion_direction);
        }
        fireworks.set_particles(&Particles {
            start_positions,
            start_velocities,
            colors,
            ..Default::default()
        });

        Self {
            renderable: fireworks,
        }
    }
}

impl RenderableEffect for ParticleEmitter {
    fn object(&self) -> Option<&dyn Object> {
        Some(&self.renderable as &dyn Object)
    }

    fn update(&mut self, camera: &Camera, entity_position: Matrix4<f32>, time: f32) {
        // Since our geometry is a square, we always want to view it from the same direction, nomatter how we change the camera.

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
        let f = self.renderable.time / 30.0;
        self.renderable.material.fade = 1.0 - f * f * f * f;
        self.renderable.time = time;
    }
}

/*


        let explosion_speed = 0.5;
        let explosion_time = 3.0;
        let colors = [
            Color::new_opaque(255, 255, 178),
            Color::new_opaque(255, 51, 25),
            Color::new_opaque(51, 102, 51),
            Color::new_opaque(127, 127, 204),
            Color::new_opaque(217, 23, 51),
            Color::new_opaque(250, 237, 38),
            Color::new_opaque(76, 237, 38),
            Color::new_opaque(40, 178, 222),
        ];
        // let mut square = CpuMesh::circle(8);
        // let mut square = CpuMesh::cube();
        let mut square = CpuMesh::square();
        square.transform(&Mat4::from_scale(0.05)).unwrap();
        let particles = ParticleSystem::new(&self.context, &Particles::default(), &square);
        let fireworks_material = FireworksMaterial {
            color: colors[0],
            fade: 0.0,
        };
        let mut fireworks = Gm::new(particles, fireworks_material);


        println!("fireworks: {:?}", fireworks.acceleration);
        fireworks.time = explosion_time + 100.0;
        fireworks.acceleration = vec3(0.0, 0.0, 0.0);

        let mut color_index = 0;
*/

/*


            //----------------------------------------------------------------
            let radius = 0.1;
            let elapsed_time = (frame_input.elapsed_time * 0.001) as f32;
            fireworks.time += elapsed_time;
            if fireworks.time > explosion_time {
                color_index = (color_index + 1) % colors.len();
                fireworks.material.color = colors[color_index];
                fireworks.time = 0.0;
                let start_position = vec3(-1.0, 0.0, 1.0);

                let start_positions = (0..300).map(|_| start_position).collect();
                let colors = Some(
                    (0..300)
                        .map(|_| {
                            Color::new_opaque(
                                (rng.gen::<f32>() * 100.0 - 50.0) as u8,
                                (rng.gen::<f32>() * 100.0 - 50.0) as u8,
                                (rng.gen::<f32>() * 100.0 - 50.0) as u8,
                            )
                        })
                        .collect(),
                );
                let R = 0.5;
                let mut start_velocities = Vec::new();
                for _ in 0..300 {
                    let theta = rng.gen::<f32>() * 2.0 - 1.0;
                    let phi = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                    let explosion_direction = vec3(
                        R * theta.acos().sin() * phi.cos(),
                        R * theta.acos().sin() * phi.sin(),
                        R * theta,
                    );
                    start_velocities.push(
                        (rng.gen::<f32>() * 0.2 + 0.9) * explosion_speed * explosion_direction,
                    );
                }
                fireworks.set_particles(&Particles {
                    start_positions,
                    start_velocities,
                    colors,
                    ..Default::default()
                });
            }

            let f = fireworks.time / explosion_time.max(0.0);
            fireworks.material.fade = 1.0 - f * f * f * f;
            // Since our geometry is a square, we always want to view it from the same direction, nomatter how we change the camera.

            fireworks.set_transformation(
                Mat4::from_cols(
                    self.camera.view().x,
                    self.camera.view().y,
                    self.camera.view().z,
                    vec4(0.0, 0.0, 0.0, 1.0),
                )
                .invert()
                .unwrap(),
            );/*
            */

*/
