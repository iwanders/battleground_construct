use engine::prelude::*;

/// Velocity expressed in body frame (in all cases).
#[derive(Copy, Debug, Clone)]
pub struct Velocity {
    /// Translation component.
    pub v: cgmath::Vector3<f32>,
    /// Rotation component.
    pub w: cgmath::Vector3<f32>,
}

macro_rules! create_velocity_implementation {
    ($the_type:ty) => {
        impl $the_type {
            pub fn new() -> Self {
                Self {
                    v: cgmath::Vector3::new(0.0, 0.0, 0.0),
                    w: cgmath::Vector3::new(0.0, 0.0, 0.0),
                }
            }
            pub fn from_se2(x: f32, y: f32, yaw: f32) -> Self {
                Self {
                    v: cgmath::Vector3::new(x, y, 0.0),
                    w: cgmath::Vector3::new(0.0, 0.0, yaw),
                }
            }

            pub fn from_velocities(v: cgmath::Vector3<f32>, w: cgmath::Vector3<f32>) -> Self {
                Self { v, w }
            }

            pub fn integrate(&self, dt: f32) -> cgmath::Matrix4<f32> {
                (cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3::new(
                    self.v[0] * dt,
                    self.v[1] * dt,
                    self.v[2] * dt,
                )) * cgmath::Matrix4::<f32>::from_angle_x(cgmath::Rad(self.w[0]) * dt)
                    * cgmath::Matrix4::<f32>::from_angle_y(cgmath::Rad(self.w[1]) * dt)
                    * cgmath::Matrix4::<f32>::from_angle_z(cgmath::Rad(self.w[2]) * dt))
            }

            pub fn to_twist(&self) -> crate::util::cgmath::Twist<f32> {
                crate::util::cgmath::Twist::<f32>::new(self.v, self.w)
            }

            pub fn integrate_pose(&self, pose: &super::pose::Pose, dt: f32) -> super::pose::Pose {
                let res = (pose.h * self.integrate(dt));
                // return res.into();

                // Re-orthogonalize the rotation part of this matrix.
                //https://stackoverflow.com/a/23082112
                // x_new = 0.5*(3-dot(x_ort,x_ort))*x_ort
                // y_new = 0.5*(3-dot(y_ort,y_ort))*y_ort
                // z_new = 0.5*(3-dot(z_ort,z_ort))*z_ort
                use cgmath::InnerSpace;

                let x_ort = res.x.truncate();
                let y_ort = res.y.truncate();
                let z_ort = res.z.truncate();
                let c0 = (0.5 * (3.0 - x_ort.dot(x_ort)) * x_ort);
                let c1 = (0.5 * (3.0 - y_ort.dot(y_ort)) * y_ort);
                let c2 = (0.5 * (3.0 - z_ort.dot(z_ort)) * z_ort);

                // Finally, re-normalize the matrix as well.
                use cgmath::SquareMatrix;
                let m3 = cgmath::Matrix3::<f32>::from_cols(c0, c1, c2);
                let det = m3.determinant();
                let c0 = (1.0 / det) * c0;
                let c1 = (1.0 / det) * c1;
                let c2 = (1.0 / det) * c2;

                let c3 = res.w;

                // Finally, reconstruct the transform matrix.
                cgmath::Matrix4::<f32>::from_cols(
                    c0.extend(0.0),
                    c1.extend(0.0),
                    c2.extend(0.0),
                    c3,
                )
                .into()
            }
            pub fn integrate_global_pose(
                &self,
                pose: &super::pose::Pose,
                dt: f32,
            ) -> super::pose::Pose {
                (self.integrate(dt) * pose.h).into()
            }
        }
        impl Component for $the_type {}

        /*
        impl Into<cgmath::Matrix4<f32>> for $the_type {
            fn into(self) -> cgmath::Matrix4<f32> {
                use crate::util::cgmath::ToCross;
                let s = self.w.to_cross();
                cgmath::Matrix4::<f32>::from_cols(
                    s.x.extend(0.0),
                    s.y.extend(0.0),
                    s.z.extend(0.0),
                    self.v.extend(0.0),
                )
            }
        }*/
        impl Into<$the_type> for crate::util::cgmath::Twist<f32> {
            fn into(self) -> $the_type {
                <$the_type>::from_velocities(self.v, self.w)
            }
        }
    };
}
create_velocity_implementation!(Velocity);

