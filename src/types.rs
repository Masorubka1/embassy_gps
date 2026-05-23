use core::option::Option;

use chrono::Timelike;
use nmea::{Nmea, SentenceType};

/// Compact GPS fix data used by higher-level consumers.
#[derive(Clone, Copy, Debug)]
pub struct GpsFix {
    pub lat_microdeg: i32,
    pub lon_microdeg: i32,
    pub sats: u8,
    /// UTC milliseconds since midnight from NMEA time fields.
    pub utc_time_ms: Option<u64>,
    pub time: Option<chrono::NaiveTime>,
    pub date: Option<chrono::NaiveDate>,
}

impl GpsFix {
    /// Builds a fix from the currently parsed NMEA state.
    pub fn from(nmea: &Nmea) -> Option<Self> {
        let time = nmea.fix_timestamp();
        let utc_time_ms = time.map(|t| {
            u64::from(t.num_seconds_from_midnight()) * 1_000 + u64::from(t.nanosecond() / 1_000_000)
        });
        let date = nmea.fix_date;

        Some(GpsFix {
            lat_microdeg: (nmea.latitude()? * 1_000_000.0) as i32,
            lon_microdeg: (nmea.longitude()? * 1_000_000.0) as i32,
            sats: nmea.fix_satellites().unwrap_or(0) as u8,
            utc_time_ms,
            time,
            date,
        })
    }

    /// Combines date and time information to full timestamp, if available
    #[must_use]
    pub fn get_timestamp(&self) -> Option<chrono::NaiveDateTime> {
        if let (Some(date), Some(time)) = (&self.date, &self.time) {
            Some(date.and_time(*time))
        } else {
            None
        }
    }

    /// Combines date and time information to full timestamp, if available
    #[must_use]
    pub fn get_timestamp_millis(&self) -> Option<i64> {
        self.get_timestamp().map(|t| t.and_utc().timestamp_millis())
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
