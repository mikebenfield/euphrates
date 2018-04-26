use std;
use std::convert::{AsMut, AsRef};

use failure::ResultExt;

use errors::{Error, SimpleKind};
use memo::{Holdable, Inbox, Memo, Payload};

use super::*;

pub mod manifests {
    use self::Descriptions::*;
    use self::PayloadType::*;
    use memo::{Descriptions, Manifest, PayloadType};

    pub const DEVICE: &'static str = &"SegaMemoryMap";

    static ALLOCATE_FIRST_PAGE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "First RAM page allocated",
        descriptions: Strings(&[]),
        payload_type: U8,
    };

    pub static ALLOCATE_FIRST_PAGE: &'static Manifest = &ALLOCATE_FIRST_PAGE_MANIFEST;

    static ALLOCATE_SECOND_PAGE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Second RAM page allocated",
        descriptions: Strings(&[]),
        payload_type: U8,
    };

    pub static ALLOCATE_SECOND_PAGE: &'static Manifest = &ALLOCATE_SECOND_PAGE_MANIFEST;

    static SYSTEM_RAM_WRITE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Write to system RAM",
        descriptions: Strings(&["logical address", "RAM address", "value"]),
        payload_type: U16,
    };

    pub static SYSTEM_RAM_WRITE: &'static Manifest = &SYSTEM_RAM_WRITE_MANIFEST;

    static CARTRIDGE_RAM_WRITE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Write to cartridge RAM",
        descriptions: Strings(&["logical address", "RAM address", "value"]),
        payload_type: U16,
    };

    pub static CARTRIDGE_RAM_WRITE: &'static Manifest = &CARTRIDGE_RAM_WRITE_MANIFEST;

    static SYSTEM_RAM_READ_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Read from system RAM",
        descriptions: Strings(&["logical address", "RAM address", "value"]),
        payload_type: U16,
    };

    pub static SYSTEM_RAM_READ: &'static Manifest = &SYSTEM_RAM_READ_MANIFEST;

    static CARTRIDGE_RAM_READ_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Read from cartridge RAM",
        descriptions: Strings(&["logical address", "RAM address", "value"]),
        payload_type: U16,
    };

    pub static CARTRIDGE_RAM_READ: &'static Manifest = &CARTRIDGE_RAM_READ_MANIFEST;

    static ROM_READ_LOGICAL_ADDRESS_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Read from ROM",
        descriptions: Strings(&["logical address", "value"]),
        payload_type: U16,
    };

    pub static ROM_READ_LOGICAL_ADDRESS: &'static Manifest = &ROM_READ_LOGICAL_ADDRESS_MANIFEST;

    static ROM_READ_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Read from ROM (giving ROM address)",
        descriptions: Strings(&["ROM address"]),
        payload_type: U32,
    };

    pub static ROM_READ: &'static Manifest = &ROM_READ_MANIFEST;

    static INVALID_WRITE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Invalid write",
        descriptions: Strings(&["logical address", "value"]),
        payload_type: PayloadType::U16,
    };

    pub static INVALID_WRITE: &'static Manifest = &INVALID_WRITE_MANIFEST;

    static REGISTER_WRITE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Register write",
        descriptions: Strings(&["register", "value"]),
        payload_type: PayloadType::U16,
    };

    pub static REGISTER_WRITE: &'static Manifest = &REGISTER_WRITE_MANIFEST;

    static MAP_ROM_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Map ROM",
        descriptions: Strings(&["page", "slot"]),
        payload_type: PayloadType::U16,
    };

    pub static MAP_ROM: &'static Manifest = &MAP_ROM_MANIFEST;

    static MAP_CARTRIDGE_RAM_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Map Cartridge RAM",
        descriptions: Strings(&["page", "slot"]),
        payload_type: PayloadType::U16,
    };

    pub static MAP_CARTRIDGE_RAM: &'static Manifest = &MAP_CARTRIDGE_RAM_MANIFEST;
}

