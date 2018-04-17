use super::higher;
use super::*;

/// Methods for the VDP that other pieces of hardware, in particular the IO
/// system, should be able to use.
///
/// An implementation is provided for any type implementing `internal::T` and
/// `higher::T`; there should be no reason to change it.
pub trait T: SmsVdpInternal + higher::T {
    /// Hardware method providing access to the VDP's `v` counter.
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
    ///
    /// In this API, it's the responsibility of `run_line` to increment and zero
    /// `v` as needed, using the `v` and `set_v` methods.
    fn read_v(&mut self) -> u8 {
        use self::TvSystem::*;
        use self::Resolution::*;
        let v = self.v();
        let result = match (self.tv_system(), self.resolution(), v) {
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
        // FIXME
        // let id = self.as_ref().id();
        // self.receive(
        //     id,
        //     Memo::ReadV {
        //         actual: v,
        //         reported: result as u8,
        //     },
        // );
        result as u8
    }

    /// Hardware method providing access to the VDP's `h` counter.
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
    fn read_h(&mut self) -> u8 {
        let result = (self.h() >> 1) as u8;
        // let id = self.as_ref().id();
        // let h = self.as_ref().h;
        // self.receive(
        //     id,
        //     Memo::ReadH {
        //         actual: h,
        //         reported: result,
        //     },
        // );
        result
    }

    /// Read from the data port.
    ///
    /// Reads are buffered into the VDP's `data_buffer`. Thus, reading the data
    /// port returns the value of `data_buffer`, while also storing the byte of
    /// VRAM at `address` into the `data_buffer`, and then incrementing the
    /// `address` (that is, incrementing the low 14 bits of `address_register`,
    /// wrapping past 0x3FFF). It also clears the control flag.
    fn read_data(&mut self) -> u8 {
        let current_buffer = self.data_buffer();
        let code_addr = self.code_address();
        let addr = code_addr & 0x3FFF;
        let new_value = unsafe { self.vram_unchecked(addr) };
        self.set_address(addr + 1);
        self.set_data_buffer(new_value);
        self.set_control_flag(false);
        // FIXME
        // self.receive(id, Memo::ReadData(current_buffer));
        current_buffer
    }

    /// Read from the control port.
    ///
    /// This returns the current status flags byte, as well as clearing the
    /// status flags, the control flag, and the line interrupt pending flag.
    fn read_control(&mut self) -> u8 {
        let current_status = self.status_flags();
        self.set_status_flags(0);
        self.set_control_flag(false);
        self.set_line_interrupt_pending(false);

        // FIXME
        // self.receive(id, Memo::ReadControl(current_status));
        current_status
    }

    /// Is the VDP requesting an interrupt?
    fn requesting_mi(&self) -> Option<u8> {
        let frame_interrupt = self.status_flags() & FRAME_INTERRUPT_FLAG != 0;
        let line_interrupt = self.line_interrupt_pending();
        if (frame_interrupt && self.frame_irq_enabled())
            || (line_interrupt && self.line_irq_enabled())
        {
            Some(0xFF)
        } else {
            None
        }
    }

    /// Write to the data port.
    ///
    /// If `code` is 3, this is a CRAM write. For the SMS or SMS2 VDP, that
    /// means write the byte `x` into the CRAM address determined by the low 5
    /// bits of `address`. For the GG VDP, that means: if `address` is even,
    /// record `x` into `cram_latch`. If `address` is odd, we actually do write
    /// to CRAM. Recall that for the GG VDP the CRAM is an array of 32 `u16`s
    /// rather than `u8`s. The address is determined by bits 1-5 of `address`,
    /// the low byte to write is `cram_latch`, and the high byte to write is
    /// `x`. (Yes, this means the value of `address` when it's even is ignored.)
    ///
    /// If `code` is 0, 1, or 2, this is a VRAM write. Write `x` to the VRAM
    /// address given by `address`.
    ///
    /// Either way, we then increment `address` (which of course really means
    /// increment the low 14 bits of `address_register, wrapping past 0x3FFF)
    /// and clear `control_flag`.
    fn write_data(&mut self, x: u8) {
        // FIXME
        // let id = self.as_ref().id();
        // self.receive(id, Memo::WriteData(x));

        // FIXME
        let code = self.code();
        let addr = self.address();

        match (code, self.kind()) {
            (3, Kind::Gg) => {
                if addr & 1 == 0 {
                    self.set_cram_latch(x);
                } else {
                    let latch = self.cram_latch();
                    let val = latch as u16 & ((x as u16) << 8);
                    let actual_address = (addr >> 1) % 32;
                    unsafe {
                        self.set_cram_unchecked(actual_address, val);
                    }
                }
            }
            (3, _) => unsafe {
                self.set_cram_unchecked(addr % 32, x as u16);
            },
            _ => unsafe {
                self.set_vram_unchecked(addr, x);
            },
        }

        self.set_address(addr + 1);
        self.set_control_flag(false);
    }

    /// Write to the control port.
    ///
    /// If the control flag is not set, this will set it and also set the
    /// lower 8 bits of the `code_address` register to `x`.
    ///
    /// Otherwise, This will set the upper 8 bits of the `code_address` register
    /// to `x`. Then, if the upper 2 bits of `x` are 0, will read a byte from
    /// VRAM at `self.address()`, store the result in the data buffer, and then
    /// increment the lower 14 bits of `code_address`. If the upper 2 bits of
    /// `x` are 2, will instead set the register indicated by the low 4 bits of
    /// `x` to the low 8 bits of the `code_address` register. (Registers past 10
    /// are ignored.)
    fn write_control(&mut self, x: u8) {
        if self.control_flag() {
            self.set_control_flag(false);
            let low_byte = self.code_address() & 0xFF;
            let code_addr = low_byte | (x as u16) << 8;
            self.set_code_address(code_addr);
            let code = self.code();
            let addr = self.address();
            if code == 0 {
                let val = unsafe { self.vram_unchecked(addr) };
                self.set_data_buffer(val);
                self.set_address(addr + 1);
            } else if code == 2 {
                let which_reg = x & 0xF;
                if which_reg < 11 {
                    unsafe {
                        self.set_register_unchecked(which_reg as u16, low_byte as u8);
                    }
                }
            }
        } else {
            self.set_control_flag(true);
            let high_byte = self.code_address() & 0xFF00;
            self.set_code_address(high_byte | x as u16);
        }
    }
}

impl<S> T for S
where
    S: SmsVdpInternal + higher::T,
{
}
