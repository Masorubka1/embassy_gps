use core::fmt::Write;

use super::models::{
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gps::l76k::pcas::PcasSentenceRate;

    #[test]
    fn encode_pcas_wraps_body_with_checksum() {
        let encoded = encode_pcas::<32>("PCAS04,3").expect("encode should succeed");
        assert_eq!(encoded.as_str().expect("utf8"), "$PCAS04,3*1A\r\n");
    }

    #[test]
    fn encode_pcas_fails_when_buffer_is_too_small() {
        let err = match encode_pcas::<13>("PCAS04,3") {
            Ok(_) => panic!("must fail on small output buffer"),
            Err(err) => err,
        };
        assert_eq!(err, PcasBuildError::BufferTooSmall);
    }

    #[test]
    fn pcas01_set_baudrate_is_encoded_correctly() {
        let encoded = PcasCommand::Pcas01SetBaudrate(PcasBaudrate::B9600)
            .encode::<32>()
            .expect("encode should succeed");
        assert_eq!(encoded.as_str().expect("utf8"), "$PCAS01,1*1D\r\n");
    }

    #[test]
    fn pcas03_sentence_rates_are_encoded_correctly() {
        let cfg = Pcas03 {
            gga: PcasSentenceRate::every_n(1),
            gll: PcasSentenceRate::OFF,
            gsa: PcasSentenceRate::OFF,
            gsv: PcasSentenceRate::OFF,
            rmc: PcasSentenceRate::every_n(1),
            vtg: PcasSentenceRate::OFF,
            zda: PcasSentenceRate::OFF,
            ant: PcasSentenceRate::OFF,
        };

        let encoded = PcasCommand::Pcas03SetSentenceRates(cfg)
            .encode::<64>()
            .expect("encode should succeed");
        assert_eq!(
            encoded.as_str().expect("utf8"),
            "$PCAS03,1,0,0,0,1,0,0,0,0,0,,,0,0*02\r\n"
        );
    }

    #[test]
    fn pcas03_keep_fields_are_left_empty() {
        let encoded = PcasCommand::Pcas03SetSentenceRates(Pcas03::ALL_KEEP)
            .encode::<64>()
            .expect("encode should succeed");
        assert_eq!(encoded.as_str().expect("utf8"), "$PCAS03,,,,,,,,,0,0,,,0,0*02\r\n");
    }
}
