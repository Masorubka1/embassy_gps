use embassy_time::Duration;

use crate::gps::l76k::pcas::models::EncodedCommand;

/// Async low-level GPS driver contract.
pub trait GpsDriver {
    /// Event yielded by the driver.
    type Event;
    /// Driver error type.
    type Error;

    /// Performs a hardware reset sequence.
    fn reset(&mut self) -> impl Future<Output = ()>;
    /// Resets parser/runtime state before work.
    fn prepare(&mut self) -> impl Future<Output = ()>;
    /// Runs one receive loop iteration until an event is available.
    fn run(&mut self) -> impl Future<Output = Result<Self::Event, Self::Error>>;

    /// Sends a raw encoded command to the GPS chip.
    fn execute_command<const N: usize>(
        &mut self,
        cmd: &EncodedCommand<N>,
        timeout: Duration,
    ) -> impl Future<Output = Result<(), Self::Error>>;
}

/// Async finite-state-machine contract around a GPS driver.
pub trait GpsFsm {
    /// Event emitted by the state machine.
    type Event;
    /// Error emitted by the state machine.
    type Error;

    /// Executes one FSM step.
    fn step(&mut self) -> impl Future<Output = Result<Option<Self::Event>, Self::Error>>;
}

/// Minimal output pin abstraction used by reset/standby control.
pub trait GpsOutput {
    /// Drives the output high.
    fn set_high(&mut self);
    /// Drives the output low.
    fn set_low(&mut self);
}
