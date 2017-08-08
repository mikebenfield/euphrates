
use super::*;

pub trait MemoryMapperHardware: Sized {
    fn new(rom: &[u8], cart_ram_size: usize) -> Result<Self, MemoryMapError>;

    fn new_from_file(filename: String, cart_ram_size: usize) -> Result<Self, MemoryMapError>;

    fn read<M: MemoryMapper0<Hardware = Self>>(m: &mut M, address: u16) -> u8;

    fn write<M: MemoryMapper0<Hardware = Self>>(m: &mut M, address: u16, value: u8);

    fn check_ok<M: MemoryMapper0<Hardware = Self>>(m: &mut M) -> Result<(), MemoryMapError>;
}

pub struct SegaMemoryMapperHardware {
    pub cart_ram: Box<[u8]>,
    pub rom: Box<[u8]>,
    pub system_ram: [u8; 0x2000],
    pub rom_page_mask: u8,
    pub reg_fffc: u8,
    pub reg_fffd: u8,
    pub reg_fffe: u8,
    pub reg_ffff: u8,
    pub errors: Vec<MemoryMapError>,
}

/// Types that want to be a MemoryMapper just need to implement this.
pub trait MemoryMapper0: Log {
    type Hardware;
    fn get_memory_mapper_hardware(&self) -> &Self::Hardware;
    fn get_mut_memory_mapper_hardware(&mut self) -> &mut Self::Hardware;
}

macro_rules! opt_mut {
    (mut $($rest: tt)*) => {
        &mut $($rest)*
    };
    (immut $($rest: tt)*) => {
	    & $($rest)*
    };
}

/// Just to avoid duplicating code for reading and writing to memory (a little
/// ugly to use a macro but it seems the only choices are macro, `unsafe`, or
/// duplicate code.)
macro_rules! sega_memory_mapper_index_fn {
    ($fn_name: ident $mutability: ident) => {
        fn $fn_name(smm: opt_mut!{$mutability SegaMemoryMapperHardware}, i: u16) -> (opt_mut!{$mutability u8}, bool) {
            fn rom_access(
                smm: opt_mut!{$mutability SegaMemoryMapperHardware},
                register: u8,
                i: u16
            ) -> opt_mut!{$mutability u8} {
                let rom_page = (register & smm.rom_page_mask) as usize;
                let rom_address = 0x4000 * rom_page + (i as usize);
                opt_mut!{$mutability smm.rom[rom_address]}
            }

            match i {
                0 ... 0x03FF => {
                    // always the first part of ROM; never paged out
                    (opt_mut!{$mutability smm.rom[i as usize]}, false)
                    },
                0x0400 ... 0x3FFF => {
                    // Slot 1: a page of ROM
                    let reg_val = smm.reg_fffd;
                    let rom_page = (reg_val & smm.rom_page_mask) as usize;
                    let rom_address = 0x4000 * rom_page + (i as usize);
                    (opt_mut!{$mutability smm.rom[rom_address]}, false)
                    // let () = rom_access(smm, reg_val, i);
                    // (opt_mut!{$mutability rom_access(smm, reg_val, i)}, false)
                    },
                0x4000 ... 0x7FFF => {
                    // Slot 2: a page of ROM
                    let reg_val = smm.reg_fffe;
                    (rom_access(smm, reg_val, i - 0x4000), false)
                    }
                0x8000 ... 0xBFFF => {
                    // Slot 3: a page of ROM or cartridge RAM

                    // XXX - there is an unimplemented feature regarding bit 4
                    // of the memory control register. See
                    // http://www.smspower.org/Development/Mappers#RAMMapping
                    // (But according to that page, "no known software" uses
                    // this feature)

                    let j = i - 0x8000;
                    if smm.reg_fffc & (1 << 3) != 0 {
                        // cartridge RAM: calculate which page
                        let page = ((smm.reg_fffc & 2) >> 1) as usize;
                        (opt_mut!{$mutability smm.cart_ram[page * 0x4000 + (j as usize)]}, true)
                    } else {
                        // a page of ROM
                        let reg_val = smm.reg_ffff;
                        (rom_access(smm, reg_val, j), false)
                    }
                }
                _ => {
                    // System RAM
                    let j = i & 0x1FFF;
                    (opt_mut!{$mutability  smm.system_ram[j as usize]}, true)
                }
            }
        }
    }
}

sega_memory_mapper_index_fn!{sega_memory_mapper_index immut}

sega_memory_mapper_index_fn!{sega_memory_mapper_index_mut mut}

impl MemoryMapperHardware for SegaMemoryMapperHardware {
    fn new(rom: &[u8], cart_ram_size: usize) -> Result<SegaMemoryMapperHardware, MemoryMapError> {
        if cart_ram_size.count_ones() > 1 || cart_ram_size < 0x2000 {
            return Err(MemoryMapError {
                msg: format!(
                    "Invalid cart_ram_size {} \
			        (must be at least 8 KB, and a power of two)",
                    cart_ram_size
                ),
            });
        }

        let mut rom_page_mask = 0;
        let page_count = rom.len() / 0x2000;
        while (0xFFFF & rom_page_mask) < page_count - 1 {
            rom_page_mask <<= 1;
            rom_page_mask |= 1;
        }
        Ok(SegaMemoryMapperHardware {
            cart_ram: vec![0; cart_ram_size].into_boxed_slice(),
            rom: rom.to_vec().into_boxed_slice(),
            system_ram: [0; 0x2000],
            rom_page_mask: rom_page_mask as u8,
            reg_fffc: 0,
            reg_fffd: 0,
            reg_fffe: 0,
            reg_ffff: 0,
            errors: Vec::new(),
        })
    }

