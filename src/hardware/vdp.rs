use bits::*;
use log::Log;

use super::irq::Irq;

#[derive(Copy)]
pub struct VdpHardware {
    status_flags: u8,
    irq_counter: u8,
    h: u16,
    v0: u16,
    read: u8,
    code_address: u16,
    registers: [u8; 16],
    cram: [u8; 32],
    vram: [u8; 0x4000],
}

// bits in status_flags
// note that only INT, OVR, and COL are part of the VDP hardware. The rest are
// garbage bits on the SMS VDP. We're using them for other purposes.
const INT: u8 = 7; // Frame interrupt pending
const OVR: u8 = 6; // Sprite overflow
const COL: u8 = 5; // Sprite collision
const LINT: u8 = 4; // Line interrupt pending
const CFLAG: u8 = 3; // Control flag - set after first write to control port

impl Default for VdpHardware {
    fn default() -> VdpHardware {
        VdpHardware {
            status_flags: 0,
            irq_counter: 0,
            h: 0,
            v0: 0,
            read: 0,
            code_address: 0,
            registers: [0; 16],
            cram: [0; 32],
            vram: [0; 0x4000],
        }
    }
}

impl Clone for VdpHardware {
    fn clone(&self) -> VdpHardware {
        *self
    }
}

/// The VDP needs access to an Irq line and a Log. It gets that access
/// through this trait.
pub trait Vdp: Log + Irq {
    fn get_vdp_hardware(&self) -> &VdpHardware;
    fn get_mut_vdp_hardware(&mut self) -> &mut VdpHardware;

    fn read_v(&mut self) -> u8 {
        let v0 = self.get_vdp_hardware().v0;

        log_minor!(self, "Vdp: read v: {:0>4X}", v0);

        if v0 <= 0xDA {
            v0 as u8
        } else {
            (v0 - 6) as u8
        }

    }

    fn read_h(&mut self) -> u8 {
        let h = self.get_vdp_hardware().h;

        log_minor!(self, "Vdp: read h: {:0>4X}", h);

        (h >> 1) as u8
    }

    fn is_requesting_interrupt(&self) -> bool {
        let vdp = self.get_vdp_hardware();
        let frame_irq =
            (vdp.registers[1] & 0x20) != 0 && vdp.status_flags & (1 << INT) != 0;
        let line_irq =
            (vdp.registers[0] & 0x10) != 0 && vdp.status_flags & (1 << LINT) != 0;
        frame_irq || line_irq
    }

    fn write_control(&mut self, x: u8) {
        log_minor!(self, "Vdp: write control: {:0>2X}", x);

        let vdp = self.get_mut_vdp_hardware();
        if vdp.status_flags & (1 << CFLAG) != 0 {
            vdp.code_address = (x as u16) << 8
                | (vdp.code_address & 0xFF);
            let code = vdp.code_address & 0xC000;
            if code == 0 {
                // code value 0: read vram
                let addr = vdp.code_address & 0x3FFF;
                vdp.read = vdp.vram[addr as usize];
                if addr == 0x3FFF { // wrap
                    vdp.code_address &= 0xC000
                } else {
                    vdp.code_address = code | (addr + 1);
                }
            } else if code == 0x8000 {
                // code value 2: register write
                let reg_number = (vdp.code_address & 0xF00) >> 8;
                let val = vdp.code_address & 0xFF;
                vdp.registers[reg_number as usize] = val as u8;
            }
        } else {
            vdp.code_address = (x as u16)
                | (vdp.code_address & 0xFF00);
        }
        vdp.status_flags ^= 1 << CFLAG;
    }

    fn read_control(&mut self) -> u8 {
        let result: u8;
        {
            let vdp = self.get_mut_vdp_hardware();
            result = vdp.status_flags;
            vdp.status_flags = 0;
        }

        log_minor!(self, "Vdp: read control: {:0>2X}", result);

        result
    }

    fn write_data(&mut self, x: u8) {
        log_minor!(self, "Vdp: write data: {:0>2X}", x);

        let vdp = self.get_mut_vdp_hardware();
        clear_bit(&mut vdp.status_flags, CFLAG);
        let code = vdp.code_address & 0xC000;
        if code == 0xC000 {
            // CRAM
            let addr = vdp.code_address & 0x1F;
            vdp.cram[addr as usize] = x;
        } else {
            // VRAM
            let addr = vdp.code_address & 0x3FFF;
            vdp.vram[addr as usize] = x;
        }
        let addr = vdp.code_address & 0x3FFF;
        if addr == 0x3FFF {
            vdp.code_address = code;
        } else {
            vdp.code_address = code | (addr + 1);
        }
    }

    fn read_data(&mut self) -> u8 {
        let result: u8;
        {
            let vdp = self.get_mut_vdp_hardware();
            clear_bit(&mut vdp.status_flags, CFLAG);
            result = vdp.read;
            let addr = vdp.code_address & 0x3FFF;
            let code = vdp.code_address & 0xC000;
            if addr == 0x3FFF {
                vdp.code_address = code;
            } else {
                vdp.code_address = code | (addr + 1);
            }
            vdp.read = vdp.vram[addr as usize];
        }

        log_minor!(self, "Vdp: read data: {:0>4X}", result);

        result
    }
}

