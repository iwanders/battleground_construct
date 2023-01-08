use crate::construct_render::render::RenderableGeometry;
use battleground_construct::display;
use three_d::*;

pub trait RetainedEffect: RenderableGeometry {
    fn update(
        &mut self,
        effect_type: &display::primitives::EffectType,
        camera: &Camera,
        entity_position: Matrix4<f32>,
        time: f32,
    );

    fn as_renderable_geometry(&self) -> &dyn RenderableGeometry;
    fn as_renderable_geometry_mut(&mut self) -> &mut dyn RenderableGeometry;
}
