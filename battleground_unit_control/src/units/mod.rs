pub mod artillery;
pub mod tank;

pub mod common {
    pub const MODULE_CLOCK: u32 = 0x0100;
    pub const MODULE_OBJECTIVES: u32 = 0x0200;
    pub const MODULE_TEAM: u32 = 0x0300;
    pub const MODULE_UNIT: u32 = 0x0400;
    pub const MODULE_RADIO_TRANSMITTER: u32 = 0x0500;
    pub const MODULE_RADIO_RECEIVER: u32 = 0x0600;

    /// Gps is located in the body origin.
    pub const MODULE_GPS: u32 = 0x1700;
    /// Drawing also originates from the body origin.
    pub const MODULE_DRAW: u32 = 0x1800;
}

/// Unit type enum to denote the unit type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitType {
    Artillery,
    Tank,
}

impl TryFrom<u32> for UnitType {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == UnitType::Artillery as u32 => Ok(UnitType::Artillery),
            x if x == UnitType::Tank as u32 => Ok(UnitType::Tank),
            _ => Err(()),
        }
    }
}

/// Finally, it implements display.
impl std::fmt::Display for UnitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnitType::Tank => {
                write!(f, "tank")
            }
            UnitType::Artillery => {
                write!(f, "artillery")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_unit_type_conversion() {
        assert_eq!(UnitType::Tank, (UnitType::Tank as u32).try_into().unwrap());
        assert_eq!(
            UnitType::Artillery,
            (UnitType::Artillery as u32).try_into().unwrap()
        );
    }
}
