use battleground_construct::util::cgmath::ToQuaternion;
use three_d::*;

/// Thin wrapper around [`InstancedMesh`] to use Mat4 and guarantee color & transforms in sync.
pub struct InstancedEntity {
    gm: three_d::renderer::object::Gm<three_d::renderer::geometry::InstancedMesh, PhysicalMaterial>,
    transforms: Vec<Mat4>,
    colors: Vec<Color>,
}

impl InstancedEntity {
    pub fn new(context: &Context, cpu_mesh: &CpuMesh) -> Self {
        let instances: three_d::renderer::geometry::Instances = Default::default();
        InstancedEntity {
            gm: Gm::new(
                InstancedMesh::new(context, &instances, cpu_mesh),
                three_d::renderer::material::PhysicalMaterial::new(
                    &context,
                    &CpuMaterial {
                        albedo: Color {
                            r: 255,
                            g: 255,
                            b: 255,
                            a: 255,
                        },
                        ..Default::default()
                    },
                ),
            ),
            transforms: vec![],
            colors: vec![],
        }
    }

    pub fn gm(
        &self,
    ) -> &three_d::renderer::object::Gm<three_d::renderer::geometry::InstancedMesh, PhysicalMaterial>
    {
        &self.gm
    }

    pub fn update_instances(&mut self) {
        let mut instances: three_d::renderer::geometry::Instances = Default::default();
        instances.translations = self
            .transforms
            .iter()
            .map(|m| m.w.truncate())
            .collect::<_>();

        // The transforms we have a homogeneous matrices, so the top left 3x3 is a rotation matrix.
        // We need to express that as a quaternion here.
        instances.rotations = Some(
            self.transforms
                .iter()
                .map(|m| m.to_quaternion())
                .collect::<_>(),
        );

        // Scaling is not done, this is ALWAYS done in the mesh itself, since all transforms are
        // homogeneous transforms.
        instances.colors = Some(self.colors.clone());
        self.gm.geometry.set_instances(&instances);
    }

    pub fn add(&mut self, transform: Mat4, color: Color) {
        self.transforms.push(transform);
        self.colors.push(color);
    }
}
