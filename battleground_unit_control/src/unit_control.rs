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
#[derive(Debug, Clone)]
pub enum ControlError {
    ResourcesExceeded,
    Unrecoverable,
    ErrorCode(i32),
}

/// Finally, it implements display.
impl std::fmt::Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ControlError::ResourcesExceeded => {
                write!(f, "resource limit exceeded")
            }
            ControlError::Unrecoverable => {
                write!(f, "an unrecoverable error")
            }
            ControlError::ErrorCode(i) => {
                write!(f, "error code {i}")
            }
        }
    }
}

/// And error, such that it is convertible to Box<dyn std::error:Error>
impl std::error::Error for ControlError {}
