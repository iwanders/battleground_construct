//!
//! This module holds all information about the unit control modules. Each module is of a certain
//! type, a unit may contain multiple modules of the same type, for example the revolute joint
//! control is very common and occurs multiple times in the units.
//!
//! Register offset constants start with `REG_`, they contain the module name so should not cause
//! naming collisions when imported with a wildcard.
//!
//! If modules need to provide a list, the standard format is to have a `_COUNT` register denoting
//! the number of entries in the list.
//! Individual entry starts are at `_START + index * _STRIDE`.
//! Fields are provided with `_OFFSET_` values, offsetting from the individual entry start.
//! - So the first field of the first entry would be `_START + 0 * _STRIDE + _OFFSET_FIELD1`
//! - So the second field of the first entry would be `_START + 0 * _STRIDE + _OFFSET_FIELD2`
//! - So the first field of the second entry would be `_START + 1 * _STRIDE + _OFFSET_FIELD1`
//! - So the second field of the second entry would be `_START + 1 * _STRIDE + _OFFSET_FIELD2`
//! If stride is omitted, it is assumed to be `1`.
//!
//! Boolean values are represented by integers, if the integer is zero, this represents boolean
//! `false`, if the value is non-zero it represents `true`.

pub mod cannon;
pub mod clock;
pub mod differential_drive;
pub mod draw;
pub mod gps;
pub mod gun_battery;
pub mod objectives;
pub mod radar;
pub mod radio_receiver;
pub mod radio_transmitter;
pub mod revolute;
pub mod team;
pub mod unit;
