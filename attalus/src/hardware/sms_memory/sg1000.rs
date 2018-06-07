use super::*;

/// The SG-1000 mapper.
///
/// Which isn't really much of a mapper.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Sg1000Mapper;

impl<M> SmsMapper<M> for Sg1000Mapper
where
    M: SmsMemory + ?Sized,
{
    #[inline]
    fn write_reg(_memory: &mut M, _address: u16, _value: u8) {}

    fn default_mappings(memory: &mut M) {
        use self::MemoryPage::*;
        memory.set_system_ram_kib(1);
        memory.map_page(0, Rom(0));
        memory.map_page(1, Rom(1));
        memory.map_page(2, Rom(2));
        memory.map_page(3, SystemRam);
    }
}