// pub use world_velocity_linear as world_velocity;
pub use world_velocity_adjoint as world_velocity;

pub fn world_velocity_adjoint(world: &World, entity: EntityId) -> Velocity {
    let linear = world_velocity_linear(world, entity);
    // use crate::components::pose::world_pose;
    use crate::components::pose::Pose;
    use crate::components::pose::PreTransform;
    use crate::util::cgmath::prelude::*;

    let mut current_id = entity.clone();
    let mut current_velocity = Velocity::new().to_twist();
    let mut current_pose = Pose::new();
    let mut last_pose = *Pose::new().transform();
    /*
        Changing frame of a twist.
        twist = Adjoint(Frame) * frame_twist.

        Order of coordinate frames per entity is
        parent -> pre_transform -> pose -> child

        Pose on the entity is calculated by integrating the velocity.
        Velocity is local frame. Pose is the offset to the next frame.
    */
    loop {
        // println!("Current id: {current_id:?}");
        // T^ib_a, motion of body a with respect to body b, expressed in psi_i.
        // H^a_b, H_to_a_from_b, is H from B to A.

        // Let Pose be frame c.
        // Let PreTransform be frame p.

        // Pose is current frame in pre-transform frame.
        // So this is H^c_p
        let pose_t = if let Some(pose) = world.component::<Pose>(current_id) {
            // println!("Found pose for  {current_id:?}");
            *pose.transform()
        } else {
            *Pose::new().transform()
        };
        last_pose = pose_t;
        // The previous velocity needs to be lifted to the current frame, using the pose.
        current_velocity = pose_t.to_adjoint() * (current_velocity);

        // Not velocity of current frame, expressed in current frame. That doesn't make any sense.
        // That's always zero.
        // It must be current frame, relative to pre-transform, expressed in pre-transform.
        // T^pp_c
        let vel_t = if let Some(vel) = world.component::<Velocity>(current_id) {
            // println!("Found vel for  {current_id:?}");
            vel.to_twist()
        } else {
            Velocity::new().to_twist()
        };
        let vel_t = Twist::new(-vel_t.v, -vel_t.w); // negate the velocity, because we are walking upwards in the tree.
                                                    // Ah, that's the problem, if the Velocity is provided, the Pose will be calculated from
                                                    // it. If the Velocity is not provided, Pose may be used to perform a static transform.
        current_pose = (pose_t * *current_pose).into();

        // Now, we can add the current velocity to this local velocity, getting us the total
        // velocity.
        current_velocity = current_velocity + vel_t;
        // println!("  current_velocity: {current_velocity:?}");
        // println!("  vel_t: {vel_t:?}");
        // println!("  combined_vel: {combined_vel:?}");
        // println!("  pose_t: {pose_t:?}");
        // println!("  pose_t adjoint: {:?}", pose_t.to_adjoint());

        // println!("  new current_velocity: {current_velocity:?}");
        // Pre transform is 'current' (pose) frame expressed in parent (denoted sr).
        // So H^c_r
        let pre_pose_t = if let Some(pre_pose) = world.component::<PreTransform>(current_id) {
            // println!("Found PreTransform for  {current_id:?}");
            *pre_pose.transform()
        } else {
            *Pose::new().transform()
        };
        current_pose = (pre_pose_t * current_pose.transform()).into();
        current_velocity = pre_pose_t.to_adjoint() * current_velocity;

        // T^i,b_a, motion of body a with respect to body b, expressed in psi_i.
        // T^j,l_k = Ad(H^j_i) * T^i,l_k
        // T^c,l_k = Ad(H^c_r) * T^r,l_k

        // println!("  pre_pose_t: {pre_pose_t:?}");
        // println!("  pre_pose_t adjoint: {:?}", pre_pose_t.to_adjoint());
        // println!("  new current_velocity: {current_velocity:?}");

        if let Some(parent) = world.component::<super::parent::Parent>(current_id) {
            current_id = parent.parent().clone();
        } else {
            // No parent... which means the parent is the origin!
            current_velocity = last_pose.to_inv_h().to_adjoint() * current_velocity;
            break;
        }
        // println!("  parent: {current_id:?}");
    }
    // That give us the velocity in the origin, do we need to mutiply that with the final transform
    // to the pose again?
    // return (current_pose.to_adjoint() * current_velocity).into();
    println!("linear: {linear:?}");
    println!("current_velocity: {current_velocity:?}");
    return current_velocity.into();
}

