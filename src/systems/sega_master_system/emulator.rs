// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::thread;

use ::errors::*;
use ::has::Has;
use ::memo::{Pausable, Inbox};
use ::hardware::irq::Irq;
use ::hardware::vdp;
use ::hardware::memory_16_8;
use ::hardware::z80;
use ::hardware::sn76489;
use ::hardware::io_16_8;
use ::host_multimedia::SimpleAudio;

use super::UserInterface;

pub trait MasterSystem: z80::Machine + vdp::Machine + memory_16_8::Machine
{}

impl<T> MasterSystem for T
where
    T: z80::Machine + vdp::Machine + memory_16_8::Machine
{}

pub struct Hardware<M> {
    pub z80: z80::Component,
    pub memory: M,
    pub io: io_16_8::sms2::Component,
    pub vdp: vdp::Component,
    pub sn76489: sn76489::real::Component,
}

pub struct System<I, M> {
    pub inbox: I,
    pub hardware: Hardware<M>,
}

impl<I, M> System<I, M> {
    pub fn new(inbox: I, hardware: Hardware<M>) -> Self {
        System {
            inbox,
            hardware,
        }
    }
}

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
    I: Inbox<vdp::Memo> + Inbox<io_16_8::sms2::Memo>
{
    type C = io_16_8::sms2::Component;
}

impl<I, M> Pausable for System<I, M>
where
    I: Pausable
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
    I: Inbox<T>
{
    #[inline(always)]
    fn receive(&mut self, id: u32, memo: T) {
        self.inbox.receive(id, memo)
    }
}

impl<I, M> Irq for System<I, M>
{
    #[inline(always)]
    fn requesting_mi(&self) -> Option<u8> {
        self.hardware.vdp.requesting_mi()
    }

    #[inline(always)]
    fn requesting_nmi(&self) -> bool {
        self.hardware.vdp.requesting_nmi()
    }

    #[inline(always)]
    fn clear_nmi(&self) {
        self.hardware.vdp.clear_nmi()
    }
}

impl<I, M> z80::MachineImpl for System<I, M> {}

impl<I, M> vdp::MachineImpl for System<I, M> {}

impl<I, M> sn76489::MachineImpl for System<I, M> {
    type C = sn76489::real::Component;
}

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

    pub fn build<M>(&self, memory: M) -> Hardware<M>
    {
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
        M: memory_16_8::sega::MasterSystemMemory
    {
        let memory = M::new(rom)?;
        Ok(
            self.build(memory)
        )
    }

    pub fn build_from_file<M>(&self, filename: &str) -> Result<Hardware<M>>
    where
        M: memory_16_8::sega::MasterSystemMemory
    {
        let memory = M::new_from_file(filename)?;
        Ok(
            self.build(memory)
        )
    }
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
    ) -> Emulator<Z80Emulator, VdpEmulator>{
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
}

impl<Z80Emulator, VdpEmulator> Emulator<Z80Emulator, VdpEmulator> {
    pub fn run<I, M, HostGraphics>(
        &mut self,
        master_system: &mut System<I, M>,
        graphics: &mut HostGraphics,
        audio: &mut SimpleAudio,
        user_interface: &mut UserInterface,
    ) -> Result<()>
    where
        System<I, M>: MasterSystem,
        Z80Emulator: z80::Emulator<System<I, M>>,
        // System<I, M>: z80::Machine,
        VdpEmulator: vdp::Emulator<HostGraphics>,
        I: Pausable,
    {
        const AUDIO_BUFFER_SIZE: u16 = 0x800;

        if let Some(frequency) = self.z80_frequency {
            audio.configure(
                frequency as u32 / 16,
                AUDIO_BUFFER_SIZE,
            )?;
            audio.play()?;
        }

        let start_time = Instant::now();
        let z80_cycles_start = master_system.hardware.z80.cycles;
        let mut frame_info: FrameInfo = Default::default();

        let mut sn76489_emulator = <sn76489::real::Emulator as Default>::default();

        loop {
            if master_system.inbox.wants_pause() {
                unimplemented!();
            }

            self.vdp_emulator
                .draw_line(&mut master_system.hardware.vdp, graphics)?;

            let vdp_cycles = master_system.hardware.vdp.cycles;
            let z80_target_cycles = 2 * vdp_cycles / 3;

            while master_system.hardware.z80.cycles < z80_target_cycles {
                self.z80_emulator
                    .run(master_system, z80_target_cycles);
            }

            if master_system.hardware.vdp.v == 0 {
                if let Some(_) = self.z80_frequency {
                    let sound_target_cycles = master_system.hardware.z80.cycles / 16;
                    sn76489::Emulator::queue(
                        &mut sn76489_emulator,
                        &mut master_system.hardware.sn76489,
                        sound_target_cycles,
                        audio,
                    )?;
                }

                user_interface.update_user(master_system);

                if user_interface.wants_quit() {
                    return Ok(())
                }

                user_interface.update_player();

                let player_status = user_interface.player_status();
                master_system.hardware.io.set_joypad_a(player_status.joypad_a);
                master_system.hardware.io.set_joypad_b(player_status.joypad_b);

                let time_info = TimeInfo {
                    total_cycles: master_system.hardware.z80.cycles,
                    cycles_start: z80_cycles_start,
                    frequency: self.z80_frequency,
                    start_time: start_time,
                    hold_duration: Duration::from_secs(0),
                };

                frame_info = time_govern(time_info, frame_info);
            }
        }
    }
}

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

const KEEP_FRAMES: usize = 50;

#[derive(Clone, Debug, Default, PartialEq)]
struct FrameInfo {
    last_frames: VecDeque<Instant>,
    fps: f64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct TimeInfo {
    total_cycles: u64,
    cycles_start: u64,
    frequency: Option<u64>,
    start_time: Instant,
    hold_duration: Duration,
}

fn time_govern(time_info: TimeInfo, mut frame_info: FrameInfo) -> FrameInfo {
    debug_assert!(time_info.cycles_start <= time_info.total_cycles);

    let now = Instant::now();

    let new_fps = if frame_info.last_frames.len() < KEEP_FRAMES {
        0.0
    } else {
        let first_instant = frame_info.last_frames[0];
        let duration = now.duration_since(first_instant);
        let duration_secs = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        KEEP_FRAMES as f64 / duration_secs
    };

    frame_info.last_frames.push_back(now);
    if frame_info.last_frames.len() > KEEP_FRAMES {
        frame_info.last_frames.pop_front();
    }

    if let Some(frequency) = time_info.frequency {
        let guest_cycles = time_info.total_cycles - time_info.cycles_start;
        let guest_duration_seconds = guest_cycles / frequency;
        let guest_cycles_remaining = guest_cycles % frequency;
        let guest_duration_subsec_nanos = (1000000000 * guest_cycles_remaining) / frequency;
        let guest_duration =
            Duration::new(guest_duration_seconds, guest_duration_subsec_nanos as u32);

        let host_total_duration = now.duration_since(time_info.start_time);
        debug_assert!(host_total_duration >= time_info.hold_duration);
        let host_active_duration = host_total_duration - time_info.hold_duration;

        if let Some(diff_duration) = guest_duration.checked_sub(host_active_duration) {
            thread::sleep(diff_duration);
        }
    }

    FrameInfo {
        last_frames: frame_info.last_frames,
        fps: new_fps,
    }
}
