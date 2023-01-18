//! This crate provides the interface used to control units.
//!
//! It exposes the [`UnitControl`] trait which all unit controllers should implement. Interaction
//! with the unit itself happens through the [`Interface`] that's provided to the controller.
//! A unit may expose several modules of the same type, information about the modules themselves is
//! provided in the [`modules`] submodule of this crate, and [`units`] provides the module ids per
//! unit.
//!
//! This crate also provides the necessary boilerplate to allow compiling a unit controller as a
//! `wasm32-unknown-unknown` type, as is used by the `unit_control_wasm` crate.

/// Only for wasm32, load the wasm_interface, it has a bunch of extern "C" methods to create the
/// controller and interface and invoke this periodically.
#[doc(hidden)]
#[cfg(feature = "wasm-interface")]
pub mod wasm_interface;

/// The register interface is used as the default implementation of the [`Interface`] trait, used by
/// both the wasm control and native control. It is not considered public API.
#[doc(hidden)]
#[cfg(feature = "register-interface")]
pub mod register_interface;

/// The constants and helpers for modules.
pub mod modules;

/// The constants for units.
pub mod units;

/// Export the log interface, this is used on wasm32 to be able to print.
pub use log;

/// Plugins will provide a function of this signature.
pub type ControllerSpawn = fn() -> Box<dyn UnitControl>;

/// The interface trait and types.
pub mod interface;
pub use interface::{Interface, InterfaceError, RegisterType};

/// The unit control trait and related types.
pub mod unit_control;
pub use unit_control::UnitControl;
