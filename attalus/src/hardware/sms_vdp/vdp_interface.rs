use impler::{Cref, Impl, Mref, Ref};

use super::*;

/// The hardware intereface of the VDP.
pub trait SmsVdpInterface {
    /// Write a byte to the data port.
    fn write_data(&mut self, x: u8);

    /// Read a byte from the data port.
    ///
    /// Reads are buffered into the VDP's `data_buffer`. Thus, reading the data
    /// port returns the value of `data_buffer`, while also storing the byte of
    /// VRAM at `address` into the `data_buffer`, and then incrementing the
    /// `address` (that is, incrementing the low 14 bits of `address_register`,
    /// wrapping past 0x3FFF). It also clears the control flag.
    fn read_data(&mut self) -> u8;

    /// Write a byte to the control port.
    fn write_control(&mut self, x: u8);

    /// Read a byte from the control port.
    ///
    /// This returns the current status flags byte, as well as clearing the
    /// status flags, the control flag, and the line interrupt pending flag.
    fn read_control(&mut self) -> u8;

    /// Read the `v` counter.
    ///
    /// Each time a line is drawn, the `v` counter is incremented. When the last
    /// line is completed, `v` is returned to 0. Since there are 262 lines in
    /// NTSC and 313 in PAL (not all displayed), `v` needs to be at least 9 bits
    /// wide; however, the VDP can only provide an 8 bit value here.
    /// Consequently, there are numbers `threshold` and `delta`, and if `v >
    /// threshold`, then `v - delta` is what is actually returned, and it will
    /// fit into 8 bits. The value of `threshold` and `delta` vary depending on
    /// `tv_system()` and `resolution()`; see the default implementation of
    /// `read_v` to see what these values are. (Actually, it's slightly more
    /// complicated: for medium and high resolution modes in PAL, there are two
    /// different thresholds and two different deltas.)
    fn read_v(&mut self) -> u8;

    /// Read the `h` counter.
    ///
    /// The VDP draws pixels on the screen left to right, top to bottom. Each
    /// time a pixel is drawn, `h` is incremented. There are 342 pixels in each
    /// line (although only the first 256 are displayed), and after the 342nd
    /// pixel on a line is drawn, `h` returns to 0. Thus `h` is a 9 bit value,
    /// but since the VDP can only produce 8 bits here, only the most significant
    /// 8 bits are provided. (This is the way it's described in MacDonald's VDP
    /// documentation and the way this API is designed to work, but I suppose we
    /// could equivalently say the counter is an 8 bit value that's incremented
    /// every second pixel.)
    ///
    /// But it's more complicated than that: the value returned here is not
    /// the value of `h` *now*; it's the value of `h` the last time the TH pin
    /// of either joystick port was changed. This is apparently useful for the
    /// lightgun peripheral.
    ///
    /// All that said, it may not be possible to do anything reasonable with `h`
    /// using this API. This API is designed to draw a line at a time; no one is
    /// ever going to have access to the trait while `h` should be anything but
    /// 0. There is probably no better option than for `h` to always be 0.
    /// Fortunately, this doesn't seem to matter for anything but lightgun
    /// games.
    fn read_h(&mut self) -> u8;
}

pub struct SmsVdpInterfaceImpl;

impl<T> SmsVdpInterface for T
where
    T: Impl<SmsVdpInterfaceImpl> + ?Sized,
    T::Impler: SmsVdpInterface,
{
    #[inline]
    fn write_data(&mut self, x: u8) {
        self.make_mut().write_data(x)
    }

    #[inline]
    fn read_data(&mut self) -> u8 {
        self.make_mut().read_data()
    }

    #[inline]
    fn write_control(&mut self, x: u8) {
        self.make_mut().write_control(x)
    }

    #[inline]
    fn read_control(&mut self) -> u8 {
        self.make_mut().read_control()
    }

    #[inline]
    fn read_v(&mut self) -> u8 {
        self.make_mut().read_v()
    }

    #[inline]
    fn read_h(&mut self) -> u8 {
        self.make_mut().read_h()
    }
}