pub trait Canvas {
    fn paint(&mut self, x: usize, y: usize, color: u8);
}

pub struct NoCanvas;

impl Canvas for NoCanvas {
    fn paint(&mut self, _: usize, _: usize, _: u8) {}
}

#[allow(unused_variables)]
pub fn draw_line<C: Canvas, V: Vdp>(
    v: &mut V,
    canvas: &mut C,
) -> u32 {
    log_minor!(v, "Vdp: draw line");

    {
        let vdp = v.get_mut_vdp_hardware();

        let nt_address = ((vdp.registers[2] & 0x0E) as usize) << 10;
        let sat_address = ((vdp.registers[5] & 0x7E) as usize) << 7;
        let overscan_color: u16 = (vdp.registers[7] & 0x0F) as u16;
        let x_starting_column: u16 = 32 - ((vdp.registers[8] & 0xF8) as u16 >> 3);
        let x_scroll: u16 = (vdp.registers[8] & 0x07) as u16;
        let y_starting_column: u16 = (vdp.registers[9] & 0xF8) as u16 >> 3;
        let y_scroll: u16 = (vdp.registers[9] & 0x07) as u16;

        let line = vdp.v0 as usize;

        if vdp.v0 <= 192 { // yes, 192 (one past active display region) is right
            if vdp.irq_counter == 0 {
                // line interrupt
                vdp.status_flags |= 1 << LINT;
                vdp.irq_counter = vdp.registers[10];
            } else {
                vdp.irq_counter -= 1;
            }
        } else {
            vdp.irq_counter = vdp.registers[10];
        }
        if vdp.v0 == 0xC1 {
            // frame interrupt
            vdp.status_flags |= 1 << INT;
        }
        vdp.v0 += 1;
        if vdp.v0 == 262 {
            vdp.v0 = 0;
        }

        if line >= 192 {
            // we are out of the active display region
            return 684;
        }

        let mut line_colors = [0x80u8; 256];

        //// first, draw sprites to line_colors
        let mut sprites_drawn = 0;
        for sprite_index in 0..64 {
            let y = vdp.vram[sat_address + sprite_index] as usize + 1;
            let x = (vdp.vram[sat_address + 0x80 + 2*sprite_index] as isize) -
                if 0 != vdp.registers[0] & 0x08 {
                    8
                } else {
                    0
                };

            let n = vdp.vram[sat_address + 0x81 + 2*sprite_index] as usize;
            if y == 0xD1 {
                // such a y coordinate has a special meaning in 192-line mode: don't
                // render any more sprites this line
                break;
            }
            if line < y || line >= y + 8 {
                continue;
            }
            if sprites_drawn == 8 {
                // only draw 8 sprites per line, and if more are scheduled to be
                // drawn, set the overflow flag
                vdp.status_flags |= 1 << OVR;
                break;
            }

            sprites_drawn += 1;

            // the index of the line within the pattern we need to draw
            let sprite_line = line - y;

            // we have 8 pixels, each of which needs a byte of color
            let pattern_byte0 = vdp.vram[32*n + sprite_line];
            let pattern_byte1 = vdp.vram[32*n + sprite_line + 1];
            let pattern_byte2 = vdp.vram[32*n + sprite_line + 2];
            let pattern_byte3 = vdp.vram[32*n + sprite_line + 3];
            for pixel in 0..8u8 {
                let mut palette_index = 0u8;
                // pixel 0 will be the leftmost pixel to draw... but that is
                // found in the most significant bit of each byte
                assign_bit(&mut palette_index, 0, pattern_byte0, 7 - pixel);
                assign_bit(&mut palette_index, 1, pattern_byte1, 7 - pixel);
                assign_bit(&mut palette_index, 2, pattern_byte2, 7 - pixel);
                assign_bit(&mut palette_index, 3, pattern_byte3, 7 - pixel);

                if palette_index == 0 {
                    // for sprites, this means transparency
                    continue;
                }

                // sprites use the second palette
                let color = vdp.cram[0x10 + palette_index as usize];

                // the x coordinate of the canvas where this pixel will be drawn
                let x0 = x + 7 - (pixel as isize);

                if x0 < 0 || x0 > 255 {
                    // I *think* pixels outside this range don't count for sprite
                    // collision
                    continue;
                }

                if line_colors[x0 as usize] != 0x80 {
                    // sprite collision
                    // also, the earlier sprite gets priority
                    set_bit(&mut vdp.status_flags, COL);
                } else {
                    line_colors[x0 as usize] = color;
                }
            }
        }

        // Now we can actually draw
        for i in 0..256usize {
            canvas.paint(i, line, line_colors[i]);
        }
    }

    if v.is_requesting_interrupt() {
        v.request_maskable_interrupt();
    }

    684
}