pub type Result<T> = std::result::Result<T, Error<SimpleKind>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SystemOrCartridge {
    System,
    Cartridge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum RamOrRom {
    FirstRamPage,
    SecondRamPage,
    Rom(u8),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SegaMemoryMapState {
    pub rom: Box<[u8]>,
    pub system_ram: Box<[u8]>,
    pub cartridge_ram0: Option<Box<[u8]>>,
    pub cartridge_ram1: Option<Box<[u8]>>,

    /// Which ROM page is mapped to slot 0?
    pub slot0: u8,

    /// Which ROM page is mapped to slot 1?
    pub slot1: u8,

    // Which RAM or ROM page is mapped to slot 2?
    pub slot2: RamOrRom,

    pub slot3: SystemOrCartridge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
enum RamPagesAllocated {
    Zero,
    One,
    Two,
}

use self::RamPagesAllocated::*;

/// The so-called Sega memory mapper, used in the large majority of games for the
/// Sega Master System.
///
/// # Hardware
///
/// The remainder of this documentation describes the internal workings of the
/// memory mapper and should not be necessary for consumers of this struct.
///
/// ## Intro
///
/// The Z80 addresses `0x10000` logical bytes (64 KiB), and in this memory map
/// they are divided into 4 `0x4000` (16 Kib) slots. The slots and their
/// function are listed here:
///
/// ```nolang
/// 0x0000 to 0x3FFF: slot 0 (first ROM slot)
/// 0x4000 to 0x7FFF: slot 1 (second ROM slot)
/// 0x8000 to 0xBFFF: slot 2 (ROM/RAM slot)
/// 0xC000 to 0xFFFF: slot 3 (RAM slot)
/// ```
///
/// Slot 4 always maps to the 8 KiB of system RAM, mirrored across the regions
/// `0xC000 to 0xDFFF` and `0xE000 to 0xFFFF`. The other slots, however, can be
/// remapped to other regions of physical memory. This is done using four 8 bit
/// registers of the memory mapper. The register names (which are hexadecimal
/// numerals) and their functions are listed here:
///
/// ```nolang
/// 0xFFFC: cartridge RAM control
/// 0xFFFD: slot 0 control
/// 0xFFFE: slot 1 control
/// 0xFFFF: slot 2 control
/// ```
///
/// These registers can be written to by writing to the corresponding address in
/// logical memory. Such a write also writes to the underlying RAM. Note that
/// since system RAM is mirrored, these physical memory locations can also be
/// accessed at logical addresses `0xDFFFC`, `0xDFFFD`, `0xDFFFE`, and `0xDFFF`.
/// Writes to these addresses will change the underlying memory without changing
/// the registers.
///
/// ## Slot control registers
///
/// I will now describe the use of the slot control registers `0xFFFD`,
/// `0xFFFE`, and `0xFFFF`. Consider the ROM an array of `N` 16 KiB pages. Let
/// `v` be the value mapped to the slot i control register. Then ROM page `v %
/// N` is mapped to slot i. [^modulonote]
///
/// There's one exception to the above, which is that the first 0x400 bytes (1
/// KiB) bytes of slot 0 always map to the first 0x400 bytes of ROM.
///
/// [^modulonote]: Well, as long as the ROM size is a power of 2 and the actual
/// memory mapper (there were several revisions of this memory mapper) in use
/// supported `log_2 N` significant bits. I can't imagine any released games had
/// a memory mapper that didn't support enough bits, but the ROM size not being
/// a power of 2 may be another issue. I'm not sure what the mapper would do in
/// that case. For now I just put 0s to bump it up to a power of 2. I need to
/// experiment.
///
/// ## Cartridge RAM control register
///
/// The Cartridge RAM control register `0xFFFC` is a little more complicated.
/// There are three meaningful bits (where 0 is the least significant bit),
/// listed here:
///
/// ```nolang
/// Bit 2: RAM bank select
/// Bit 3: RAM enable for slot 2
/// Bit 4: RAM enable for slot 3
/// ```
///
/// If bit 4 is set, the first 16 KiB page of cartridge RAM is mapped to slot 3.
///
/// If bit 4 is unset, instead the 8 KiB of system RAM is mapped to slot 3,
/// mirrored across the two regions `0xC000 to 0xDFFF` and `0xE000 to 0xFFFF`.
///
/// If bit 3 is set, either the first 16 KiB page (if bit 2 is set) or the
/// second 16 KiB page (if bit 2 is unset) of cartridge RAM is mapped to slot 2.
/// In this case the value in slot 2 control register `0xFFFF` is ignored.
///
/// If bit 3 is unset, a page of ROM is mapped to slot 2 as determined by slot 2
/// control register `0xFFFF`.
///
/// I believe that cartridge RAM control register `0xFFFC` has the final
/// say about slot 2, so that when bit 3 is set, even if additional writes happen
/// at register `0xFFFF`, slot 2 remains mapped to cartridge RAM.
///
///
/// ## Unimplemented features
///
/// The Sega memory mapper also has a feature called ROM bank shifting, which
/// involves writing to bits 0 and 1 of `0xFFFC` to change the page numbers
/// in the other registers, and a feature called ROM write protection, which
/// was used in development hardware when ROM was emulated by RAM. No known
/// software uses either of these features, so I'm not implementing them.
///
/// ## Sources
///
/// Much of this information comes from the site [SMS Power](http://smspower.org),
/// in particular [this page][SMS Power Memory Map].
///
/// [SMS Power Memory Map]: http://www.smspower.org/Development/Mappers?from=Development.Mapper
#[derive(Clone, Serialize, Deserialize)]
pub struct SegaMemoryMap {
    /// All memory from the ROM, the cartridge RAM, or the system RAM
    /// is in this `Vec`.
    ///
    /// The first `0x2000` bytes (8 KiB) are system RAM. Then comes a chunk of
    /// ROM. Finally, there are either `0` bytes, `0x4000` bytes (16 KiB), or
    /// `0x8000` (32 KiB) of cartridge RAM, which can be determined by looking
    /// at `ram_pages_allocated`.
    ///
    /// Storing all the memory together here allows us to store and use the
    /// memory mapper's banking information as the high bits of an index
    /// into this `Vec`, which is more efficient than branching to choose among
    /// fields for each of ROM, cartridge RAM, and system RAM.
    memory: Vec<u8>,

    /// How many sega-pages of cartridge RAM have we allocated?
    ///
    /// It's apparently not possible to know ahead of time how much RAM was in a
    /// given cartridge, so we simply don't allocate any until cartridge RAM
    /// control register `0xFFFC` is written to, indicating that it should be
    /// mapped in.
    ram_pages_allocated: RamPagesAllocated,

    /// The special registers that control the memory map. When writing to
    /// `reg_fffc`, we set `reg_fffc` to the actual value written. For the
    /// others, which are selectors for the ROM slots, we instead set the
    /// register to the sega-page selected, which may be a modulus of the actual
    /// value written. (In actual hardware these registers are not readable
    /// anyway.)
    reg_fffc: u8,
    reg_fffd: u8,
    reg_fffe: u8,
    reg_ffff: u8,

    /// This array maps implementation-slots to implementation-pages.
    ///
    /// Its ridiculous type is just to make it convenient to derive Serialize
    /// and whatnot; use the `pages` method to get a [u16; 64].
    ///
    /// In documentation of the actual memory mapper, slots (regions of logical
    /// memory) and pages (regions of physical memory) are described as being
    /// `0x4000` bytes (16 KiB) in size. This allows for a brief description
    /// of the mapper as having only four slots. However it necessitates some
    /// exceptions: System RAM is only 8 KiB, so it must be mirrored, and
    /// the first `0x400` bytes (1 KiB) of slot 0 is always mapped to the
    /// first `0x400` bytes of ROM. In order to avoid these kinds of
    /// exceptions in code, I define an implementation-slot
    /// as a 1 KiB region of logical memory, and an implementation-page as
    /// a 1 KiB region of physical memory in the field `memory`.
    ///
    /// I'll use the terms sega-slot and sega-page to refer to the usual
    /// 16 KiB ideas.
    ///
    /// The indices of this array are implementation-slots. The elements of the
    /// array are the high bits of indices into the `memory` array. Thus, for
    /// each 16 bit logical address, its high 6 bits are an index into this
    /// array to determine an implementation-page, and its low 10 bits are the
    /// low 10 bits of the index into `memory`.
    ///
    /// Note that since 8 bits are used to select a ROM sega-page and each
    /// sega-page has 14 bits' worth of addresses, there are effectively
    /// 22 bits' worth of addressable ROM bytes, so that's `0x400000`
    /// bytes (2 MiB) of max ROM (and apparently the largest ROMs are
    /// 1 MiB).
    pages: [[u16; 16]; 4],

    /// Which slots can be written to?
    ///
    /// This is a bitmask, with each bit indicating whether the corresponding
    /// implementation-slot can be written to.
    slot_writable: u64,
}

impl SegaMemoryMap {
    #[inline(always)]
    fn pages(&self) -> &[u16; 64] {
        use std::mem::transmute;
        unsafe { transmute(&self.pages) }
    }

    #[inline(always)]
    fn pages_mut(&mut self) -> &mut [u16; 64] {
        use std::mem::transmute;
        unsafe { transmute(&mut self.pages) }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum MemoryLocation {
    RomAddress(u32),
    SystemRamAddress(u16),
    CartridgeRamAddress(u16),
}

/// Check if `logical_address` refers to a register and, if so, do the register
/// write.
fn write_register<S>(s: &mut S, logical_address: u16, value: u8)
where
    S: AsMut<SegaMemoryMap> + AsRef<SegaMemoryMap> + Inbox,
{
    fn ensure_one_page_allocated<S>(s: &mut S)
    where
        S: AsMut<SegaMemoryMap> + AsRef<SegaMemoryMap> + Inbox,
    {
        if s.as_ref().ram_pages_allocated == Zero {
            manifests::ALLOCATE_FIRST_PAGE.send(s, Payload::U8([0, 0, 0, 0, 0, 0, 0, 0]));
            let smm = s.as_mut();
            let current_len = smm.memory.len();
            smm.memory.resize(current_len + 0x4000, 0);
            smm.ram_pages_allocated = One;
            smm.memory.shrink_to_fit();
        }
    }

    fn ensure_two_pages_allocated<S>(s: &mut S)
    where
        S: AsMut<SegaMemoryMap> + AsRef<SegaMemoryMap> + Inbox,
    {
        if s.as_ref().ram_pages_allocated == Zero {
            manifests::ALLOCATE_FIRST_PAGE.send(s, Payload::U8([0, 0, 0, 0, 0, 0, 0, 0]));
            manifests::ALLOCATE_SECOND_PAGE.send(s, Payload::U8([0, 0, 0, 0, 0, 0, 0, 0]));
            let smm = s.as_mut();
            let current_len = smm.memory.len();
            smm.memory.resize(current_len + 0x8000, 0);
            smm.memory.shrink_to_fit();
        } else if s.as_ref().ram_pages_allocated == One {
            manifests::ALLOCATE_SECOND_PAGE.send(s, Payload::U8([0, 0, 0, 0, 0, 0, 0, 0]));
            let smm = s.as_mut();
            let current_len = smm.memory.len();
            smm.memory.resize(current_len + 0x4000, 0);
            smm.memory.shrink_to_fit();
        }
        s.as_mut().ram_pages_allocated = Two;
    }

    let rom_len = s.as_ref().memory.len() - match s.as_ref().ram_pages_allocated {
        // subtract 0x2000 for the system memory impl_page,
        // and 0x4000 for each sega-page of RAM allocated
        Zero => 0x2000,
        One => 0x6000,
        Two => 0xA000,
    };

    debug_assert!(rom_len > 0);

    let rom_sega_page_count = rom_len / 0x4000;
    // Is this the right thing to do?
    // It's correct when `rom_sega_page_count` is a power of two, but who knows
    // what happens in actual hardware when it's not?
    let rom_sega_page = if rom_sega_page_count == 0 {
        0
    } else {
        value as usize % rom_sega_page_count
    };

    // since `rom_sega_page` is smaller than 0x100, multiplying it by 16 still
    // leaves it smaller than 0x1000, so 16 bits is plenty.
    let rom_impl_page_count = (rom_len / 0x400) as u16;
    let rom_impl_page = (rom_sega_page * 0x10) as u16;

    match logical_address {
        0xFFFC => {
            // Cartridge RAM control
            manifests::REGISTER_WRITE.send(s, Payload::U16([0xFFFC, value as u16, 0, 0]));
            match value & 0b1100 {
                0b1000 => {
                    // sega-slot 2 mapped to sega-page 0 of cartridge RAM
                    ensure_one_page_allocated(s);
                    manifests::MAP_CARTRIDGE_RAM.send(s, Payload::U16([0, 2, 0, 0]));
                    let smm = s.as_mut();
                    let total_impl_pages = smm.memory.len() / 0x400;
                    let start_impl_page = total_impl_pages - match smm.ram_pages_allocated {
                        One => 16,
                        Two => 32,
                        Zero => unreachable!("No pages allocated?"),
                    };
                    for i in 0..16 {
                        smm.pages_mut()[32 + i] = (start_impl_page + i) as u16;
                    }
                    smm.slot_writable |= 0x0000FFFF00000000;
                }
                0b1100 => {
                    // sega-slot 2 mapped to sega-page 1 of cartridge RAM
                    ensure_two_pages_allocated(s);
                    manifests::MAP_CARTRIDGE_RAM.send(s, Payload::U16([1, 2, 0, 0]));
                    let smm = s.as_mut();
                    let total_impl_pages = smm.memory.len() / 0x400;
                    let start_impl_pages = total_impl_pages - 16;
                    for i in 0..16 {
                        smm.pages_mut()[32 + i] = (start_impl_pages + i) as u16;
                    }
                }
                _ => {
                    // sega-slot 2 mapped to sega-page of ROM indicated by register
                    // 0xFFFF
                    let reg = s.as_ref().reg_ffff as u16;
                    let start_impl_page = reg * 16;
                    manifests::MAP_ROM.send(s, Payload::U16([reg, 2, 0, 0]));
                    let smm = s.as_mut();
                    for i in 0..16 {
                        smm.pages_mut()[32 + i] = start_impl_page + i as u16 % rom_impl_page_count;
                    }
                }
            };
            // if value & 0b10000 != 0 {
            //     // cartridge RAM sega-page 0 mapped to sega-slot 3
            //     manifests::MAP_CARTRIDGE_RAM.send(s, Payload::U16([0, 3, 0, 0]));
            //     ensure_one_page_allocated(s);
            //     let smm = s.as_mut();
            //     let total_impl_pages = smm.memory.len() / 0x400;
            //     let start_impl_pages = total_impl_pages - match smm.ram_pages_allocated {
            //         One => 16,
            //         Two => 32,
            //         Zero => unreachable!("No pages allocated?"),
            //     };
            //     for i in 0..16 {
            //         smm.pages_mut()[48 + i] = (start_impl_pages + i) as u16;
            //     }
            // } else {
            //     // XXX
            // }
            s.as_mut().reg_fffc = value;
        }
        0xFFFD => {
            // sega-slot 0 control
            manifests::REGISTER_WRITE.send(s, Payload::U16([0xFFFD, value as u16, 0, 0]));
            manifests::MAP_ROM.send(s, Payload::U16([rom_sega_page as u16, 0, 0, 0]));
            let smm = s.as_mut();
            // leave the first KiB alone, so start at 1
            for i in 1..16 {
                smm.pages_mut()[i] = (rom_impl_page + i as u16) % rom_impl_page_count;
            }
            smm.reg_fffd = rom_sega_page as u8;
        }
        0xFFFE => {
            // sega-slot 1 control
            manifests::REGISTER_WRITE.send(s, Payload::U16([0xFFFE, value as u16, 0, 0]));
            manifests::MAP_ROM.send(s, Payload::U16([rom_sega_page as u16, 1, 0, 0]));
            let smm = s.as_mut();
            for i in 0..16 {
                smm.pages_mut()[i + 16] = (rom_impl_page + i as u16) % rom_impl_page_count;
            }
            smm.reg_fffe = rom_sega_page as u8;
        }
        0xFFFF => {
            // sega-slot 2 control
            manifests::REGISTER_WRITE.send(s, Payload::U16([0xFFFF, value as u16, 0, 0]));
            if s.as_ref().reg_ffff & 0b1000 == 0 {
                // bit 3 of reg 0xFFFF is unset, so we do indeed map ROM
                manifests::MAP_ROM.send(s, Payload::U16([rom_sega_page as u16, 2, 0, 0]));
                let smm = s.as_mut();
                for i in 0..16 {
                    smm.pages_mut()[i + 32] = (rom_impl_page + i as u16) % rom_impl_page_count;
                }
                smm.slot_writable &= 0xFFFF0000FFFFFFFF;
            }
            s.as_mut().reg_ffff = rom_sega_page as u8;
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
                return MemoryLocation::RomAddress(page * 0x4000 + physical_address as u32);
            }
            1 => {
                // ROM, page determined by register fffe
                let page = self.reg_fffe as u32;
                return MemoryLocation::RomAddress(page * 0x4000 + physical_address as u32);
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
                        return MemoryLocation::RomAddress(page * 0x4000 + physical_address as u32);
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
    fn new(mut rom: &[u8]) -> Result<Self> {
        if rom.len() == 0 || rom.len() > 0x400000 {
            Err(SimpleKind(format!(
                "Invalid Sega Master System ROM size 0x{:0>X} \
                 (should be a nonzero and no larger than 0x400000)",
                rom.len()
            )))?
        }

        // Some ROMs have a 512 byte header that shouldn't be there. This
        // apparently is due to a piece of dumping hardware called the Super
        // Magic Drive. So for any ROM of size an odd multiple of 512 bytes,
        // just strip off that header.
        if (rom.len() / 512) & 1 != 0 {
            rom = &rom[512..];
        }

        // I'm not certain whether the SMS actually has this restriction,
        // but I don't know what to do with smaller ROMs.
        if rom.len() < 0x4000 {
            Err(SimpleKind(format!(
                "Invalid Sega Master System ROM size 0x{:0>X} \
                 (should be at least 16 KiB)",
                rom.len()
            )))?
        }

        let mut memory = Vec::with_capacity(rom.len() + 0x2000);

        memory.extend(rom);

        // XXX I need to experiment and figure out what to do with
        // strange ROM sizes.

        memory.resize(rom.len() + 0x2000, 0);

        // There are several different versions of this Sega Memory Mapper. For
        // most of them, the initial values of these registers are undefined,
        // but for the 315-5235, there were fixed initial values. I'd like to
        // just always use those values, but they need at least 3 sega-pages of
        // ROM. If we don't have that, just put everything to 0.
        let (reg_fffc, reg_fffd, reg_fffe, reg_ffff) = if rom.len() >= 3 * 0x4000 {
            (0, 0, 1, 2)
        } else {
            (0, 0, 0, 0)
        };

        // XXX
        let rom_impl_pages = (rom.len() / 0x400) as u16;

        let pages = if rom.len() >= 3 * 0x4000 {
            [
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                [
                    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
                ],
                [
                    32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
                ],
                [
                    rom_impl_pages,
                    rom_impl_pages + 1,
                    rom_impl_pages + 2,
                    rom_impl_pages + 3,
                    rom_impl_pages + 4,
                    rom_impl_pages + 5,
                    rom_impl_pages + 6,
                    rom_impl_pages + 7,
                    rom_impl_pages,
                    rom_impl_pages + 1,
                    rom_impl_pages + 2,
                    rom_impl_pages + 3,
                    rom_impl_pages + 4,
                    rom_impl_pages + 5,
                    rom_impl_pages + 6,
                    rom_impl_pages + 7,
                ],
            ]
        } else {
            [
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                [
                    rom_impl_pages,
                    rom_impl_pages + 1,
                    rom_impl_pages + 2,
                    rom_impl_pages + 3,
                    rom_impl_pages + 4,
                    rom_impl_pages + 5,
                    rom_impl_pages + 6,
                    rom_impl_pages + 7,
                    rom_impl_pages,
                    rom_impl_pages + 1,
                    rom_impl_pages + 2,
                    rom_impl_pages + 3,
                    rom_impl_pages + 4,
                    rom_impl_pages + 5,
                    rom_impl_pages + 6,
                    rom_impl_pages + 7,
                ],
            ]
        };

        Ok(SegaMemoryMap {
            memory: memory,
            ram_pages_allocated: Zero,
            reg_fffc,
            reg_fffd,
            reg_fffe,
            reg_ffff,
            pages,
            // only the system RAM is writable
            slot_writable: 0xFFFF000000000000,
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

impl Holdable for SegaMemoryMap {
    fn holding(&self) -> bool {
        false
    }
}

impl Inbox for SegaMemoryMap {
    fn receive(&mut self, _memo: Memo) {}
}

impl Memory16Impl for SegaMemoryMap {
    type Impler = Self;
}

impl<S> Memory16Impler<S> for SegaMemoryMap
where
    S: AsMut<SegaMemoryMap> + AsRef<SegaMemoryMap> + Inbox,
{
    fn read(s: &mut S, logical_address: u16) -> u8 {
        let impl_slot = logical_address >> 10; // highest 6 bits;
        let impl_page = (s.as_ref().pages()[impl_slot as usize] as usize) << 10;
        let low_bits = (logical_address & 0x03FF) as usize;
        let result = s.as_ref().memory[low_bits | impl_page];

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
        write_register(s, logical_address, value);

        let impl_slot = logical_address >> 10; // highest 6 bits;

        if s.as_ref().slot_writable & (1 << impl_slot) != 0 {
            let location = s.as_ref()
                .logical_address_to_memory_location(logical_address);
            match location {
                MemoryLocation::SystemRamAddress(address) => manifests::SYSTEM_RAM_WRITE
                    .send(s, Payload::U16([logical_address, address, value as u16, 0])),
                MemoryLocation::CartridgeRamAddress(address) => manifests::CARTRIDGE_RAM_WRITE
                    .send(s, Payload::U16([logical_address, address, value as u16, 0])),
                _ => unreachable!("ROM Address marked writable?"),
            }

            let impl_page = (s.as_ref().pages()[impl_slot as usize] as usize) << 10;
            let low_bits = (logical_address & 0x03FF) as usize;
            s.as_mut().memory[low_bits | impl_page] = value;
        } else {
            manifests::INVALID_WRITE.send(s, Payload::U16([logical_address, value as u16, 0, 0]));
        }
    }
}

mod tests {
    use super::*;

    #[allow(dead_code)]
    fn build_mmap() -> SegaMemoryMap {
        let mut rom = [0u8; 0x10000]; // 64 KiB (8 8KiB impl-pages or 4 16KiB sega-pages)
        rom[0x2000] = 1;
        rom[0x4000] = 2;
        rom[0x6000] = 3;
        rom[0x8000] = 4;
        rom[0xA000] = 5;
        rom[0xC000] = 6;
        rom[0xE000] = 7;

        rom[0x9E02] = 100;

        SegaMemoryMap::new(&rom).unwrap()
    }

    #[test]
    fn read() {
        let smm = &mut build_mmap();

        // read impl-slot 0
        assert!(smm.read(0) == 0);

        // read impl-slot 1
        assert!(smm.read(0x2000) == 1);

        // read impl-slot 2
        assert!(smm.read(0x4000) == 2);

        // read impl-slot 3
        assert!(smm.read(0x6000) == 3);

        // read impl-slot 4
        assert!(smm.read(0x8000) == 4);

        // read impl-slot 5
        assert!(smm.read(0xA000) == 5);

        // read impl-slot 6 (should be system memory)
        assert!(smm.read(0xC000) == 0);

        // read impl-slot 7 (should be system memory)
        assert!(smm.read(0xE000) == 0);
    }

    #[test]
    fn reg_ffff() {
        let smm = &mut build_mmap();
        smm.write(0xFFFF, 3); // sega-slot 2 should now map to sega-page 3
        assert!(smm.read(0x8000) == 6);
        assert!(smm.read(0xA000) == 7);
        smm.write(0xFFFF, 0); // sega-slot 2 should now map to sega-page 0
        assert!(smm.read(0x8000) == 0);
        assert!(smm.read(0xA000) == 1);
        smm.write(0xFFFF, 2); // sega-slot 2 should now map to sega-page 2
        assert!(smm.read(0x9E02) == 100);
        assert!(smm.read(0x8000) == 4);
        assert!(smm.read(0xA000) == 5);
    }

    #[test]
    fn reg_fffe() {
        let smm = &mut build_mmap();
        smm.write(0xFFFE, 3); // sega-slot 1 should now map to sega-page 3
        assert!(smm.read(0x4000) == 6);
        assert!(smm.read(0x6000) == 7);
        smm.write(0xFFFE, 0); // sega-slot 1 should now map to sega-page 0
        assert!(smm.read(0x4000) == 0);
        assert!(smm.read(0x6000) == 1);
    }

    #[test]
    fn reg_fffd() {
        let smm = &mut build_mmap();
        smm.write(0xFFFD, 1); // sega-slot 0 should now map to sega-page 1
        assert!(smm.read(0x0000) == 0); // except the first KiB
        assert!(smm.read(0x2000) == 3);
        smm.write(0xFFFD, 0); // sega-slot 0 should now map to sega-page 0
        assert!(smm.read(0x0000) == 0);
        assert!(smm.read(0x2000) == 1);
    }

    #[test]
    fn reg_fffc() {
        let smm = &mut build_mmap();
        smm.write(0xFFFC, 0b1000); // sega-slot 2 mapped to sega-page 0 of cartridge RAM
        assert!(smm.read(0x8000) == 0);
        smm.write(0x8000, 102);
        assert!(smm.read(0x8000) == 102);

        smm.write(0xFFFC, 0); // sega-slot 2 mapped back to sega-page 2 of ROM
        assert!(smm.read(0x8000) == 4);
        smm.write(0, 17);
        assert!(smm.read(0x8000) == 4); // which should not be writable

        smm.write(0xFFFC, 0b1000); // back to sega-page 0 of cartridge RAM
        assert!(smm.read(0x8000) == 102);

        smm.write(0xFFFC, 0b1100); // to sega-page 1 of cartridge RAM
        assert!(smm.read(0x8000) == 0);
        smm.write(0x8000, 103);
        assert!(smm.read(0x8000) == 103);

        smm.write(0xFFFC, 0b1000); // back to sega-page 0 of cartridge RAM
        assert!(smm.read(0x8000) == 102);
    }
}
