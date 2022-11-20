use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct Pose {
    pub h: cgmath::Matrix4<f32>,
}

#[derive(Copy, Debug, Clone)]
pub struct PreTransform {
    pub h: cgmath::Matrix4<f32>,
}


macro_rules! create_transform_component {
    ($the_type:ty) => {
        impl $the_type {
            pub fn new() -> Self {
                Self {
                    h: cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3::new(0.0, 0.0, 0.0)),
                }
            }
            pub fn from_translation(v: cgmath::Vector3<f32>) -> Self {
                Self::from_mat4(cgmath::Matrix4::<f32>::from_translation(v))
            }
            pub fn from_mat4(h: cgmath::Matrix4::<f32>) -> Self {
                Self {
                    h
                }
            }
            pub fn transform(&self) -> &cgmath::Matrix4::<f32> {
                &self.h
            }
            pub fn transform_mut(&mut self) -> &mut cgmath::Matrix4::<f32> {
                &mut self.h
            }
        }
        impl Component for $the_type {}


        impl std::ops::Deref for $the_type {
            type Target = cgmath::Matrix4<f32>;
            fn deref(&self) -> &<Self as std::ops::Deref>::Target {
                &self.h
            }
        }

        impl From<cgmath::Matrix4<f32>> for $the_type {
            fn from(v: cgmath::Matrix4<f32>) -> Self {
                <$the_type>::from_mat4(v)
            }
        }

        impl Into<cgmath::Matrix4<f32>> for $the_type {
            fn into(self) -> cgmath::Matrix4<f32> {
                self.h
            }
        }

        impl std::ops::Mul<$the_type> for $the_type {
            type Output = $the_type;
            fn mul(self, v: $the_type) -> <Self as std::ops::Mul<$the_type>>::Output {
                <$the_type>::from_mat4(self.h * v.h)
            }
        }

    };
}
create_transform_component!(Pose);
create_transform_component!(PreTransform);

pub fn world_pose(world: &World, entity: &EntityId) -> Pose {
    let mut current_id = entity.clone();
    let mut current_pose = Pose::new();
    loop {
        let pose = world.component::<Pose>(&current_id);
        if let Some(pose) = pose {
                current_pose = *pose * current_pose;
        }
        let pre_pose = world.component::<PreTransform>(&current_id);
        if let Some(pre_pose) = pre_pose {
                current_pose = (pre_pose.transform() * current_pose.transform()).into();
        }
        if let Some(parent) = world
            .component::<super::parent::Parent>(&current_id)
        {
            current_id = parent.parent().clone();
        } else {
            break;
        }
    }
    current_pose
}
