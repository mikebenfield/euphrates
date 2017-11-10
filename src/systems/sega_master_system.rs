// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

// use std;

// use hardware::memory_map::{MasterSystemMemoryMap, SegaMemoryMap, CodemastersMemoryMap};
// use hardware::z80::Z80Interpreter;
// use hardware::z80::Z80;
// use hardware::io::sms2::Sms2Io;

// pub struct MasterSystem<MemoryMap>(Z80<Sms2Io<MemoryMap>>);

// impl<MemoryMap> MasterSystem<MemoryMap>
// where
//     MemoryMap: MasterSystemMemoryMap
// {
//     fn new(rom: &[u8]) {
//         let memory_map = 
//     }
// }

// pub fn new_system_sega_memory_map() -> MasterSystem<SegaMemoryMap> {
//     unimplemented!()
// }

// pub fn new_system_codemasters_memory_map() -> MasterSystem<CodemastersMemoryMap> {
//     unimplemented!()
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
// pub enum MemoryMap {
//     Sega,
//     Codemasters,
// }

// impl Default for MemoryMap {
//     fn default() -> Self {
//         MemoryMap::Sega
//     }
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
// pub enum Receiver {
//     Nothing,
//     Disassembling,
// }

// impl Default for Receiver {
//     fn default() -> Self {
//         Receiver::Nothing
//     }
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
// pub enum Frequency {
//     Ntsc,
//     Pal,
//     MasterFrequency(u64),
//     Z80Frequency(u64),
//     Fps(u64),
//     Unlimited,
// }

// impl Default for Frequency {
//     fn default() -> Self {
//         Frequency::Ntsc
//     }
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
// pub enum Z80Emulator {
//     Interpreter,
// }

// impl Default for Z80Emulator {
//     fn default() -> Self {
//         Z80Emulator::Interpreter
//     }
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
// pub enum VdpKind {
//     Sms,
//     Sms2,
//     Gg,
// }

// impl Default for VdpKind {
//     fn default() -> Self {
//         VdpKind::Sms
//     }
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
// pub struct MasterSystemBuilder<MemoryMap> {
//     vdp_kind: VdpKind,
//     phantom: std::marker::PhantomData<MemoryMap>,
// }

// impl<MemoryMap> Default for MasterSystemBuilder<MemoryMap> {
//     fn default() -> Self {
//         MasterSystemBuilder {
//             vdp_kind: Default::default(),
//             phantom: std::marker::PhantomData,
//         }
//     }
// }

// fn new_master_system_builder() -> MasterSystemBuilder<SegaMemoryMap> {
//     MasterSystemBuilder {
//         vdp_kind: VdpKind::Sms,
//         phantom: std::marker::PhantomData,
//     }
// }

// impl<MemoryMap> MasterSystemBuilder<MemoryMap> {
//     pub fn sega_memory_map(&self) -> MasterSystemBuilder<SegaMemoryMap> {
//         self.memory_map()
//     }

//     pub fn codemasters_memory_map(&self) -> MasterSystemBuilder<CodemastersMemoryMap> {
//         self.memory_map()
//     }

//     pub fn memory_map<NewMemoryMap>(&self) -> MasterSystemBuilder<NewMemoryMap> {
//         MasterSystemBuilder {
//             vdp_kind: VdpKind::Sms,
//             .. Default::default()
//         }
//     }

//     pub fn vdp_kind(&mut self, v: VdpKind) -> &mut Self {
//         self.vdp_kind = v;
//         self
//     }
// }



// pub struct MasterSystemEmulator {

// }

// const NTSC_MASTER_FREQUENCY: u64 = 10738580;

// const PAL_MASTER_FREQUENCY: u64 = 10640685;

// #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
// pub struct MasterSystemEmulatorBuilder<Z80Emulator> {
//     // z80_emulator: Z80Emulator,
//     // memory_map: MemoryMap,
//     frequency: Frequency,
//     phantom2: std::marker::PhantomData<Z80Emulator>,
// }

// impl<Z80Emulator> Default for MasterSystemEmulatorBuilder<Z80Emulator> {
//     fn default() -> Self {
//         MasterSystemBuilder {
//             frequency: Default::default(),
//             phantom1: std::marker::PhantomData,
//             phantom2: std::marker::PhantomData,
//         }
//     }
// }

// pub fn new_emulator_builder() -> MasterSystemEmulatorBuilder<Z80Interpreter> {
//     Default::default()
// }

// impl<Z80Emulator> MasterSystemEmulatorBuilder<Z80Emulator> {
//     pub fn frequency(&mut self, f: Frequency) -> &mut Self {
//         self.frequency = f;
//         self
//     }

//     pub fn z80_emulator<NewZ80Emulator>(&self) -> MasterSystemEmulatorBuilder<NewZ80Emulator> {
//         MasterSystemBuilder {
//             frequency: self.frequency,
//             .. Default::default()
//         }
//     }

//     pub fn z80_interpreter(&self) -> MasterSystemBuilder<Z80Interpreter> {
//         self.z80_emulator()
//     }

//     // pub fn build(&self) -> 
// }

// // impl MasterSystemBuilder {
// //     pub fn new() -> Self {
// //         Default::default()
// //     }

// //     pub fn memory_map(&mut self, m: MemoryMap) -> &mut Self {
// //         self.memory_map = m;
// //         self
// //     }

// //     pub fn receiver(&mut self, r: Receiver) -> &mut Self {
// //         self.receiver = r;
// //         self
// //     }

// //     pub fn z80_emulator(&mut self, e: Z80Emulator) -> &mut Self {
// //         self.z80_emulator = e;
// //         self
// //     }

// //     pub fn create() 
// // }

// //         // self.master_frequency = match f {
// //         //     Frequency::Ntsc => Some(NTSC_MASTER_FREQUENCY),
// //         //     Frequency::Pal => Some(PAL_MASTER_FREQUENCY),
// //         //     Frequency::MasterFrequency(x) => Some(x),
// //         //     Frequency::Z80Frequency(x) => Some(3 * x),
// //         //     Frequency::Fps(_) => unimplemented!(),
// //         //     Frequency::Unlimited => None,
// //         // };
// //         // self