pub fn world_velocity_linear(world: &World, entity: EntityId) -> Velocity {
    // use crate::components::pose::world_pose;
    use crate::components::pose::Pose;
    use crate::components::pose::PreTransform;
    use crate::util::cgmath::prelude::*;

    let mut current_id = entity.clone();
    let mut current_velocity = cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0);
    // let mut current_pose = Pose::new();
    /*
        Changing frame of a twist.
        twist = Adjoint(Frame) * frame_twist.
    */
    loop {
        // println!("Current id: {current_id:?}");

        let pose_t = if let Some(pose) = world.component::<Pose>(current_id) {
            // println!("Found pose for  {current_id:?}");
            *pose.transform()
        } else {
            *Pose::new().transform()
        };
        let vel_t = if let Some(vel) = world.component::<Velocity>(current_id) {
            // println!("Found vel for  {current_id:?}");
            vel.to_twist()
        } else {
            Velocity::new().to_twist()
        };

        // current_pose = (pose_t * *current_pose).into();
        let combined_vel = current_velocity + vel_t.v;

        // println!("  current_velocity: {current_velocity:?}");
        // println!("  vel_t: {vel_t:?}");
        // println!("  combined_vel: {combined_vel:?}");
        // println!("  pose_t: {pose_t:?}");
        // println!("  pose_t adjoint: {:?}", pose_t.to_adjoint());
        current_velocity = pose_t.to_rotation() * (combined_vel);
        // println!("  new current_velocity: {current_velocity:?}");

        let pre_pose_t = if let Some(pre_pose) = world.component::<PreTransform>(current_id) {
            // println!("Found PreTransform for  {current_id:?}");
            *pre_pose.transform()
        } else {
            *Pose::new().transform()
        };
        // current_pose = (pre_pose_t * current_pose.transform()).into();
        // println!("  pre_pose_t: {pre_pose_t:?}");
        // println!("  pre_pose_t adjoint: {:?}", pre_pose_t.to_adjoint());
        current_velocity = pre_pose_t.to_rotation() * current_velocity;
        // println!("  new current_velocity: {current_velocity:?}");

        if let Some(parent) = world.component::<super::parent::Parent>(current_id) {
            current_id = parent.parent().clone();
        } else {
            break;
        }
        // println!("  parent: {current_id:?}");
    }
    // That give us the velocity in the origin, do we need to mutiply that with the final transform
    // to the pose again?
    // (current_pose.to_adjoint() * current_velocity).into()
    Velocity::from_velocities(current_velocity, cgmath::Vector3::<f32>::new(0.0, 0.0, 0.0))
}

#[cfg(test)]
mod test {
    use super::super::pose::Pose;
    use super::*;

    #[test]
    fn test_velocity_integration() {
        let start = Pose::new();
        let dt = 0.01f32;
        let mut v = Velocity::new();
        v.v[0] = 1.0;

        let mut p = start;
        for _i in 0..100 {
            p = (p.h * v.integrate(dt)).into();
        }

        assert!((p.h.w[0] - 100.0 * dt * 1.0).abs() <= 0.00001);
    }
    #[test]
    fn test_adjoint_from_lynch_mr_v2_fig_3_dot_18_p_100() {
        use crate::util::cgmath::prelude::*;
        use crate::util::test_util::*;
        use cgmath::vec3;
        #[allow(non_snake_case)]
        let Tsb = cgmath::Matrix4::<f32>::from_cols(
            cgmath::Vector4::<f32>::new(-1.0, 0.0, 0.0, 0.0),
            cgmath::Vector4::<f32>::new(0.0, 1.0, 0.0, 0.0),
            cgmath::Vector4::<f32>::new(0.0, 0.0, -1.0, 0.0),
            cgmath::Vector4::<f32>::new(4.0, 0.4, 0.0, 1.0),
        );

        // let rs = vec3(2.0, -1.0, 0.0);
        // let rb = vec3(2.0, -1.4, 0.0);

        // let ws = vec3(0.0, 0.0, 2.0);
        // let wb = vec3(0.0, 0.0, -2.0);

        // Different order convention!
        let given_vs = Twist::new(vec3(-2.0, -4.0, 0.0), vec3(0.0, 0.0, 2.0));
        let given_vb = Twist::new(vec3(2.8, 4.0, 0.0), vec3(0.0, 0.0, -2.0));

        let calculated_vs = Tsb.to_adjoint() * given_vb;
        println!("calculated_vs: {calculated_vs:?}");

        approx_equal!(given_vs.v.x, calculated_vs.v.x, 0.0001);
        approx_equal!(given_vs.v.y, calculated_vs.v.y, 0.0001);
        approx_equal!(given_vs.v.z, calculated_vs.v.z, 0.0001);
        approx_equal!(given_vs.w.x, calculated_vs.w.x, 0.0001);
        approx_equal!(given_vs.w.y, calculated_vs.w.y, 0.0001);
        approx_equal!(given_vs.w.z, calculated_vs.w.z, 0.0001);

        // Check the inverse.
        let calculated_vb = Tsb.to_inv_h().to_adjoint() * given_vs;
        approx_equal!(given_vb.v.x, calculated_vb.v.x, 0.0001);
        approx_equal!(given_vb.v.y, calculated_vb.v.y, 0.0001);
        approx_equal!(given_vb.v.z, calculated_vb.v.z, 0.0001);
        approx_equal!(given_vb.w.x, calculated_vb.w.x, 0.0001);
        approx_equal!(given_vb.w.y, calculated_vb.w.y, 0.0001);
        approx_equal!(given_vb.w.z, calculated_vb.w.z, 0.0001);
    }

