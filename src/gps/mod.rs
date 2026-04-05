//! Module layout for GPS support.
//! - `gps_interfases`: common traits for drivers, FSMs, and output pins.
//! - `l76k`: concrete implementation for the Quectel L76K module.
//! - `l76k::pcas`: encoder for PCAS configuration commands.
//! - `l76k::{nrf,esp32c3}`: platform glue for pin/output setup and FSM wiring.

pub mod gps_interfases;
pub mod l76k;
