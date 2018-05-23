//! Functions for dealing with Sega Master System ROM images.

use std::fs::File;
use std::io::Error as IoError;
use std::io::Read;
use std::path::Path;

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

impl From<IoError> for SmsRomError {
    fn from(x: IoError) -> Self {
        SmsRomError::Io(x)
    }
}

/// Turn the ROM image as a boxed slice of bytes into a boxed slice of `0x4000` byte arrays.
///
/// Since SMS ROMs are composed of `0x4000` byte pages, we use the type `[[u8;
/// 0x4000]]` to represent them. This function:
///
/// * strips off a 512 byte header, if present
/// * does ROM mirroring of `0x2000` byte slices (it just puts two copies of the ROM image
///   into a `[u8; 0x4000]`).
/// * pads ROMs shorter than `0x2000` bytes
///
/// ROMs longer than `0x2000` bytes but not a multiple of `0x4000` will give an
/// error. (I'm not sure at the moment whether such ROMs are valid.)
pub fn format(mut rom: Box<[u8]>) -> Result<Box<[[u8; 0x4000]]>, SmsRomError> {
    use std::mem::forget;
    use std::slice::from_raw_parts_mut;

    let len = rom.len();

    if len == 0 || len > 0x400000 {
        return Err(SmsRomError::BadLength(len));
    }

    if len % 0x4000 == 0 {
        // we're good; just need to change the type
        let ptr = rom.as_mut_ptr() as *mut [u8; 0x4000];
        let new_len = len / 0x4000;
        forget(rom);
        return Ok(unsafe { Box::from_raw(from_raw_parts_mut(ptr, new_len)) });
    }

    if len % 0x2000 == 0x200 {
        // it's got a 512 byte header we need to remove
        return format(rom[0x200..].to_vec().into_boxed_slice());
    }

    if len < 0x2000 {
        // pad with 0? XXX
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
        // now format it correctly
        return format(x.into_boxed_slice());
    }

    return Err(SmsRomError::BadLength(len));
}

/// Load a SMS ROM from the indicated file.
///
/// This function will fix up the ROM in the same way `format` does.
pub fn from_file<P>(p: P) -> Result<Box<[[u8; 0x4000]]>, SmsRomError>
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
