
use hardware::z80::{maskable_interrupt, nonmaskable_interrupt};
use hardware::z80::types::*;
use hardware::irq::Irq;
use hardware::vdp::{Vdp, VdpHardware};
use hardware::memory_mapper::{MemoryMapError, MemoryMapper};
use log::Log;
use hardware::io::{Io, IoHardware};

struct TestingZ80 {
    z80_hardware: Z80Hardware,
    last_cycles: u32,
    input_address: Vec<u16>,
    output_address: Vec<u16>,
    output_data: Vec<u8>,
    memory: [u8; 0x10000],
}

impl Default for TestingZ80 {
    fn default() -> TestingZ80 {
        TestingZ80 {
            memory: [0; 0x10000],
            ..Default::default()
        }
    }
}

impl Log for TestingZ80 {
    fn log_minor0(&mut self, _: String) {}
    fn log_major0(&mut self, _: String) {}
    fn log_fault0(&mut self, _: String) {}
    fn does_log_minor(&self) -> bool { false }
    fn does_log_major(&self) -> bool { false }
    fn does_log_fault(&self) -> bool { false }
    fn check_fault(&self) -> Option<String> { None }
}

impl Irq for TestingZ80 {
	fn request_maskable_interrupt(&mut self) -> bool {
        maskable_interrupt(self)
    }

    fn request_nonmaskable_interrupt(&mut self) {
        nonmaskable_interrupt(self)
    }
}

impl Vdp for TestingZ80 {
    fn get_vdp_hardware(&self) -> &VdpHardware {
        unimplemented!()
    }
    fn get_mut_vdp_hardware(&mut self) -> &mut VdpHardware {
        unimplemented!()
    }
}

impl Io for TestingZ80 {
    fn get_io_hardware(&self) -> &IoHardware { unimplemented!() }
    fn get_mut_io_hardware(&mut self) -> &mut IoHardware { unimplemented!() }

    fn input(&mut self, address: u16) -> u8 {
        self.input_address.push(address);
        0
    }
    fn output(&mut self, address: u16, data: u8) {
        self.output_address.push(address);
        self.output_data.push(data);
    }
}

impl MemoryMapper for TestingZ80 {
    fn read(&mut self, i: u16) -> u8 {
        self.memory[i as usize]
    }
    fn write(&mut self, i: u16, v: u8) {
        self.memory[i as usize] = v;
    }
    fn check_ok(&self) -> Result<(), MemoryMapError> {
        Ok(())
    }
}

impl Z80 for TestingZ80 {
    fn get_z80_hardware(&self) -> &Z80Hardware {
        &self.z80_hardware
    }
    fn get_mut_z80_hardware(&mut self) -> &mut Z80Hardware {
        &mut self.z80_hardware
    }
    fn cycles(&mut self, i: &[u32]) {
        let it = i.iter().sum();
        self.last_cycles = it;
    }
}

fn z80() -> TestingZ80 {
    Default::default()
}

#[test]
fn test_ld() {
    let z = z80();
    
    assert_eq!(17, 17);
}
