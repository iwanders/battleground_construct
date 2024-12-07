use three_d::*;

/// Renderpasses that involve geometry in some way
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderPass {
    /// Render out shadow casters to shadow maps for all relevant lights
    ShadowMaps,
    /// All normal geometry that is present in the scene and
    BaseScene,
    /// Overlay on top of BaseScene
    BaseSceneOverlay,
    /// A depth buffer render for all non-emissives, so emissive glow is blocked by things in front of it
    NonGlowDepths,
    /// Geometry used to produce emissive glows
    GlowSources,
    /// Geometry that acts as a fence, indicating when intersects with it
    Fences,
}

/// Anything that is RenderableGeometry can articipate in the renderinepipeline by providing
/// renderable objects and geometry used for the different render passes
pub trait RenderableGeometry {
    /// Produces the objects to render for this render pass
    fn objects(&self, pass: RenderPass) -> Vec<&dyn Object>;

    /// Produces the geometries for this render pass.
    fn geometries(&self, pass: RenderPass) -> Vec<GeometryRef>;

    /// Prepares internals for a new frame.
    fn prepare_scene(&mut self, _context: &Context) {}

    /// Finishes up the frame, and performs necessary bookkeeping.
    fn finish_scene(&mut self, _context: &Context) {}
}

/// Work-around for three-d light's `generate_shadow_map` method signature
pub enum GeometryRef<'a> {
    InstancedMesh(&'a InstancedMesh),
    Mesh(&'a Mesh),
}

impl<'a> Geometry for GeometryRef<'a> {
    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        match self {
            GeometryRef::InstancedMesh(m) => m.render_with_material(material, camera, lights),
            GeometryRef::Mesh(m) => m.render_with_material(material, camera, lights),
        }
    }

    fn render_with_effect(
        &self,
        material: &dyn PostMaterial,
        camera: &Camera,
        lights: &[&dyn Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) {
        match self {
            GeometryRef::InstancedMesh(m) => {
                m.render_with_post_material(material, camera, lights, color_texture, depth_texture)
            }
            GeometryRef::Mesh(m) => {
                m.render_with_post_material(material, camera, lights, color_texture, depth_texture)
            }
        }
    }

    fn aabb(&self) -> AxisAlignedBoundingBox {
        match self {
            GeometryRef::InstancedMesh(m) => m.aabb(),
            GeometryRef::Mesh(m) => m.aabb(),
        }
    }
}
