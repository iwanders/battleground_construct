//!
//! Just as with the registers, module identifier constants have either the unit name in them or
//! are shared between all units at a common register id. There should not be any naming conflicts
//! when importing with a wildcard.

pub mod artillery;
pub mod tank;

/// Holds module information that's shared between units.
pub mod common {
    /// Module identifier for the clock module.
    pub const MODULE_CLOCK: u32 = 0x0100;
    /// Module identifier for the objectives module.
    pub const MODULE_OBJECTIVES: u32 = 0x0200;
    /// Module identifier for the unit's team module.
    pub const MODULE_TEAM: u32 = 0x0300;
    /// Module identifier for the unit's unit module.
    pub const MODULE_UNIT: u32 = 0x0400;
    /// Module identifier for the unit's radio transmitter module.
    pub const MODULE_RADIO_TRANSMITTER: u32 = 0x0500;
    /// Module identifier for the unit's radio receiver module.
    pub const MODULE_RADIO_RECEIVER: u32 = 0x0600;

    /// Module identifier for the unit's gps module.
    pub const MODULE_GPS: u32 = 0x1700;
    /// Module identifier for the unit's draw module.
    pub const MODULE_DRAW: u32 = 0x1800;
}

/// Unit type enum to denote the unit type.
///
/// This implements [`UnitType::try_from`] to convert from u32 to the enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum UnitType {
    Tank,
    Artillery,
}

impl TryFrom<u32> for UnitType {
    type Error = &'static str;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == UnitType::Artillery as u32 => Ok(UnitType::Artillery),
            x if x == UnitType::Tank as u32 => Ok(UnitType::Tank),
            _ => Err("could not convert unit_type"),
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
