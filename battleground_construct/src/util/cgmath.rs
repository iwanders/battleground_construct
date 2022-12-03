use cgmath::{BaseFloat, Matrix4};

pub mod prelude {
    pub use super::ToQuaternion;
    pub use super::ToRotationH;
    pub use super::ToHomogenous;
    pub use super::InvertHomogeneous;
    pub use super::ToTranslation;
}

// https://github.com/rustgd/cgmath/issues/461
pub trait ToQuaternion<S: BaseFloat> {
    fn to_quaternion(&self) -> cgmath::Quaternion<S>;
}

impl<S: BaseFloat> ToQuaternion<S> for Matrix4<S> {
    fn to_quaternion(&self) -> cgmath::Quaternion<S> {
        // Build a 3x3 matrix.
        let m = cgmath::Matrix3::<S>::from_cols(
            self.x.truncate(),
            self.y.truncate(),
            self.z.truncate(),
        );
        m.into()
    }
}

pub trait ToRotationH<S: BaseFloat> {
    fn to_rotation_h(&self) -> Matrix4<S>;
}

impl<S: BaseFloat> ToRotationH<S> for Matrix4<S> {
    fn to_rotation_h(&self) -> cgmath::Matrix4<S> {
        cgmath::Matrix4::<S>::from_cols(
            self.x,
            self.y,
            self.z,
            cgmath::Vector4::<S>::new(S::zero(), S::zero(), S::zero(), S::one()),
        )
    }
}

pub trait ToHomogenous<S: BaseFloat> {
    fn to_h(&self) -> Matrix4<S>;
}

impl<S: BaseFloat> ToHomogenous<S> for cgmath::Vector3<S> {
    fn to_h(&self) -> cgmath::Matrix4<S> {
        cgmath::Matrix4::<S>::from_cols(
            cgmath::Vector4::<S>::new(S::one(), S::zero(), S::zero(), S::zero()),
            cgmath::Vector4::<S>::new(S::zero(), S::one(), S::zero(), S::zero()),
            cgmath::Vector4::<S>::new(S::zero(), S::zero(), S::one(), S::zero()),
            cgmath::Vector4::<S>::new(self.x, self.y, self.z, S::one()),
        )
    }
}

pub trait InvertHomogeneous<S: BaseFloat> {
    fn to_inv_h(&self) -> Matrix4<S>;
}

impl<S: BaseFloat> InvertHomogeneous<S> for Matrix4<S> {
    fn to_inv_h(&self) -> cgmath::Matrix4<S> {
        use cgmath::Matrix;
        let inv_r = cgmath::Matrix3::<S>::from_cols(
            self.x.truncate(),
            self.y.truncate(),
            self.z.truncate(),
        )
        .transpose();
        let v_inv = -inv_r * self.w.truncate();
        cgmath::Matrix4::<S>::from_cols(
            inv_r.x.extend(S::zero()),
            inv_r.y.extend(S::zero()),
            inv_r.z.extend(S::zero()),
            v_inv.extend(S::one()),
        )
    }
}


pub trait ToTranslation<S: BaseFloat> {
    fn to_translation(&self) -> cgmath::Vector3<S>;
}

impl<S: BaseFloat> ToTranslation<S> for Matrix4<S> {
    fn to_translation(&self) -> cgmath::Vector3<S> {
        self.w.truncate()
    }
}

