use chrono::Timelike;
use core::option::Option;
use nmea::{Nmea, SentenceType};

/// Compact GPS fix data used by higher-level consumers.
#[derive(Clone, Copy, Debug)]
pub struct GpsFix {
    pub lat_microdeg: i32,
    pub lon_microdeg: i32,
    pub sats: u8,
    /// UTC milliseconds since midnight from NMEA time fields.
    pub utc_time_ms: Option<u64>,
}

impl GpsFix {
    /// Builds a fix from the currently parsed NMEA state.
    pub fn from(nmea: &Nmea) -> Option<Self> {
        let utc_time_ms = nmea.fix_timestamp().map(|t| {
            u64::from(t.num_seconds_from_midnight()) * 1_000 + u64::from(t.nanosecond() / 1_000_000)
        });

        Some(GpsFix {
            lat_microdeg: (nmea.latitude()? * 1_000_000.0) as i32,
            lon_microdeg: (nmea.longitude()? * 1_000_000.0) as i32,
            sats: nmea.fix_satellites().unwrap_or(0) as u8,
            utc_time_ms,
        })
    }
}

/// Events emitted by the driver/FSM pipeline.
#[derive(Clone, Copy, Debug)]
pub enum GpsEvent {
    BytesDetected,
    Sentence(SentenceType),
    Fix(GpsFix),
}

/// Internal states used by the recovery-oriented FSM.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpsState {
    Reset,
    SetupConnection,
    Running,
    Recover,
}

/// Unified error type for GPS IO, parsing, and setup.
#[derive(Debug)]
pub enum GpsError<IO> {
    IO(IO),
    Utf8,
    NotReady,
    BaudNotDetected,
    Timeout,
    BuildCommand,
}
