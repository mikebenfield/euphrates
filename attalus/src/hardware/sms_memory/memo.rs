use std::fmt::{Display, Error, Formatter};

use impler::{Cref, Mref};

use hardware::memory16::Memory16;
use memo::Inbox;

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SmsMemoryMemo {
    AllocateFirstPage,
    AllocateSecondPage,
    AllocateHalfPage,
    MemoryMap {
        slot: u8,
        page: MemoryPage,
    },
    CartridgeRamRead {
        logical_address: u16,
        actual_address: u16,
        value: u8,
    },
    CartridgeRamWrite {
        logical_address: u16,
        actual_address: u16,
        value: u8,
    },
    SystemRamRead {
        logical_address: u16,
        actual_address: u16,
        value: u8,
    },
    SystemRamWrite {
        logical_address: u16,
        actual_address: u16,
        value: u8,
    },
    RomRead {
        logical_address: u16,
        actual_address: u32,
        value: u8,
    },
    RomWrite {
        logical_address: u16,
        actual_address: u32,
        value: u8,
    },
}

impl Display for SmsMemoryMemo {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use self::SmsMemoryMemo::*;

        let s = *self;
        display_branch!(s, f, AllocateFirstPage);
        display_branch!(s, f, AllocateSecondPage);
        display_branch!(s, f, AllocateHalfPage);
        display_branch!(
            s,
            f,
            MemoryMap {
                slot: u8,
                page: MemoryPage
            }
        );
        display_branch!(
            s,
            f,
            CartridgeRamRead {
                logical_address: u16,
                actual_address: u16,
                value: u8
            }
        );
        display_branch!(
            s,
            f,
            CartridgeRamWrite {
                logical_address: u16,
                actual_address: u16,
                value: u8
            }
        );
        display_branch!(
            s,
            f,
            SystemRamRead {
                logical_address: u16,
                actual_address: u16,
                value: u8
            }
        );
        display_branch!(
            s,
            f,
            SystemRamWrite {
                logical_address: u16,
                actual_address: u16,
                value: u8
            }
        );
        display_branch!(
            s,
            f,
            RomRead {
                logical_address: u16,
                actual_address: u32,
                value: u8
            }
        );
        display_branch!(
            s,
            f,
            RomWrite {
                logical_address: u16,
                actual_address: u32,
                value: u8
            }
        );
        unreachable!();
    }
}

/// An Impler for both `SmsMemory` and `Memory16` if you have an `Inbox` and
/// want to receive memos.
pub struct OutboxSmsMemoryImpler<T>(Ref<T>);

impl<T> OutboxSmsMemoryImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(OutboxSmsMemoryImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(OutboxSmsMemoryImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T> Memory16 for OutboxSmsMemoryImpler<T>
where
    T: Memory16 + SmsMemory + Inbox,
    T::Memo: From<SmsMemoryMemo>,
{
    #[inline]
    fn read(&mut self, logical_address: u16) -> u8 {
        use self::MemoryPage::*;
        use self::SmsMemoryMemo::*;

        let slot = (logical_address >> 14) as u8;
        let page = self.0._0().page(slot);
        let offset = logical_address & 0x3FFF;
        let value = self.0.mut_0().read(logical_address);

        self.0.mut_0().receive(From::from(match page {
            SystemRam => SystemRamRead {
                logical_address,
                actual_address: offset & 0x1FFF,
                value,
            },
            FirstCartridgeRam(_) => CartridgeRamRead {
                logical_address,
                actual_address: offset,
                value,
            },
            SecondCartridgeRam(_) => CartridgeRamRead {
                logical_address,
                actual_address: offset + 0x4000,
                value,
            },
            HalfCartridgeRam(rom_page) => if offset >= 0x2000 {
                CartridgeRamRead {
                    logical_address,
                    actual_address: offset - 0x2000,
                    value,
                }
            } else {
                RomRead {
                    logical_address,
                    actual_address: rom_page as u32 * 0x4000 + offset as u32,
                    value,
                }
            },
            Rom(page) => RomRead {
                logical_address,
                actual_address: page as u32 * 0x4000 + offset as u32,
                value,
            },
            RomButFirstKiB(page) => {
                let actual_page = if offset < 0x400 {
                    0 as u32
                } else {
                    page as u32
                };
                RomRead {
                    logical_address,
                    actual_address: actual_page as u32 * 0x4000 + offset as u32,
                    value,
                }
            }
        }));
        value
    }

    #[inline]
    fn write(&mut self, logical_address: u16, value: u8) {
        use self::MemoryPage::*;
        use self::SmsMemoryMemo::*;

        let mem = self.0.mut_0();

        let slot = (logical_address >> 14) as u8;
        let page = mem.page(slot);
        let offset = logical_address & 0x3FFF;

        let main_cartridge_ram_len = mem.main_cartridge_ram_len();
        let half_cartridge_ram_len = mem.half_cartridge_ram_len();

        mem.write(logical_address, value);

        match (main_cartridge_ram_len, mem.main_cartridge_ram_len()) {
            (0, 0x4000) => {
                mem.receive(From::from(AllocateFirstPage));
            }
            (0, 0x8000) => {
                mem.receive(From::from(AllocateFirstPage));
                mem.receive(From::from(AllocateSecondPage));
            }
            (0x4000, 0x8000) => {
                mem.receive(From::from(AllocateSecondPage));
            }
            _ => {}
        }
        if half_cartridge_ram_len == 0 && mem.half_cartridge_ram_len() != 0 {
            mem.receive(From::from(AllocateHalfPage));
        }

        mem.receive(From::from(match page {
            SystemRam => SystemRamWrite {
                logical_address,
                actual_address: offset & 0x1FFF,
                value,
            },
            FirstCartridgeRam(_) => CartridgeRamWrite {
                logical_address,
                actual_address: offset,
                value,
            },
            SecondCartridgeRam(_) => CartridgeRamWrite {
                logical_address,
                actual_address: offset + 0x4000,
                value,
            },
            HalfCartridgeRam(rom_page) => if offset >= 0x2000 {
                CartridgeRamWrite {
                    logical_address,
                    actual_address: offset - 0x2000,
                    value,
                }
            } else {
                RomWrite {
                    logical_address,
                    actual_address: rom_page as u32 * 0x4000 + offset as u32,
                    value,
                }
            },
            Rom(page) => RomWrite {
                logical_address,
                actual_address: page as u32 * 0x4000 + offset as u32,
                value,
            },
            RomButFirstKiB(page) => {
                let actual_page = if offset < 0x400 {
                    0 as u32
                } else {
                    page as u32
                };
                RomWrite {
                    logical_address,
                    actual_address: actual_page as u32 * 0x4000 + offset as u32,
                    value,
                }
            }
        }));
    }
}
