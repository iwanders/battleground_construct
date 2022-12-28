pub mod tank;

/// Unit type enum to denote the unit type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitType {
    Tank,
}

impl TryFrom<u32> for UnitType {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
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
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_unit_type_conversion() {
        assert_eq!(UnitType::Tank, (UnitType::Tank as u32).try_into().unwrap());
    }
}
