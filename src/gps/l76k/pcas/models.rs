use core::fmt::{self, Write};

/// Fixed-size encoded PCAS frame ready for UART transmission.
pub struct EncodedCommand<const N: usize> {
    pub len: usize,
    pub buf: [u8; N],
}

impl<const N: usize> EncodedCommand<N> {
    /// Returns the valid command bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Returns the command as UTF-8 text.
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.as_bytes())
    }
}

/// Errors produced while building a PCAS command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcasBuildError {
    BufferTooSmall,
    Fmt,
}

/// Supported UART baudrate values for PCAS01.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcasBaudrate {
    B4800 = 0,
    B9600 = 1,
    B19200 = 2,
    B38400 = 3,
    B57600 = 4,
    B115200 = 5,
}

impl PcasBaudrate {
    /// Returns the numeric protocol code.
    pub fn code(self) -> u8 {
        self as u8
    }
}

/// GNSS constellation selection for PCAS04.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcasGnssMode {
    Gps = 1,
    BeiDou = 2,
    GpsBeiDou = 3,
    Glonass = 4,
    GpsGlonass = 5,
    BeiDouGlonass = 6,
    GpsBeiDouGlonass = 7,
}

impl PcasGnssMode {
    /// Returns the numeric protocol code.
    pub fn code(self) -> u8 {
        self as u8
    }
}

/// Restart mode for PCAS10.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcasRestartMode {
    HotStart = 0,
    WarmStart = 1,
    ColdStart = 2,
    FactoryReset = 3,
}

impl PcasRestartMode {
    /// Returns the numeric protocol code.
    pub fn code(self) -> u8 {
        self as u8
    }
}

/// Per-sentence output rate entry used by PCAS03.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcasSentenceRate(Option<u8>);

impl PcasSentenceRate {
    pub const KEEP: Self = Self(None);
    pub const OFF: Self = Self(Some(0));

    /// Enables a sentence every `n` cycles.
    pub fn every_n(n: u8) -> Self {
        assert!(n <= 9);
        Self(Some(n))
    }

    /// Writes the field in PCAS03 comma format.
    pub fn write_to(&self, w: &mut FixedBuf<'_>) -> Result<(), PcasBuildError> {
        match self.0 {
            Some(v) => write!(w, "{v}").map_err(|_| PcasBuildError::Fmt),
            None => Ok(()),
        }
    }
}

/// Sentence-rate configuration payload for PCAS03.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pcas03 {
    pub gga: PcasSentenceRate,
    pub gll: PcasSentenceRate,
    pub gsa: PcasSentenceRate,
    pub gsv: PcasSentenceRate,
    pub rmc: PcasSentenceRate,
    pub vtg: PcasSentenceRate,
    pub zda: PcasSentenceRate,
    pub ant: PcasSentenceRate,
}

impl Pcas03 {
    /// Disables every known sentence.
    pub const ALL_OFF: Self = Self {
        gga: PcasSentenceRate::OFF,
        gll: PcasSentenceRate::OFF,
        gsa: PcasSentenceRate::OFF,
        gsv: PcasSentenceRate::OFF,
        rmc: PcasSentenceRate::OFF,
        vtg: PcasSentenceRate::OFF,
        zda: PcasSentenceRate::OFF,
        ant: PcasSentenceRate::OFF,
    };

    /// Keeps current rates for every known sentence.
    pub const ALL_KEEP: Self = Self {
        gga: PcasSentenceRate::KEEP,
        gll: PcasSentenceRate::KEEP,
        gsa: PcasSentenceRate::KEEP,
        gsv: PcasSentenceRate::KEEP,
        rmc: PcasSentenceRate::KEEP,
        vtg: PcasSentenceRate::KEEP,
        zda: PcasSentenceRate::KEEP,
        ant: PcasSentenceRate::KEEP,
    };
}

/// Small fixed writer used to build command buffers in `no_std`.
pub struct FixedBuf<'a> {
    buf: &'a mut [u8],
    pub len: usize,
}

impl<'a> FixedBuf<'a> {
    /// Creates a writer over a mutable byte slice.
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, len: 0 }
    }

    /// Returns the currently written bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Appends a single byte.
    pub fn push_byte(&mut self, b: u8) -> Result<(), PcasBuildError> {
        if self.len >= self.buf.len() {
            return Err(PcasBuildError::BufferTooSmall);
        }
        self.buf[self.len] = b;
        self.len += 1;
        Ok(())
    }

    /// Appends a byte slice.
    pub fn push_bytes(&mut self, data: &[u8]) -> Result<(), PcasBuildError> {
        if self.len + data.len() > self.buf.len() {
            return Err(PcasBuildError::BufferTooSmall);
        }
        self.buf[self.len..self.len + data.len()].copy_from_slice(data);
        self.len += data.len();
        Ok(())
    }
}

impl Write for FixedBuf<'_> {
    /// Writes UTF-8 text into the fixed buffer.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_bytes(s.as_bytes()).map_err(|_| fmt::Error)
    }
}
