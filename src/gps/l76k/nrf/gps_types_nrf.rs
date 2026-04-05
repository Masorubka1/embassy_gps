use embassy_nrf::Peri;
use embassy_nrf::gpio::{Level, Output, OutputDrive, Pin};

use crate::gps::gps_interfases::GpsOutput;

/// nRF pin bundle required by the L76K hardware driver.
pub struct GpsHw<REINIT, STANDBY>
where
    REINIT: Pin,
    STANDBY: Pin,
{
    pub reinit: Peri<'static, REINIT>,
    pub standby: Peri<'static, STANDBY>,
}

/// Thin `GpsOutput` wrapper over `embassy_nrf::gpio::Output`.
pub struct NrfOutput<'d>(pub Output<'d>);

impl<'d> GpsOutput for NrfOutput<'d> {
    /// Sets GPIO high.
    fn set_high(&mut self) {
        self.0.set_high();
    }

    /// Sets GPIO low.
    fn set_low(&mut self) {
        self.0.set_low();
    }
}

impl<REINIT, STANDBY> GpsHw<REINIT, STANDBY>
where
    REINIT: Pin,
    STANDBY: Pin,
{
    /// Converts raw pins into configured output drivers.
    pub fn into_outputs(self) -> (NrfOutput<'static>, NrfOutput<'static>) {
        (
            NrfOutput(Output::new(self.reinit, Level::High, OutputDrive::Standard)),
            NrfOutput(Output::new(
                self.standby,
                Level::High,
                OutputDrive::Standard,
            )),
        )
    }
}
