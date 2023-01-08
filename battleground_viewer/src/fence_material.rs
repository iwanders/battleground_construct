use three_d::*;

#[derive(Clone)]
pub struct FenceMaterial<'a> {
    pub color: Color,
    pub render_states: RenderStates,
    pub depth_texture: &'a DepthTexture2D,
}

impl<'a> FenceMaterial<'a> {
    pub fn new(depth_texture: &'a DepthTexture2D) -> Self {
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
            depth_texture,
        }
    }
}

impl Material for FenceMaterial<'_> {
    fn fragment_shader_source(&self, use_vertex_colors: bool, _lights: &[&dyn Light]) -> String {
        let mut shader = String::new();
        if use_vertex_colors {
            shader.push_str("#define USE_VERTEX_COLORS\nin vec4 col;\n");
        }
        shader.push_str(include_str!("shaders/fence_material.frag"));
        shader
    }
    fn use_uniforms(&self, program: &Program, camera: &Camera, _lights: &[&dyn Light]) {
        program.use_uniform("surfaceColor", self.color);
        program.use_uniform(
            "viewProjectionInverse",
            (camera.projection() * camera.view()).invert().unwrap(),
        );
        program.use_depth_texture("depthTexture", self.depth_texture);
    }
    fn render_states(&self) -> RenderStates {
        self.render_states
    }
    fn material_type(&self) -> MaterialType {
        MaterialType::Transparent
    }
}
