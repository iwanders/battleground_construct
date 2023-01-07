use three_d::*;

use super::batching::*;
use super::*;

/// A geometry buffer for that accepts arbitrary meshes
pub struct MeshGeometry<M: Material + BatchMaterial> {
    participates_in_pass: fn(RenderPass) -> bool,
    buffer: Vec<(CpuMesh, Mat4, Color)>,
    meshes: Vec<Gm<Mesh, M>>,
}

impl<M: Material + BatchMaterial> MeshGeometry<M> {
    pub fn new(participates_in_pass: fn(RenderPass) -> bool) -> Self {
        Self {
            participates_in_pass,
            buffer: Default::default(),
            meshes: Default::default(),
        }
    }

    pub fn add_mesh(&mut self, mesh: &CpuMesh, transform: Mat4, color: Color) {
        self.buffer.push((mesh.clone(), transform, color));
    }
}

impl<M: Material + BatchMaterial> RenderableGeometry for MeshGeometry<M> {
    fn objects(&self, pass: RenderPass) -> Vec<&dyn Object> {
        if (self.participates_in_pass)(pass) {
            self.meshes.iter().map(|x| x as &dyn Object).collect()
        } else {
            vec![]
        }
    }

    fn geometries(&self, pass: RenderPass) -> Vec<GeometryRef> {
        if (self.participates_in_pass)(pass) {
            self.meshes
                .iter()
                .map(|x| GeometryRef::Mesh(&x.geometry))
                .collect()
        } else {
            vec![]
        }
    }

    fn prepare_scene(&mut self, _context: &Context) {
        self.buffer.clear();
        self.meshes.clear();
    }

    fn finish_scene(&mut self, context: &Context) {
        for (cpu_mesh, transform, color) in &self.buffer {
            let mut mesh = Mesh::new(context, cpu_mesh);
            mesh.set_transformation(*transform);
            let gm = Gm::new(
                mesh,
                M::new_for_batch_colored(
                    context,
                    BatchProperties::Basic {
                        // We can do this, because the material is not retained over frames
                        is_transparent: color.a < 255,
                    },
                    *color,
                ),
            );
            self.meshes.push(gm);
        }
    }
}
