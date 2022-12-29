use crate::interface::Interface;

/// The unit controller should implement this trait.
pub trait UnitControl {
    /// Function used to control the unit.
    fn update(&mut self, interface: &mut dyn Interface) -> Result<(), Box<ControlError>>;

    /// Return a string denoting the control type.
    fn control_type(&self) -> &'static str {
        "unknown"
    }
}

/// If an error occurs in the unit control, this is the type is used in the Err field.
#[derive(Debug)]
pub enum ControlError {
    /// The controller exceeded allowed resources.
    ResourcesExceeded,
    /// An exception occured inside the controller and is propagated up. If this is used from within
    /// a wasm environment, the string representation can only be propagated.
    WrappedError(Box<dyn std::error::Error>),
    /// A simple integer return value,
    ErrorCode(u32),
}

/// Finally, it implements display.
impl std::fmt::Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ControlError::ResourcesExceeded => {
                write!(f, "resource limit exceeded")
            }
            ControlError::WrappedError(ref e) => {
                write!(f, "wrapped error: {}", e)
            }
            ControlError::ErrorCode(i) => {
                write!(f, "error code {i}")
            }
        }
    }
}

/// And error, such that it is convertible to Box<dyn std::error:Error>
impl std::error::Error for ControlError {}

/// Simple error struct for strings, used to [`ControlError::WrappedError`] type errors coming from
/// webassembly modules.
#[derive(Debug)]
pub struct ControlStringError(pub String);
impl std::error::Error for ControlStringError {}
impl std::fmt::Display for ControlStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
