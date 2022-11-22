use cgmath::{BaseFloat, Matrix4};

// https://github.com/rustgd/cgmath/issues/461
pub trait ToQuaternion<S: BaseFloat>  {
    fn to_quaternion(&self) -> cgmath::Quaternion::<S>;
}

impl<S: BaseFloat> ToQuaternion<S> for Matrix4<S> {
    fn to_quaternion(&self) -> cgmath::Quaternion::<S>
    {
        // Build a 3x3 matrix.
        let m = cgmath::Matrix3::<S>::from_cols(self.x.truncate(),self.y.truncate(),self.z.truncate());
        m.into()
    }
}
