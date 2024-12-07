use three_d::*;

use three_d::core::prelude::Srgba as Color;

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
    // fn fragment_shader_source(&self, use_vertex_colors: bool, _lights: &[&dyn Light]) -> String {
    fn fragment_shader_source(&self, _lights: &[&dyn Light]) -> String {
        let mut shader = String::new();
        // if use_vertex_colors {
        // shader.push_str("#define USE_VERTEX_COLORS\nin vec4 col;\n");
        // }
        shader.push_str(include_str!("shaders/fence_material.frag"));
        shader
    }
    fn use_uniforms(&self, program: &Program, camera: &dyn Viewer, _lights: &[&dyn Light]) {
        let z = three_d::core::prelude::Vector4 {
            x: self.color.r,
            y: self.color.g,
            z: self.color.b,
            w: self.color.a,
        };
        program.use_uniform("surfaceColor", z);
        program.use_uniform(
            "viewProjectionInverse",
            (camera.projection() * camera.view()).invert().unwrap(),
        );
        program.use_depth_texture("depthTexture", self.depth_texture);
        // program.use_vertex_attribute("color", &self.colors);
        // println!("{:?}", self.color);
    }
    fn render_states(&self) -> RenderStates {
        self.render_states
    }
    fn material_type(&self) -> MaterialType {
        MaterialType::Transparent
    }
    fn id(&self) -> EffectMaterialId {
        three_d::renderer::EffectMaterialId(0x0011)
    }
}

pub struct FenceEffect {}

impl Effect for FenceEffect {
    fn fragment_shader_source(
        &self,
        _lights: &[&dyn Light],
        _color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) -> String {
        let mut shader = String::new();
        // if use_vertex_colors {
        shader.push_str("#define USE_VERTEX_COLORS\nin vec4 col;\n");
        // }
        shader.push_str(include_str!("shaders/fence_material.frag"));
        shader
    }
    fn id(
        &self,
        color_texture: Option<ColorTexture<'_>>,
        depth_texture: Option<DepthTexture<'_>>,
    ) -> EffectMaterialId {
        three_d::renderer::EffectMaterialId(0x0011)
    }

    fn use_uniforms(
        &self,
        program: &Program,
        camera: &dyn Viewer,
        _lights: &[&dyn Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) {
        // program.use_vertex_attribute("color", color_texture.unwrap());
        // program.use_texture("emissive_buffer", color_texture.unwrap());
        // color_texture.unwrap().use_uniforms(program);
    }

    fn render_states(&self) -> RenderStates {
        RenderStates {
            write_mask: WriteMask::COLOR,
            blend: Blend::TRANSPARENCY,
            ..Default::default()
        }
    }
}
