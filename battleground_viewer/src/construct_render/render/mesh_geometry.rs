use three_d::*;

use super::batching::*;
use super::*;

/// A geometry buffer for that accepts arbitrary meshes
pub struct MeshGeometry<M: Material + BatchMaterial> {
    participates_in_pass: fn(RenderPass) -> bool,
    buffer: Vec<(CpuMesh, Mat4, Color)>,
    meshes: Vec<Gm<InstancedMesh, M>>,
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

    fn geometries(&self, pass: RenderPass) -> Vec<&InstancedMesh> {
        if (self.participates_in_pass)(pass) {
            self.meshes.iter().map(|x| &x.geometry).collect()
        } else {
            vec![]
        }
    }

    fn prepare_scene(&mut self, _context: &Context) {
        self.buffer.clear();
        self.meshes.clear();
    }

    fn finish_scene(&mut self, context: &Context) {
        for (mesh, transform, color) in &self.buffer {
            let instanced = Gm::new(
                InstancedMesh::new(
                    context,
                    &Instances {
                        transformations: vec![*transform],
                        colors: Some(vec![*color]),
                        ..Default::default()
                    },
                    mesh,
                ),
                M::new_for_batch(
                    context,
                    BatchProperties::Basic {
                        // We can do this, because the material is not retained over frames
                        is_transparent: color.a < 255,
                    },
                ),
            );
            self.meshes.push(instanced);
        }
    }
}
