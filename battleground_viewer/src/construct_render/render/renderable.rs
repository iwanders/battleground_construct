use three_d::*;

/// Renderpasses that involve geometry in some way
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RenderPass {
    /// Render out shadow casters to shadow maps for all relevant lights
    ShadowMaps,
    /// All normal geometry that is present in the scene and
    BaseScene,
    /// A depth buffer render for all non-emissives, so emissive glow is blocked by things in front of it
    NonGlowDepths,
    /// Geometry used to produce emissive glows
    GlowSources,
    /// Geometry that acts as a fence, indicating when intersects with it
    Fences,
}

pub trait RenderableGeometry {
    /// Produces the objects to render for this render pass
    fn objects(&self, pass: RenderPass) -> Vec<&dyn Object>;

    /// Produces the geometries for this render pass.
    fn geometries(&self, pass: RenderPass) -> Vec<&InstancedMesh>;

    /// Prepares internals for a new frame.
    fn prepare_scene(&mut self, _context: &Context) {}

    /// Finishes up the frame, and performs necessary bookkeeping.
    fn finish_scene(&mut self, _context: &Context) {}
}
