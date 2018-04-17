use std::convert::{AsMut, AsRef};
use std;

use failure::ResultExt;

use memo::{Inbox, Payload};
use errors::{Error, SimpleKind};

use super::*;

pub mod manifests {
    use memo::{Descriptions, Manifest, PayloadType};
    use self::Descriptions::*;
    use self::PayloadType::*;

    pub const DEVICE: &'static str = &"SegaMemoryMap";

    pub const ALLOCATE_FIRST_PAGE: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "First RAM page allocated",
        descriptions: Strings(&[]),
        payload_type: U8,
    };

    pub const ALLOCATE_SECOND_PAGE: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Second RAM page allocated",
        descriptions: Strings(&[]),
        payload_type: U8,
    };

    pub const SYSTEM_RAM_WRITE: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Write to system RAM",
        descriptions: Strings(&["logical address", "RAM address", "value"]),
        payload_type: U16,
    };

    pub const CARTRIDGE_RAM_WRITE: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Write to cartridge RAM",
        descriptions: Strings(&["logical address", "RAM address", "value"]),
        payload_type: U16,
    };

    pub const SYSTEM_RAM_READ: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Read from system RAM",
        descriptions: Strings(&["logical address", "RAM address", "value"]),
        payload_type: U16,
    };

    pub const CARTRIDGE_RAM_READ: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Read from cartridge RAM",
        descriptions: Strings(&["logical address", "RAM address", "value"]),
        payload_type: U16,
    };

    pub const ROM_READ_LOGICAL_ADDRESS: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Read from ROM",
        descriptions: Strings(&["logical address", "value"]),
        payload_type: U16,
    };

    pub const ROM_READ: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Read from ROM (giving ROM address)",
        descriptions: Strings(&["ROM address"]),
        payload_type: U32,
    };

    pub const INVALID_WRITE: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Invalid write",
        descriptions: Strings(&["logical address", "value"]),
        payload_type: PayloadType::U16,
    };

    pub const REGISTER_WRITE: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Register write",
        descriptions: Strings(&["register", "value"]),
        payload_type: PayloadType::U16,
    };

    pub const MAP_ROM: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Map ROM",
        descriptions: Strings(&["page", "slot"]),
        payload_type: PayloadType::U16,
    };

    pub const MAP_CARTRIDGE_RAM: &'static Manifest = &Manifest {
        device: DEVICE,
        summary: "Map Cartridge RAM",
        descriptions: Strings(&["page", "slot"]),
        payload_type: PayloadType::U16,
    };
}

pub type Result<T> = std::result::Result<T, Error<SimpleKind>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
enum RamPagesAllocated {
    Zero,
    One,
    Two,
}

use self::RamPagesAllocated::*;

/// The so-called Sega memory map, used in the large majority of games for the
/// Sega Master System.
#[derive(Clone)]
pub struct SegaMemoryMap {
    // memory is a sequence of 8 KiB implementation-pages. The first
    // implementation-page corresponds to the 8 KiB of system memory.
    // Then successive pairs of implementation-pages correspond to
    // 16 KiB sega-pages of cartridge ROM. Then finally, depending on
    // the value of the `ram_pages` field, there may be zero, two, or
    // four implementation pages corresponding to the zero, one, or two
    // sega-pages of cartridge RAM. If there are two sega-pages
    // of cartridge RAM, the first page is at the very end of the sequence,
    // and the second page comes just before it.
    //
    // We don't allocate the cartridge RAM until/unless we need it, which we
    // detect by observing writes to the memory control register at logical
    // address 0xFFFC.
    //
    // Justification: we could store system RAM, cartridge RAM, and cartridge
    // ROM in separate fields. But for read and write access to be efficient, we
    // would like fields of `T` to index, for each logical slot of
    // memory, directly into the physical memory required. This can't be safely
    // done with references in Rust, so we put all pages of memory into a
    // slice and let our slot indices be slice indices.
    //
    // We use 8 KiB implementation-pages, although sega-pages are 16 KiB,
    // because the system RAM is only 8 KiB, and the last two logical memory
    // slots are only 8 KiB, so this way we don't have to make special cases.
    memory: Vec<[u8; 0x2000]>,

