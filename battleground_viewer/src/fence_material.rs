use three_d::*;

#[derive(Clone, Default)]
pub struct FenceMaterial {
    pub color: Color,
    pub render_states: RenderStates,
}

impl FenceMaterial {
    pub fn new() -> Self {
        Self {
            color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            render_states: RenderStates {
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        }
    }
}

impl Material for FenceMaterial {
    fn fragment_shader_source(&self, use_vertex_colors: bool, _lights: &[&dyn Light]) -> String {
        let mut shader = String::new();
        if use_vertex_colors {
            shader.push_str("#define USE_VERTEX_COLORS\nin vec4 col;\n");
        }
        shader.push_str(include_str!("shaders/fence_material.frag"));
        shader
    }
    fn use_uniforms(&self, program: &Program, _camera: &Camera, _lights: &[&dyn Light]) {
        program.use_uniform("surfaceColor", self.color);
    }
    fn render_states(&self) -> RenderStates {
        self.render_states
    }
    fn material_type(&self) -> MaterialType {
        MaterialType::Transparent
    }
}