    fn new_from_file(
        filename: String,
        cart_ram_size: usize,
    ) -> Result<SegaMemoryMapperHardware, MemoryMapError> {
        use std::fs::File;
        use std::io::Read;

        let mut f = File::open(filename)?;
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf)?;

        SegaMemoryMapperHardware::new(&buf[0..], cart_ram_size)
    }

    fn read<M: MemoryMapper0<Hardware = SegaMemoryMapperHardware>>(m: &mut M, address: u16) -> u8 {
        log_minor!(m, "MemoryMapper: read attempt: address {:0>2X}", address);
        let result = *sega_memory_mapper_index(m.get_memory_mapper_hardware(), address).0;
        log_minor!(m, "MemoryMapper: read: {:0>4X}", result);
        result
    }

    fn write<M: MemoryMapper0<Hardware = SegaMemoryMapperHardware>>(
        m: &mut M,
        address: u16,
        value: u8,
    ) {
        log_minor!(
            m,
            "MemoryMapper: write attempt: value {:0>2X}, address {:0>4X}",
            value,
            address);

        if address == 0xFFFC {
            log_major!(
                m,
                "MemoryMapper: write register: value {:0>2X}, reg FFFC",
                value
            );
            m.get_mut_memory_mapper_hardware().reg_fffc = value
        } else if address == 0xFFFD {
            log_minor!(
                m,
                "MemoryMapper: write register: value {:0>2X}, reg FFFD",
                value
            );
            m.get_mut_memory_mapper_hardware().reg_fffd = value
        } else if address == 0xFFFE {
            log_minor!(
                m,
                "MemoryMapper: write register: value {:0>2X}, reg FFFE",
                value
            );
            m.get_mut_memory_mapper_hardware().reg_fffe = value
        } else if address == 0xFFFF {
            log_minor!(
                m,
                "MemoryMapper: write register: value {:0>2X}, reg FFFF",
                value
            );
            m.get_mut_memory_mapper_hardware().reg_ffff = value
        }

        let can_write = {
            let (reference, can_write) =
                sega_memory_mapper_index_mut(m.get_mut_memory_mapper_hardware(), address);
            if can_write {
                *reference = value;
            }
            can_write
        };

        if can_write {
            log_minor!(m, "MemoryMapper: wrote");
        } else {
            log_fault!(m, "MemoryMapper: failed write");
        }
    }

    fn check_ok<M: MemoryMapper0<Hardware = SegaMemoryMapperHardware>>(
        _: &mut M,
    ) -> Result<(), MemoryMapError> {
        unimplemented!();
    }
}

#[derive(Copy)]
pub struct SimpleMemoryMapperHardware {
    pub mem: [u8; 0x10000]
}

impl Default for SimpleMemoryMapperHardware {
    fn default() -> SimpleMemoryMapperHardware {
        SimpleMemoryMapperHardware {
            mem: [0; 0x10000]
        }
    }
}

impl Clone for SimpleMemoryMapperHardware {
    fn clone(&self) -> SimpleMemoryMapperHardware {
        SimpleMemoryMapperHardware {
            mem: self.mem
        }
    }
}

impl std::fmt::Debug for SimpleMemoryMapperHardware {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        format!("SimpleMemoryMapperHardware {{ mem: {:?} (...) }}", &self.mem[0..64]);
        Ok(())
    }
}

impl MemoryMapperHardware for SimpleMemoryMapperHardware {
    fn new(rom: &[u8], cart_ram_size: usize) -> Result<SimpleMemoryMapperHardware, MemoryMapError> {
        let mut mem: [u8; 0x10000] = [0; 0x10000];
        mem[0..rom.len()].copy_from_slice(rom);
        Ok(
            SimpleMemoryMapperHardware {
                mem: mem
            }
        )
    }

    fn new_from_file(filename: String, cart_ram_size: usize) -> Result<SimpleMemoryMapperHardware, MemoryMapError> {
        use std::fs::File;
        use std::io::Read;

        let mut f = File::open(filename)?;
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf)?;

        SimpleMemoryMapperHardware::new(&buf[0..], cart_ram_size)
    }

    fn read<M: MemoryMapper0<Hardware = SimpleMemoryMapperHardware>>(m: &mut M, address: u16) -> u8 {
        m.get_memory_mapper_hardware().mem[address as usize]
    }

    fn write<M: MemoryMapper0<Hardware = SimpleMemoryMapperHardware>>(m: &mut M, address: u16, value: u8) {
        m.get_mut_memory_mapper_hardware().mem[address as usize] = value
    }

    fn check_ok<M: MemoryMapper0<Hardware = SimpleMemoryMapperHardware>>(m: &mut M) -> Result<(), MemoryMapError> {
        Ok(())
    }
}

impl<H: MemoryMapperHardware, M: MemoryMapper0<Hardware = H>> MemoryMapper for M {
    fn read(&mut self, address: u16) -> u8 {
        <H as MemoryMapperHardware>::read(self, address)
    }
    fn write(&mut self, address: u16, value: u8) {
        <H as MemoryMapperHardware>::write(self, address, value)
    }
    fn check_ok(&self) -> Result<(), MemoryMapError> {
        unimplemented!()
    }
}
