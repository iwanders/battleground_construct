//! Information about the unit.

/// Holds the unit id, integer value.
pub const REG_UNIT_UNIT_ID: u32 = 0x10;

/// Holds the unit type, integer value convertible to [`UnitType`].
pub const REG_UNIT_UNIT_TYPE: u32 = 0x11;

/// Sentinel value if the unit doesn't have an id.
pub const UNIT_NO_UNIT_ID: i32 = -1;
/// Sentinel value if the unit doesn't have a unit type.
pub const UNIT_NO_UNIT_TYPE: i32 = -1;
