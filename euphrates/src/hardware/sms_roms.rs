//! Functions for dealing with Sega Master System ROM images.

use std::fs::File;
use std::io::Error as IoError;
use std::io::Read;
use std::path::Path;


// This superfluous module with the `allow` attribute is necessary until the
// `fail` crate begins using `dyn trait` syntax
#[allow(bare_trait_objects)]
mod sms_rom_error {
    use super::*;

    /// Error generated when attempting to load an invalid ROM image.
    #[derive(Debug, Fail)]
    pub enum SmsRomError {
        #[fail(
            display = "ROM of bad length 0x{:X} (should be positive, less than 0x400000, and either no bigger than 0x2000 or a multiple of 0x4000",
            _0
        )]
        BadLength(usize),
        #[fail(display = "IO error {}", _0)]
        Io(#[cause] IoError),
    }
}

pub use self::sms_rom_error::SmsRomError;

impl From<IoError> for SmsRomError {
    fn from(x: IoError) -> Self {
        SmsRomError::Io(x)
    }
}

/// If the ROM does not have the right length, fix that. Our ROMs will always
/// have a length a multiple of 0x4000.
///
/// This function:
/// * strips off a 512 byte header, if present
/// * does ROM mirroring of `0x2000` byte slices (it just puts two copies of the
///   ROM image into a `[u8; 0x4000]`).
/// * pads ROMs shorter than `0x2000` bytes
///
/// ROMs longer than `0x2000` bytes but not a multiple of `0x4000` will give an
/// error. (I'm not sure at the moment whether such ROMs are valid.)
pub fn format(rom: Box<[u8]>) -> Result<Box<[u8]>, SmsRomError> {
    let len = rom.len();

    if len == 0 || len > 0x400000 {
        return Err(SmsRomError::BadLength(len));
    }

    if len % 0x4000 == 0 {
        return Ok(rom);
    }

    if len % 0x2000 == 0x200 {
        // it's got a 512 byte header we need to remove
        return format(rom[0x200..].to_vec().into_boxed_slice());
    }

    if len < 0x2000 {
        // pad with 0?
        let mut x = rom.into_vec();
        x.resize(0x2000, 0u8);
        return format(x.into_boxed_slice());
    }

    if len == 0x2000 {
        // mirror the ROM manually. Not sure what happens in actual hardware,
        // but this works on the ROMs I've tested. SMS Plus just rejects ROMs
        // smaller than 0x4000.
        let mut x = Vec::with_capacity(0x4000);
        x.extend_from_slice(&rom[..]);
        x.extend_from_slice(&rom[..]);
        return Ok(x.into_boxed_slice());
    }

    return Err(SmsRomError::BadLength(len));
}

/// Load a SMS ROM from the indicated file.
///
/// This function will fix up the ROM in the same way `format` does.
pub fn from_file<P>(p: P) -> Result<Box<[u8]>, SmsRomError>
where
    P: AsRef<Path>,
{
    use std::mem::drop;
    let mut file = File::open(p)?;
    let mut buf = Vec::with_capacity(0x8000);
    file.read_to_end(&mut buf)?;
    drop(file);
    format(buf.into_boxed_slice())
}