    // How many sega-pages of cartridge RAM have we allocated?
    ram_pages_allocated: RamPagesAllocated,

    // The special registers that control the memory map. When writing to
    // `reg_fffc`, we set `reg_fffc` to the actual value written. For the
    // others, which are selectors for the ROM slots, we instead set the
    // register to the sega-page selected, which may be a modulus of the actual
    // value written. (In actual hardware these registers are not readable
    // anyway.)
    reg_fffc: u8,
    reg_fffd: u8,
    reg_fffe: u8,
    reg_ffff: u8,

    // the indices of this array correspond to implementation-slots (8 KiB pages of
    // logical memory), and the members of the array indicate which
    // implementation-page (8 KiB pages of physical memory) that slot should be
    // mapped to, as an index into the `memory` field. Pairs of
    // implementation-slots correspond to 16 KiB sega-slots, except that the
    // last portion of logical memory is divided into two 8 KiB slots, which
    // normally are both mapped to System RAM.
    // These need to be `u16` since conceivably there could be up to 256 sega-pages
    // of cartridge ROM, and we would then need more implementation-pages than that.
    pages: [u16; 8],

    // bitmask, with each bit indicating whether the corresponding slot in the pages field
    // can be written to
    slot_writable: u8,
}

serde_struct_arrays!{
    impl_serde,
    SegaMemoryMap,
    [ram_pages_allocated, reg_fffc, reg_fffd, reg_fffe, reg_ffff, pages, slot_writable,],
    [],
    [memory: [u8; 0x2000],]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum MemoryLocation {
    RomAddress(u32),
    SystemRamAddress(u16),
    CartridgeRamAddress(u16),
}

fn write_check_register<S>(s: &mut S, logical_address: u16, value: u8)
where
    S: AsMut<SegaMemoryMap> + AsRef<SegaMemoryMap> + Inbox,
{
    // macro_rules! receive {
    //     ($x: expr) => {
    //         {
    //             // XXX - need to fix this when I bring back memos
    //             // let id = s.as_ref().id();
    //             // let __y = $x;
    //             // s.receive(id, __y);
    //         }
    //     }
    // }

    macro_rules! ensure_one_page_allocated {
        () => {
            if s.as_ref().ram_pages_allocated == Zero {
                manifests::ALLOCATE_FIRST_PAGE.send(
                    s,
                    Payload::U8([0,0,0,0,0,0,0,0])
                );
                let smm = s.as_mut();
                smm.memory.push([0; 0x2000]);
                smm.memory.push([0; 0x2000]);
                smm.ram_pages_allocated = One;
                smm.memory.shrink_to_fit();
            }
        }
    }

    macro_rules! ensure_two_pages_allocated {
        () => {
            if s.as_ref().ram_pages_allocated == Zero {
                manifests::ALLOCATE_FIRST_PAGE.send(
                    s,
                    Payload::U8([0,0,0,0,0,0,0,0])
                );
                manifests::ALLOCATE_SECOND_PAGE.send(
                    s,
                    Payload::U8([0,0,0,0,0,0,0,0])
                );
                let smm = s.as_mut();
                smm.memory.push([0; 0x2000]);
                smm.memory.push([0; 0x2000]);
                smm.memory.push([0; 0x2000]);
                smm.memory.push([0; 0x2000]);
                smm.memory.shrink_to_fit();
            } else if s.as_ref().ram_pages_allocated == One {
                manifests::ALLOCATE_SECOND_PAGE.send(
                    s,
                    Payload::U8([0,0,0,0,0,0,0,0])
                );
                let smm = s.as_mut();
                assert!(smm.memory.len() >= 3);
                // the first sega-page of cartridge RAM needs to come last, so
                // push it over
                let first_position = smm.memory.len() - 2;
                smm.memory.insert(first_position, [0; 0x2000]);
                smm.memory.insert(first_position + 1, [0; 0x2000]);
                smm.memory.shrink_to_fit();
            }
            s.as_mut().ram_pages_allocated = Two;
        }
    }

    let rom_impl_page_count = match s.as_ref().ram_pages_allocated {
        // subtract off 1 for the system memory impl_page, and two for each
        // sega_page of ram allocated
        Zero => s.as_ref().memory.len() - 1,
        One => s.as_ref().memory.len() - 3,
        Two => s.as_ref().memory.len() - 5,
    };

    // debug_assert!(rom_impl_page_count % 2 == 0);

    // there are at most 0x100 sega-pages of ROM, so there should be at most
    // 0x200 implementation-pages
    debug_assert!(rom_impl_page_count < 0x200);

    let rom_sega_page_count = (rom_impl_page_count / 2) as u8;

    if rom_sega_page_count.count_ones() != 1 {
        // XXX Since I'm not really sure what is the right thing to do in this
        // case, I'll log it as a fault
        // log_fault!(
        //     "T: ROM size not a power of two: {:0>2X} sega-pages",
        //     rom_sega_page_count
        // );
    }

    // XXX is this the right thing to do?
    // It's correct when `rom_sega_page_count` is a power of two, but who knows
    // what happens in actual hardware when it's not?
    let sega_page = if rom_sega_page_count == 0 {
        0
    } else {
        value % rom_sega_page_count
    };

    let impl_page = (sega_page as u16) * 2 + 1;

    match logical_address {
        0xFFFC => {
            // RAM mapping and misc register
            // XXX - there is an unimplemented feature in which, if bit 4 is
            // set, the fist sega-page of Cartridge RAM is mapped into sega-slot
            // 3. But "no known software" uses this feature.
            manifests::REGISTER_WRITE.send(s, Payload::U16([0xFFFC, value as u16, 0, 0]));
            let impl_page = match value & 0b1100 {
                0b1000 => {
                    // sega-slot 2 mapped to sega-page 0 of cartridge RAM
                    ensure_one_page_allocated!();
                    manifests::MAP_CARTRIDGE_RAM.send(s, Payload::U16([0, 2, 0, 0]));
                    let smm = s.as_mut();
                    smm.slot_writable |= 1 << 4;
                    smm.slot_writable |= 1 << 5;
                    (smm.memory.len() - 2) as u16
                }
                0b1100 => {
                    // sega-slot 2 mapped to sega-page 1 of cartridge RAM
                    ensure_two_pages_allocated!();
                    manifests::MAP_CARTRIDGE_RAM.send(s, Payload::U16([1, 2, 0, 0]));
                    let smm = s.as_mut();
                    smm.slot_writable |= 1 << 4;
                    smm.slot_writable |= 1 << 5;
                    (smm.memory.len() - 4) as u16
                }
                _ => {
                    // sega-slot 2 mapped to page of ROM indicated by register
                    // 0xFFFF
                    let reg = s.as_ref().reg_ffff as u16;
                    manifests::MAP_ROM.send(s, Payload::U16([reg, 2, 0, 0]));
                    let smm = s.as_mut();
                    smm.slot_writable &= !(1 << 4);
                    smm.slot_writable &= !(1 << 5);
                    (smm.reg_ffff as u16) * 2 + 1
                }
            };
            let smm = s.as_mut();
            smm.pages[4] = impl_page;
            smm.pages[5] = impl_page + 1;
            smm.reg_fffc = value;
        }
        0xFFFD => {
            manifests::REGISTER_WRITE.send(s, Payload::U16([0xFFFD, value as u16, 0, 0]));
            manifests::MAP_ROM.send(s, Payload::U16([sega_page as u16, 0, 0, 0]));
            let smm = s.as_mut();
            smm.pages[0] = impl_page;
            smm.pages[1] = impl_page + 1;
            smm.slot_writable &= !(1 << 0);
            smm.slot_writable &= !(1 << 1);
            smm.reg_fffd = sega_page;
        }
        0xFFFE => {
            manifests::REGISTER_WRITE.send(s, Payload::U16([0xFFFE, value as u16, 0, 0]));
            manifests::MAP_ROM.send(s, Payload::U16([sega_page as u16, 1, 0, 0]));
            let smm = s.as_mut();
            smm.pages[2] = impl_page;
            smm.pages[3] = impl_page + 1;
            smm.slot_writable &= !(1 << 2);
            smm.slot_writable &= !(1 << 3);
            smm.reg_fffe = sega_page;
        }
        0xFFFF => {
            manifests::REGISTER_WRITE.send(s, Payload::U16([0xFFFF, value as u16, 0, 0]));
            if s.as_ref().reg_ffff & 0b1000 == 0 {
                manifests::MAP_ROM.send(s, Payload::U16([sega_page as u16, 1, 0, 0]));
                let smm = s.as_mut();
                smm.pages[4] = impl_page;
                smm.pages[5] = impl_page + 1;
            }
            s.as_mut().reg_ffff = sega_page;
        }
        _ => {}
    }
}

impl SegaMemoryMap {
    /// For use in a `Memo`.
    // Always inline: the result will be passed to a `Inbox`. In the case
    // that the `Inbox` does nothing, hopefully the compiler sees that this
    // code has no side effects and optimizes it away.
    #[inline(always)]
    #[allow(dead_code)]
    fn logical_address_to_memory_location(&self, logical_address: u16) -> MemoryLocation {
        if logical_address < 0x400 {
            return MemoryLocation::RomAddress(logical_address as u32);
        }
        let sega_slot = (logical_address & 0xC000) >> 14; // high order 2 bits
        let physical_address = logical_address & 0x3FFF; // low order 14 bits
        match sega_slot {
            0 => {
                // ROM, page determined by register fffd
                let page = self.reg_fffd as u32;
                return MemoryLocation::RomAddress(page * physical_address as u32);
            }
            1 => {
                // ROM, page determined by register fffe
                let page = self.reg_fffe as u32;
                return MemoryLocation::RomAddress(page * physical_address as u32);
            }
            2 => {
                match self.reg_fffc & 0b1100 {
                    0b1000 => {
                        // mapped to sega-page 0 of cartridge RAM
                        return MemoryLocation::CartridgeRamAddress(physical_address);
                    }
                    0b1100 => {
                        // mapped to sega-page 1 of cartridge RAM
                        return MemoryLocation::CartridgeRamAddress(0x4000 | physical_address);
                    }
                    _ => {
                        // ROM, page determined by register ffff
                        let page = self.reg_ffff as u32;
                        return MemoryLocation::RomAddress(page * physical_address as u32);
                    }
                }
            }
            3 => {
                // System RAM, which is only 8 KiB, mirrored
                return MemoryLocation::SystemRamAddress(physical_address & 0x1FFF);
            }
            _ => {
                unreachable!();
            }
        }
    }
}

/// A memory map for the Sega Master System which uses a ROM image.
pub trait MasterSystemMemory: Sized {
    fn new(rom: &[u8]) -> Result<Self>;

    fn new_from_file(filename: &str) -> Result<Self> {
        use std::fs::File;
        use std::io::Read;

        let mut f = File::open(filename)
            .with_context(|e| SimpleKind(format!("Unable to open ROM file {}: {}", filename, e)))?;

        let mut buf: Vec<u8> = Vec::new();

        f.read_to_end(&mut buf)
            .with_context(|e| SimpleKind(format!("Error reading ROM file {}: {}", filename, e)))?;

        Ok(Self::new(&buf)
            .with_context(|e| SimpleKind(format!("Error from ROM file {}: {}", filename, e)))?)
    }
}

impl MasterSystemMemory for SegaMemoryMap {
    fn new(rom: &[u8]) -> Result<Self> {
        if rom.len() % 0x2000 != 0 || rom.len() == 0 {
            Err(SimpleKind(format!(
                "Invalid Sega Master System ROM size 0x{:0>6X} \
                 (should be a positive multiple of 0x2000)",
                rom.len()
            )))?
        }

        let rom_impl_page_count = rom.len() / 0x2000;

        let mut memory = Vec::with_capacity(1 + rom_impl_page_count);

        // push the system RAM
        memory.push([0; 0x2000]);

        // push the ROM
        for i in 0..rom_impl_page_count {
            let mut impl_page = [0u8; 0x2000];
            impl_page.copy_from_slice(&rom[0x2000 * i..0x2000 * (i + 1)]);
            memory.push(impl_page);
        }

        Ok(SegaMemoryMap {
            memory: memory,
            ram_pages_allocated: Zero,
            // supposedly these registers are undefined after a reset, but
            // in the 315-5235 mapper they take these values
            reg_fffc: 0,
            reg_fffd: 0,
            reg_fffe: 1,
            reg_ffff: 2,
            // which means these are the implementation-pages we map to
            pages: [1, 2, 3, 4, 5, 6, 0, 0],
            // only the system RAM is writable
            slot_writable: 0b11000000,
        })
    }
}

impl AsRef<SegaMemoryMap> for SegaMemoryMap {
    #[inline]
    fn as_ref(&self) -> &SegaMemoryMap {
        self
    }
}

impl AsMut<SegaMemoryMap> for SegaMemoryMap {
    #[inline]
    fn as_mut(&mut self) -> &mut SegaMemoryMap {
        self
    }
}

impl<S> Memory16Impler<S> for SegaMemoryMap
where
    S: AsMut<SegaMemoryMap> + AsRef<SegaMemoryMap> + Inbox,
{
    fn read(s: &mut S, logical_address: u16) -> u8 {
        let result = if logical_address < 0x400 {
            // first KiB of logical memory is always mapped to the first KiB of
            // the first page of ROM
            // Some options for the future to avoid this check:
            // - Store an extra 8 KiB for the zeroth impl-slot, and copy the
            // desired page into it, but replacing the first KiB with the first
            // KiB of the zeroth impl-page.
            // - Use 1 KiB impl-pages, and never remap the zeroth slot. (This is
            // probably the best option.)
            s.as_ref().memory[1][logical_address as usize]
        } else {
            let physical_address = logical_address & 0x1FFF; // low order 13 bits
            let impl_slot = (logical_address & 0xE000) >> 13; // high order 3 bits
            let impl_page = s.as_ref().pages[impl_slot as usize];
            s.as_ref().memory[impl_page as usize][physical_address as usize]
        };
        // XXX - need to fix this when I bring back memos
        let location = s.as_ref()
            .logical_address_to_memory_location(logical_address);

        match location {
            MemoryLocation::SystemRamAddress(address) => manifests::SYSTEM_RAM_READ.send(
                s,
                Payload::U16([logical_address, address, result as u16, 0]),
            ),
            MemoryLocation::CartridgeRamAddress(address) => manifests::CARTRIDGE_RAM_READ.send(
                s,
                Payload::U16([logical_address, address, result as u16, 0]),
            ),
            MemoryLocation::RomAddress(address) => {
                manifests::ROM_READ_LOGICAL_ADDRESS
                    .send(s, Payload::U16([logical_address, result as u16, 0, 0]));
                manifests::ROM_READ.send(s, Payload::U32([address, 0]));
            }
        }
        result
    }

    fn write(s: &mut S, logical_address: u16, value: u8) {
        write_check_register(s, logical_address, value);
        let physical_address = logical_address & 0x1FFF; // low order 13 bits
        let impl_slot = (logical_address & 0xE000) >> 13; // high order 3 bits
        let location = s.as_ref()
            .logical_address_to_memory_location(logical_address);
        if s.as_ref().slot_writable & (1 << impl_slot) != 0 {
            match location {
                MemoryLocation::SystemRamAddress(address) => manifests::SYSTEM_RAM_WRITE
                    .send(s, Payload::U16([logical_address, address, value as u16, 0])),
                MemoryLocation::CartridgeRamAddress(address) => manifests::CARTRIDGE_RAM_WRITE
                    .send(s, Payload::U16([logical_address, address, value as u16, 0])),
                _ => unreachable!("ROM Address marked writable?"),
            }
            let impl_page = s.as_ref().pages[impl_slot as usize];
            s.as_mut().memory[impl_page as usize][physical_address as usize] = value;
        } else {
            manifests::INVALID_WRITE.send(s, Payload::U16([logical_address, value as u16, 0, 0]));
        }
    }
}

// mod tests {
//     use super::*;

//     #[allow(dead_code)]
//     fn build_mmap() -> T {
//         let mut rom = [0u8; 0x10000]; // 64 KiB (8 8KiB impl-pages or 4 16KiB sega-pages)
//         rom[0x2000] = 1;
//         rom[0x4000] = 2;
//         rom[0x6000] = 3;
//         rom[0x8000] = 4;
//         rom[0xA000] = 5;
//         rom[0xC000] = 6;
//         rom[0xE000] = 7;
//         T::new(&rom).unwrap()
//     }

//     #[test]
//     fn read() {
//         let smm = &mut build_mmap();

//         // read impl-slot 0
//         assert!(smm.read(0) == 0);

//         // read impl-slot 1
//         assert!(smm.read(0x2000) == 1);

//         // read impl-slot 2
//         assert!(smm.read(0x4000) == 2);

//         // read impl-slot 3
//         assert!(smm.read(0x6000) == 3);

//         // read impl-slot 4
//         assert!(smm.read(0x8000) == 4);

//         // read impl-slot 5
//         assert!(smm.read(0xA000) == 5);

//         // read impl-slot 6 (should be system memory)
//         assert!(smm.read(0xC000) == 0);

//         // read impl-slot 7 (should be system memory)
//         assert!(smm.read(0xE000) == 0);
//     }

//     #[test]
//     fn reg_ffff() {
//         let smm = &mut build_mmap();
//         smm.write(0xFFFF, 3); // sega-slot 2 should now map to sega-page 3
//         assert!(smm.read(0x8000) == 6);
//         assert!(smm.read(0xA000) == 7);
//         smm.write(0xFFFF, 0); // sega-slot 2 should now map to sega-page 0
//         assert!(smm.read(0x8000) == 0);
//         assert!(smm.read(0xA000) == 1);
//     }

//     #[test]
//     fn reg_fffe() {
//         let smm = &mut build_mmap();
//         smm.write(0xFFFE, 3); // sega-slot 1 should now map to sega-page 3
//         assert!(smm.read(0x4000) == 6);
//         assert!(smm.read(0x6000) == 7);
//         smm.write(0xFFFE, 0); // sega-slot 1 should now map to sega-page 0
//         assert!(smm.read(0x4000) == 0);
//         assert!(smm.read(0x6000) == 1);
//     }

//     #[test]
//     fn reg_fffd() {
//         let smm = &mut build_mmap();
//         smm.write(0xFFFD, 1); // sega-slot 0 should now map to sega-page 1
//         assert!(smm.read(0x0000) == 0); // except the first KiB
//         assert!(smm.read(0x2000) == 3);
//         smm.write(0xFFFD, 0); // sega-slot 0 should now map to sega-page 0
//         assert!(smm.read(0x0000) == 0);
//         assert!(smm.read(0x2000) == 1);
//     }

//     #[test]
//     fn reg_fffc() {
//         let smm = &mut build_mmap();
//         smm.write(0xFFFC, 0b1000); // sega-slot 2 mapped to sega-page 0 of cartridge RAM
//         assert!(smm.read(0x8000) == 0);
//         smm.write(0x8000, 102);
//         assert!(smm.read(0x8000) == 102);

//         smm.write(0xFFFC, 0); // sega-slot 2 mapped back to sega-page 2 of ROM
//         assert!(smm.read(0x8000) == 4);
//         smm.write(0, 17);
//         assert!(smm.read(0x8000) == 4); // which should not be writable

//         smm.write(0xFFFC, 0b1000); // back to sega-page 0 of cartridge RAM
//         assert!(smm.read(0x8000) == 102);

//         smm.write(0xFFFC, 0b1100); // to sega-page 1 of cartridge RAM
//         assert!(smm.read(0x8000) == 0);
//         smm.write(0x8000, 103);
//         assert!(smm.read(0x8000) == 103);

//         smm.write(0xFFFC, 0b1000); // back to sega-page 0 of cartridge RAM
//         assert!(smm.read(0x8000) == 102);
//     }
// }
