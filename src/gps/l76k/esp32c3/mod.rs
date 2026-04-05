//! ESP32-C3 specific pin wrappers and FSM wiring for L76K.

mod fsm_esp32c3;
mod gps_types_esp32c3;

pub use fsm_esp32c3::L76kFsm;
pub use gps_types_esp32c3::GpsHw;
pub(crate) use gps_types_esp32c3::EspOutput;
