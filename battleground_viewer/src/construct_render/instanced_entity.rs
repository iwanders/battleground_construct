use battleground_construct::util::cgmath::ToQuaternion;
use three_d::*;

/// Thin wrapper around [`InstancedMesh`] to use Mat4 and guarantee color & transforms in sync.
pub struct InstancedEntity<M: Material> {
    gm: three_d::renderer::object::Gm<three_d::renderer::geometry::InstancedMesh, M>,
    transforms: Vec<Mat4>,
    colors: Vec<Color>,
}
impl InstancedEntity<three_d::renderer::material::PhysicalMaterial> {
    pub fn new_physical(context: &Context, cpu_mesh: &CpuMesh) -> Self {
        let instances: three_d::renderer::geometry::Instances = Default::default();
        let material = three_d::renderer::material::PhysicalMaterial::new_opaque(
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
        );
        // material.albedo.a = 255;
        InstancedEntity::<three_d::renderer::material::PhysicalMaterial> {
            gm: Gm::new(InstancedMesh::new(context, &instances, cpu_mesh), material),
            transforms: vec![],
            colors: vec![],
        }
    }
}

impl InstancedEntity<three_d::renderer::material::ColorMaterial> {
    pub fn new_colored(context: &Context, cpu_mesh: &CpuMesh) -> Self {
        let instances: three_d::renderer::geometry::Instances = Default::default();
        let material = three_d::renderer::material::ColorMaterial::new_opaque(
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
        );
        // material.albedo.a = 255;
        InstancedEntity::<three_d::renderer::material::ColorMaterial> {
            gm: Gm::new(InstancedMesh::new(context, &instances, cpu_mesh), material),
            transforms: vec![],
            colors: vec![],
        }
    }
}

impl<M: Material> InstancedEntity<M> {
    pub fn new(context: &Context, cpu_mesh: &CpuMesh, material: M) -> Self {
        let instances: three_d::renderer::geometry::Instances = Default::default();
        InstancedEntity::<M> {
            gm: Gm::new(InstancedMesh::new(context, &instances, cpu_mesh), material),
            transforms: vec![],
            colors: vec![],
        }
    }

    pub fn gm(
        &self,
    ) -> &three_d::renderer::object::Gm<three_d::renderer::geometry::InstancedMesh, M> {
        &self.gm
    }

    pub fn object(&self) -> &dyn Object {
        &self.gm as &dyn Object
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

    pub fn set_instances(&mut self, instances: &[(&Mat4, &Color)]) {
        self.transforms.clear();
        self.colors.clear();
        for (p, c) in instances {
            self.transforms.push(**p);
            self.colors.push(**c);
        }
        self.update_instances();
    }

    pub fn set_lines(&mut self, lines: &[(Vec3, Vec3, f32, Color)]) {
        let mut translations = Vec::with_capacity(lines.len());
        let mut scales = Vec::with_capacity(lines.len());
        let mut rotations = Vec::with_capacity(lines.len());
        let mut colors = Vec::with_capacity(lines.len());

        for (p0, p1, width, c) in lines.iter() {
            translations.push(*p0);
            scales.push(vec3((*p0 - *p1).magnitude(), width / 2.0, width / 2.0));
            rotations.push(Quat::from_arc(
                vec3(1.0, 0.0, 0.0),
                (p1 - p0).normalize(),
                None,
            ));
            colors.push(*c);
        }
        let instances = three_d::renderer::geometry::Instances {
            translations,
            rotations: Some(rotations),
            scales: Some(scales),
            colors: Some(colors),
            ..Default::default()
        };
        self.gm.geometry.set_instances(&instances);
    }
}
