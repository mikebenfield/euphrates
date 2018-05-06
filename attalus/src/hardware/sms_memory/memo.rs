use std::fmt::{Display, Error, Formatter};

use hardware::memory16::Memory16;
use impler::{ConstOrMut, Impler, ImplerImpl};
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
pub struct OutboxSmsMemoryImpler<T: ?Sized>(ConstOrMut<T>);

unsafe impl<T: ?Sized> ImplerImpl for OutboxSmsMemoryImpler<T> {
    type T = T;

    #[inline]
    unsafe fn new(c: ConstOrMut<Self::T>) -> Self {
        OutboxSmsMemoryImpler(c)
    }

    #[inline]
    fn get(&self) -> &ConstOrMut<Self::T> {
        &self.0
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ConstOrMut<Self::T> {
        &mut self.0
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
        let page = self._0().page(slot);
        let offset = logical_address & 0x3FFF;
        let value = self.mut_0().read(logical_address);

        self.mut_0().receive(From::from(match page {
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

        let slot = (logical_address >> 14) as u8;
        let page = self._0().page(slot);
        let offset = logical_address & 0x3FFF;

        let main_cartridge_ram_len = self._0().main_cartridge_ram_len();
        let half_cartridge_ram_len = self._0().half_cartridge_ram_len();

        self.mut_0().write(logical_address, value);

        match (main_cartridge_ram_len, self._0().main_cartridge_ram_len()) {
            (0, 0x4000) => {
                self.mut_0().receive(From::from(AllocateFirstPage));
            }
            (0, 0x8000) => {
                self.mut_0().receive(From::from(AllocateFirstPage));
                self.mut_0().receive(From::from(AllocateSecondPage));
            }
            (0x4000, 0x8000) => {
                self.mut_0().receive(From::from(AllocateSecondPage));
            }
            _ => {}
        }
        if half_cartridge_ram_len == 0 && self._0().half_cartridge_ram_len() != 0 {
            self.mut_0().receive(From::from(AllocateHalfPage));
        }

        self.mut_0().receive(From::from(match page {
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
