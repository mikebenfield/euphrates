// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::time::{Duration, Instant};
use std;

use failure::ResultExt;
use serde::Serialize;
use serde::de::DeserializeOwned;

use errors::{Error, SimpleKind};
use hardware::io_16_8;
use hardware::irq::Irq;
use hardware::memory_16_8;
use hardware::sn76489;
use hardware::vdp;
use hardware::z80;
use has::Has;
use host_multimedia::SimpleAudio;
use memo::NothingInbox;
use memo::{Inbox, Pausable};
use utilities::{self, FrameInfo, Tag, TimeInfo};

pub type Result<T> = std::result::Result<T, Error<SimpleKind>>;

pub trait MasterSystem
    : z80::Machine
    + vdp::Machine
    + memory_16_8::Machine
    + Has<io_16_8::sms2::Component>
    + Has<sn76489::real::Component>
    + Pausable
    + Clone
    + Tag
    + Serialize
    + DeserializeOwned {
}

impl<T> MasterSystem for T
where
    T: z80::Machine
        + vdp::Machine
        + memory_16_8::Machine
        + Has<io_16_8::sms2::Component>
        + Has<sn76489::real::Component>
        + Pausable
        + Clone
        + Tag
        + Serialize
        + DeserializeOwned,
{
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Hardware<M> {
    pub z80: z80::Component,
    pub memory: M,
    pub io: io_16_8::sms2::Component,
    pub vdp: vdp::Component,
    pub sn76489: sn76489::real::Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct System<I, M> {
    pub inbox: I,
    pub hardware: Hardware<M>,
}

impl<I, M> System<I, M> {
    pub fn new(inbox: I, hardware: Hardware<M>) -> Self {
        System { inbox, hardware }
    }
}

macro_rules! impl_tag {
    ($i: ty, $m: ty, $tag: expr) => {
        impl Tag for System<$i, $m> {
            const TAG: &'static str = $tag;
        }
    }
}

impl_tag!{super::DebuggingInbox, memory_16_8::sega::Component, "debugging,sega"}
impl_tag!{NothingInbox, memory_16_8::sega::Component, "nothing,sega"}
impl_tag!{super::DebuggingInbox, memory_16_8::codemasters::Component,
"debugging,codemasters"}
impl_tag!{NothingInbox, memory_16_8::codemasters::Component,
"nothing,codemasters"}

macro_rules! impl_has {
    ($typename: ty, $component_name: ident) => {
        impl<I, M> Has<$typename> for System<I, M> {
            #[inline(always)]
            fn get(&self) -> &$typename {
                &self.hardware.$component_name
            }

            #[inline(always)]
            fn get_mut(&mut self) -> &mut $typename {
                &mut self.hardware.$component_name
            }
        }
    }
}

impl_has!{io_16_8::sms2::Component, io}
impl_has!{sn76489::real::Component, sn76489}
impl_has!{vdp::Component, vdp}
impl_has!{z80::Component, z80}

macro_rules! impl_has_memory_map {
    ($typename: ty) => {
        impl<I> Has<$typename> for System<I, $typename> {
            #[inline(always)]
            fn get(&self) -> &$typename {
                &self.hardware.memory
            }

            #[inline(always)]
            fn get_mut(&mut self) -> &mut $typename {
                &mut self.hardware.memory
            }
        }
    }
}

impl_has_memory_map!{memory_16_8::sega::Component}
impl_has_memory_map!{memory_16_8::codemasters::Component}

impl<I> memory_16_8::MachineImpl for System<I, memory_16_8::sega::Component>
where
    I: Inbox<memory_16_8::sega::Memo>
{
    type C = memory_16_8::sega::Component;
}

impl<I> memory_16_8::MachineImpl for System<I, memory_16_8::codemasters::Component>
where
    I: Inbox<memory_16_8::sega::Memo>
{
    type C = memory_16_8::codemasters::Component;
}

impl<I, M> io_16_8::MachineImpl for System<I, M>
where
    I: Inbox<vdp::Memo> + Inbox<io_16_8::sms2::Memo>,
{
    type C = io_16_8::sms2::Component;
}

impl<I, M> Pausable for System<I, M>
where
    I: Pausable,
{
    #[inline(always)]
    fn wants_pause(&self) -> bool {
        self.inbox.wants_pause()
    }

    #[inline(always)]
    fn clear_pause(&mut self) {
        self.inbox.clear_pause()
    }
}

impl<I, M, T> Inbox<T> for System<I, M>
where
    I: Inbox<T>,
{
    #[inline(always)]
    fn receive(&mut self, id: u32, memo: T) {
        self.inbox.receive(id, memo)
    }
}

impl<I, M> Irq for System<I, M> {
    #[inline(always)]
    fn requesting_mi(&self) -> Option<u8> {
        self.hardware.vdp.requesting_mi().or_else(|| {
            self.hardware.io.requesting_mi()
        })
    }

    #[inline(always)]
    fn requesting_nmi(&self) -> bool {
        self.hardware.vdp.requesting_nmi() || self.hardware.io.requesting_nmi()
    }

    #[inline(always)]
    fn clear_nmi(&mut self) {
        self.hardware.vdp.clear_nmi();
        self.hardware.io.clear_nmi();
    }
}

impl<I, M> z80::MachineImpl for System<I, M> {}

impl<I, M> vdp::MachineImpl for System<I, M> {}

impl<I, M> sn76489::MachineImpl for System<I, M> {
    type C = sn76489::real::Component;
}

//// Builders and other useful types

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Frequency {
    Ntsc,
    Pal,
    MasterFrequency(u64),
    Z80Frequency(u64),
    Unlimited,
}

impl Default for Frequency {
    fn default() -> Self {
        Frequency::Ntsc
    }
}

pub const NTSC_MASTER_FREQUENCY: u64 = 10738580;

pub const PAL_MASTER_FREQUENCY: u64 = 10640685;
#[derive(Copy, Clone, Debug, Default)]
pub struct HardwareBuilder {
    vdp_kind: vdp::Kind,
    vdp_tv_system: vdp::TvSystem,
}

impl HardwareBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn vdp_sms(&mut self) -> &mut Self {
        self.vdp_kind(vdp::Kind::Sms);
        self
    }

    pub fn vdp_sms2(&mut self) -> &mut Self {
        self.vdp_kind(vdp::Kind::Sms2);
        self
    }

    pub fn vdp_gg(&mut self) -> &mut Self {
        self.vdp_kind(vdp::Kind::Gg);
        self
    }

    pub fn vdp_kind(&mut self, vdp_kind: vdp::Kind) -> &mut Self {
        self.vdp_kind = vdp_kind;
        self
    }

    pub fn vdp_tv_pal(&mut self) -> &mut Self {
        self.vdp_tv_system(vdp::TvSystem::Pal);
        self
    }

    pub fn vdp_tv_ntsc(&mut self) -> &mut Self {
        self.vdp_tv_system(vdp::TvSystem::Ntsc);
        self
    }

    pub fn vdp_tv_system(&mut self, vdp_tv_system: vdp::TvSystem) -> &mut Self {
        self.vdp_tv_system = vdp_tv_system;
        self
    }

    pub fn build<M>(&self, memory: M) -> Hardware<M> {
        let mut vdp = vdp::Component::new();
        vdp.kind = self.vdp_kind;
        vdp.tv_system = self.vdp_tv_system;
        Hardware {
            z80: Default::default(),
            io: Default::default(),
            vdp: Default::default(),
            sn76489: Default::default(),
            memory,
        }
    }

    pub fn build_from_rom<M>(&self, rom: &[u8]) -> Result<Hardware<M>>
    where
        M: memory_16_8::sega::MasterSystemMemory,
    {
        let memory = M::new(rom)?;
        Ok(self.build(memory))
    }

    pub fn build_from_file<M>(&self, filename: &str) -> Result<Hardware<M>>
    where
        M: memory_16_8::sega::MasterSystemMemory,
    {
        let memory = M::new_from_file(filename)?;
        Ok(self.build(memory))
    }
}

//// The Emulator and types it needs

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PlayerStatus {
    pub joypad_a: u8,
    pub joypad_b: u8,
    pub pause: bool,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        PlayerStatus {
            joypad_a: 0xFF,
            joypad_b: 0xFF,
            pause: false,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeStatus {
    pub cycles_start: u64,
    pub start_time: Instant,
    pub hold_duration: Duration,
}

impl Default for TimeStatus {
    fn default() -> Self {
        TimeStatus::new(0)
    }
}

impl TimeStatus {
    pub fn new(cycles_start: u64) -> Self {
        TimeStatus {
            cycles_start,
            start_time: Instant::now(),
            hold_duration: Duration::from_millis(0),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EmulationResult {
    FrameCompleted,
    FrameInterrupted,
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash)]
pub struct Emulator<Z80Emulator, VdpEmulator> {
    z80_frequency: Option<u64>,
    z80_emulator: Z80Emulator,
    vdp_emulator: VdpEmulator,
}

impl<Z80Emulator, VdpEmulator> Emulator<Z80Emulator, VdpEmulator> {
    pub fn new(
        frequency: Frequency,
        z80_emulator: Z80Emulator,
        vdp_emulator: VdpEmulator,
    ) -> Emulator<Z80Emulator, VdpEmulator> {
        use self::Frequency::*;
        let z80_frequency = match frequency {
            Ntsc => Some(NTSC_MASTER_FREQUENCY / 3),
            Pal => Some(PAL_MASTER_FREQUENCY / 3),
            MasterFrequency(x) => Some(x / 3),
            Z80Frequency(x) => Some(x),
            Unlimited => None,
        };
        Emulator {
            z80_frequency,
            z80_emulator,
            vdp_emulator,
        }
    }

    pub fn configure_audio<A>(&self, audio: &mut A) -> Result<()>
    where
        A: SimpleAudio,
    {
        const AUDIO_BUFFER_SIZE: u16 = 0x800;

        if let Some(frequency) = self.z80_frequency {
            audio
                .configure(frequency as u32 / 16, AUDIO_BUFFER_SIZE)
                .map_err(|s| {
                    SimpleKind(format!(
                        "Master System emulation: error configuring audio: {}",
                        s
                    ))
                })?;
        }
        Ok(())
    }

    pub fn run_frame<S, HostGraphics>(
        &mut self,
        master_system: &mut S,
        graphics: &mut HostGraphics,
        audio: &mut SimpleAudio,
        player_status: &PlayerStatus,
        time_status: &TimeStatus,
        frame_info: &mut FrameInfo,
    ) -> Result<EmulationResult>
    where
        S: MasterSystem,
        Z80Emulator: z80::Emulator<S>,
        VdpEmulator: vdp::Emulator<HostGraphics>,
    {
        // audio already configured, start time, etc

        <S as Has<io_16_8::sms2::Component>>::get_mut(master_system)
            .set_joypad_a(player_status.joypad_a);
        <S as Has<io_16_8::sms2::Component>>::get_mut(master_system)
            .set_joypad_b(player_status.joypad_b);
        <S as Has<io_16_8::sms2::Component>>::get_mut(master_system).set_pause(player_status.pause);

        // XXX - probably should change to have the sn76489 emulator be a
        // field of the emulator
        let mut sn76489_emulator: sn76489::real::Emulator = Default::default();

        loop {
            if master_system.wants_pause() {
                return Ok(EmulationResult::FrameInterrupted);
            }

            self.vdp_emulator
                .draw_line(<S as Has<vdp::Component>>::get_mut(master_system), graphics)
                .with_context(|e| {
                    SimpleKind(format!("Master System emulation: VDP error {}", e))
                })?;

            let vdp_cycles = <S as Has<vdp::Component>>::get(master_system).cycles;
            let z80_target_cycles = 2 * vdp_cycles / 3;

            while <S as Has<z80::Component>>::get(master_system).cycles < z80_target_cycles {
                self.z80_emulator.run(master_system, z80_target_cycles);
                if master_system.wants_pause() {
                    return Ok(EmulationResult::FrameInterrupted);
                }
            }

            if <S as Has<vdp::Component>>::get(master_system).v == 0 {
                if let Some(_) = self.z80_frequency {
                    let sound_target_cycles = <S as Has<z80::Component>>::get(master_system)
                        .cycles / 16;
                    sn76489::Emulator::queue(
                        &mut sn76489_emulator,
                        <S as Has<sn76489::real::Component>>::get_mut(master_system),
                        sound_target_cycles,
                        audio,
                    ).with_context(|e| {
                        SimpleKind(format!(
                            "Master System emulation: error queueing audio {}",
                            e
                        ))
                    })?;
                }

                let time_info = TimeInfo {
                    total_cycles: <S as Has<z80::Component>>::get(master_system).cycles,
                    cycles_start: time_status.cycles_start,
                    frequency: self.z80_frequency,
                    start_time: time_status.start_time,
                    hold_duration: time_status.hold_duration,
                };

                *frame_info = utilities::time_govern(time_info, frame_info.clone());

                return Ok(EmulationResult::FrameCompleted);
            }
        }
    }
}