    #[test]
    fn test_adjoint_murray_one_dof_manipulator_p_74_example_2_dot_5() {
        use crate::util::cgmath::prelude::*;
        use crate::util::test_util::*;
        use cgmath::vec3;
        /*
                         D
            ^y           Theta revolve
            |            ||-------------------B
                         ||                    ^ l0
            //A----------C <----- l2 -------> |
            //<--- l1 -->                     |
            //                                v
            -> x
            into screen; y
            

            Frame A is fixed.
            l1 is arm between A and Revolute joint.
            Revolute axis is facing up (scroll up... not out of the screen)
            l2 is arm between revolute joint and B.
            Let C be the frame before the revolute joint.
            Let D be the frame after the revolute joint.
        */

        // Lets ignore l0, it doesn't add any complexity here.
        let vel_spatial = |l1: f32, _l2: f32, dtheta: f32| Velocity::from_velocities(vec3(l1 * dtheta, 0.0, 0.0), vec3(0.0, 0.0, dtheta)).to_twist();
        let vel_body = |_l1: f32, l2: f32, dtheta: f32| Velocity::from_velocities(vec3(-l2 * dtheta, 0.0, 0.0), vec3(0.0, 0.0, dtheta)).to_twist();

        let l1 = 1.1;
        let l2 = 3.2;
        let theta = 0.0;
        let dtheta = 1.0;

        let p_a_c = Pose::from_se2(l1, 0.0, 0.0);
        let p_c_d = Pose::from_se2(0.0, 0.0, theta);

        let v_c_d = Velocity::from_velocities(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, dtheta)).to_twist();
        let p_d_b = Pose::from_se2(l2, 0.0, 0.0);

        let vnull = Velocity::from_velocities(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0)).to_twist();

        // Lets try to express B in A using these, and only these components.
        // First, lift velocity of B (null, to D).
        let v_b_in_d = p_d_b.to_inv_h().to_adjoint() * vnull;
        let v_b_in_c = p_c_d.to_inv_h().to_adjoint() * (v_b_in_d + v_c_d); // add the velocity of the joint here
        println!("   v_b_in_c: {v_b_in_c:?}");
        let v_b_in_a = p_a_c.to_inv_h().to_adjoint() * (v_b_in_c);

        let vel_spatial = vel_spatial(l1, l2, dtheta);

        // Shouldn't these be the same then??
        println!("vel_spatial: {vel_spatial:?}");
        println!("   v_b_in_a: {v_b_in_a:?}");

        // Velocity of the body, should be v_c_d carried to B?
        let vel_body = vel_body(l1, l2, dtheta);
        let v_in_B = p_d_b.to_inv_h().to_adjoint() * v_c_d;
        println!(" vel_body: {vel_body:?}");
        println!("   v_in_B: {v_in_B:?}");

        // Oh, their convention has 'x' positive out of the paper. So yeah, then their -x is our y
        // and the results match.
    }
}
