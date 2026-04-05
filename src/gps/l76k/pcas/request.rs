use core::fmt::Write;

use crate::gps::l76k::pcas::models::{
    EncodedCommand, FixedBuf, Pcas03, PcasBaudrate, PcasBuildError, PcasGnssMode, PcasRestartMode,
};

/// Supported outbound PCAS commands.
pub enum PcasCommand {
    Pcas01SetBaudrate(PcasBaudrate),
    Pcas02SetUpdateRateMs(u16),
    Pcas03SetSentenceRates(Pcas03),
    Pcas04SetGnss(PcasGnssMode),
    Pcas10Restart(PcasRestartMode),
}

impl PcasCommand {
    /// Encodes a typed command into a full `$...*CS\r\n` frame.
    pub fn encode<const N: usize>(&self) -> Result<EncodedCommand<N>, PcasBuildError> {
        let mut body_buf = [0u8; 96];
        let mut w = FixedBuf::new(&mut body_buf);

        match *self {
            Self::Pcas01SetBaudrate(baud) => {
                write!(&mut w, "PCAS01,{}", baud.code()).map_err(|_| PcasBuildError::Fmt)?;
            }
            Self::Pcas02SetUpdateRateMs(interval_ms) => {
                write!(&mut w, "PCAS02,{interval_ms}").map_err(|_| PcasBuildError::Fmt)?;
            }
            Self::Pcas03SetSentenceRates(cfg) => {
                w.push_bytes(b"PCAS03,")?;

                cfg.gga.write_to(&mut w)?;
                w.push_byte(b',')?;
                cfg.gll.write_to(&mut w)?;
                w.push_byte(b',')?;
                cfg.gsa.write_to(&mut w)?;
                w.push_byte(b',')?;
                cfg.gsv.write_to(&mut w)?;
                w.push_byte(b',')?;
                cfg.rmc.write_to(&mut w)?;
                w.push_byte(b',')?;
                cfg.vtg.write_to(&mut w)?;
                w.push_byte(b',')?;
                cfg.zda.write_to(&mut w)?;
                w.push_byte(b',')?;
                cfg.ant.write_to(&mut w)?;
                w.push_byte(b',')?;

                // <Res>,<Res>,<Res>,<Res>,<Res>,<Res>
                // From the spec/example:
                // 9th  = always 0
                // 10th = always 0
                // 11th = reserved (empty is fine)
                // 12th = reserved (empty is fine)
                // 13th = always 0
                // 14th = always 0
                w.push_bytes(b"0,0,,,0,0")?;
            }
            Self::Pcas04SetGnss(mode) => {
                write!(&mut w, "PCAS04,{}", mode.code()).map_err(|_| PcasBuildError::Fmt)?;
            }
            Self::Pcas10Restart(mode) => {
                write!(&mut w, "PCAS10,{}", mode.code()).map_err(|_| PcasBuildError::Fmt)?;
            }
        }

        let body = core::str::from_utf8(w.as_bytes()).map_err(|_| PcasBuildError::Fmt)?;
        encode_pcas::<N>(body)
    }
}

/// Calculates NMEA XOR checksum for the payload body.
fn checksum(body: &[u8]) -> u8 {
    body.iter().fold(0u8, |acc, &b| acc ^ b)
}

/// Converts a nibble to uppercase hexadecimal ASCII.
fn hex_upper(n: u8) -> u8 {
    match n {
        0..=9 => b'0' + n,
        10..=15 => b'A' + (n - 10),
        _ => unreachable!(),
    }
}

/// Wraps raw PCAS payload into a checksummed NMEA frame.
pub fn encode_pcas<const N: usize>(body: &str) -> Result<EncodedCommand<N>, PcasBuildError> {
    let mut out = [0u8; N];
    let mut w = FixedBuf::new(&mut out);

    w.push_byte(b'$')?;
    w.push_bytes(body.as_bytes())?;

    let cs = checksum(body.as_bytes());
    w.push_byte(b'*')?;
    w.push_byte(hex_upper(cs >> 4))?;
    w.push_byte(hex_upper(cs & 0x0F))?;
    w.push_byte(b'\r')?;
    w.push_byte(b'\n')?;

    Ok(EncodedCommand {
        len: w.len,
        buf: out,
    })
}
