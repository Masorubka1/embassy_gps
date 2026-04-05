//! nRF-specific pin wrappers and FSM wiring for L76K.

mod fsm_nrf;
mod gps_types_nrf;

pub use fsm_nrf::L76kFsm;
pub use gps_types_nrf::GpsHw;
pub(crate) use gps_types_nrf::NrfOutput;