pub struct SmsVdpInterfaceImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> SmsVdpInterfaceImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(SmsVdpInterfaceImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(SmsVdpInterfaceImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T: ?Sized> SmsVdpInterface for SmsVdpInterfaceImpler<T>
where
    T: SmsVdpInternal,
{
    fn write_data(&mut self, x: u8) {
        let z = self.0.mut_0();
        let code = z.code();
        let addr = z.address();
        z.set_data_buffer(x);

        match (code, z.kind()) {
            (3, Kind::Gg) => {
                if addr & 1 == 0 {
                    z.set_cram_latch(x);
                } else {
                    let latch = z.cram_latch();
                    let val = latch as u16 | ((x as u16) << 8);
                    let actual_address = (addr >> 1) % 32;
                    unsafe {
                        z.set_cram_unchecked(actual_address, val);
                    }
                }
            }
            (3, _) => unsafe {
                z.set_cram_unchecked(addr % 32, x as u16);
            },
            _ => unsafe {
                z.set_vram_unchecked(addr, x);
            },
        }

        z.set_address(addr + 1);
        z.set_control_flag(false);
    }

    fn read_data(&mut self) -> u8 {
        let z = self.0.mut_0();
        let current_buffer = z.data_buffer();
        let code_addr = z.code_address();
        let addr = code_addr & 0x3FFF;
        let new_value = unsafe { z.vram_unchecked(addr) };
        z.set_address(addr + 1);
        z.set_data_buffer(new_value);
        z.set_control_flag(false);
        current_buffer
    }

    fn write_control(&mut self, x: u8) {
        let z = self.0.mut_0();
        if z.control_flag() {
            z.set_control_flag(false);
            let low_byte = z.code_address() & 0xFF;
            let code_addr = low_byte | (x as u16) << 8;
            z.set_code_address(code_addr);
            let code = z.code();
            let addr = z.address();
            if code == 0 {
                let val = unsafe { z.vram_unchecked(addr) };
                z.set_data_buffer(val);
                z.set_address(addr + 1);
            } else if code == 2 {
                let which_reg = x & 0xF;
                if which_reg < 11 {
                    unsafe {
                        z.set_register_unchecked(which_reg as u16, low_byte as u8);
                    }
                }
            }
        } else {
            z.set_control_flag(true);
            let high_byte = z.code_address() & 0xFF00;
            z.set_code_address(high_byte | x as u16);
        }
    }

    fn read_control(&mut self) -> u8 {
        let z = self.0.mut_0();
        let current_status = z.status_flags();
        z.set_status_flags(0);
        z.set_control_flag(false);
        z.set_line_interrupt_pending(false);
        current_status
    }

    fn read_v(&mut self) -> u8 {
        use self::Resolution::*;
        use self::TvSystem::*;

        let z = self.0.mut_0();
        let v = z.v();
        let result = match (z.tv_system(), z.resolution(), v) {
            (Ntsc, Low, 0...0xDA) => v,
            (Ntsc, Low, _) => v - 6,
            (Ntsc, Medium, 0...0xEA) => v,
            (Ntsc, Medium, _) => v - 6,
            (Ntsc, High, 0...0xFF) => v,
            (Ntsc, High, _) => v - 0x100,
            (Pal, Low, 0...0xF2) => v,
            (Pal, Low, _) => v - 57,
            (Pal, Medium, 0...0xFF) => v,
            (Pal, Medium, 0x100...0x102) => v - 0x100,
            (Pal, Medium, _) => v - 57,
            (Pal, High, 0...0xFF) => v,
            (Pal, High, 0x100...0x10A) => v - 0x100,
            (Pal, High, _) => v - 57,
        };
        result as u8
    }

    fn read_h(&mut self) -> u8 {
        let h = self.0.mut_0().h();
        let result = (h >> 1) as u8;
        result as u8
    }
}
