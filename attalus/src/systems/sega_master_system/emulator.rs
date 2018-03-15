// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::convert::{AsMut, AsRef};
use std::time::{Duration, Instant};
use std;

use failure::{err_msg, Error, ResultExt};

use hardware::io_16_8;
use hardware::irq::Irq;
use hardware::memory_16_8;
use hardware::sms_vdp::{self, internal::T as internalT, machine::T as machineT};
use hardware::sn76489;
use hardware::z80::{self, machine::T as Z80MachineT};
use host_multimedia::{SimpleAudio, SimpleColor, SimpleGraphics};
use memo::{Inbox, Pausable};
use utilities::{self, FrameInfo, TimeInfo};

pub type Result<T> = std::result::Result<T, Error>;

pub trait MasterSystem
    : z80::machine::T
    + sms_vdp::machine::T
    + memory_16_8::T
    + AsMut<io_16_8::sms2::T>
    + sn76489::machine::T
    + SimpleAudio {
    fn init(&mut self, frequency: Frequency) -> Result<()>;

    fn run_frame(
        &mut self,
        player_status: &PlayerStatus,
        time_status: &TimeStatus,
        frame_info: &mut FrameInfo,
    ) -> Result<EmulationResult>;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Hardware<M> {
    pub z80: z80::state::T,
    pub memory: M,
    pub io: io_16_8::sms2::T,
    pub vdp: sms_vdp::simple::T,
    pub sn76489: sn76489::real::T,
    pub interpreter: z80::interpreter::Interpreter<z80::interpreter::Safe>,
}

pub struct System<I, M> {
    pub inbox: I,
    pub hardware: Hardware<M>,
    pub audio: Box<SimpleAudio>,
    pub graphics: Box<SimpleGraphics>,
    pub z80_frequency: Option<u64>,
}

impl<I, M> MasterSystem for System<I, M>
where
    Self: z80::machine::T
        + sms_vdp::machine::T
        + memory_16_8::T
        + AsMut<io_16_8::sms2::T>
        + sn76489::machine::T
        + Pausable,
{
    fn init(&mut self, frequency: Frequency) -> Result<()> {
        const AUDIO_BUFFER_SIZE: u16 = 0x800;

        self.z80_frequency = match frequency {
            Frequency::Ntsc => Some(NTSC_MASTER_FREQUENCY / 3),
            Frequency::Pal => Some(PAL_MASTER_FREQUENCY / 3),
            Frequency::MasterFrequency(x) => Some(x / 3),
            Frequency::Z80Frequency(x) => Some(x),
            Frequency::Unlimited => None,
        };

        if let Some(frequency) = self.z80_frequency {
            self.configure(frequency as u32 / 16, AUDIO_BUFFER_SIZE)
                .map_err(|s| {
                    format_err!("Master System emulation: error configuring audio: {}", s)
                })?;
        }
        Ok(())
    }

    fn run_frame(
        &mut self,
        player_status: &PlayerStatus,
        time_status: &TimeStatus,
        frame_info: &mut FrameInfo,
    ) -> Result<EmulationResult> {
        // audio already configured, start time, etc

        <Self as AsMut<io_16_8::sms2::T>>::as_mut(self).set_joypad_a(player_status.joypad_a);
        <Self as AsMut<io_16_8::sms2::T>>::as_mut(self).set_joypad_b(player_status.joypad_b);
        <Self as AsMut<io_16_8::sms2::T>>::as_mut(self).set_pause(player_status.pause);

        loop {
            if self.wants_pause() {
                return Ok(EmulationResult::FrameInterrupted);
            }

            self.draw_line()
                .with_context(|e| err_msg(format!("Master System emulation: VDP error {}", e)))?;

            let vdp_cycles = <Self as sms_vdp::internal::T>::cycles(self);
            let z80_target_cycles = 2 * vdp_cycles / 3;

            while z80::internal::T::cycles(self) < z80_target_cycles {
                self.run(z80_target_cycles);
                if self.wants_pause() {
                    return Ok(EmulationResult::FrameInterrupted);
                }
            }

            if self.v() == 0 {
                if let Some(_) = self.z80_frequency {
                    let sound_target_cycles = z80::internal::T::cycles(self) / 16;
                    sn76489::machine::T::queue(self, sound_target_cycles).with_context(|e| {
                        format_err!("Master System emulation: error queueing audio {}", e)
                    })?;
                }

                let time_info = TimeInfo {
                    total_cycles: z80::internal::T::cycles(self),
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

impl<I, M> System<I, M> {
    pub fn new(
        inbox: I,
        hardware: Hardware<M>,
        graphics: Box<SimpleGraphics>,
        audio: Box<SimpleAudio>,
    ) -> Self {
        System {
            z80_frequency: None,
            inbox,
            hardware,
            graphics,
            audio,
        }
    }
}

impl<I, M> SimpleGraphics for System<I, M> {
    #[inline]
    fn set_resolution(&mut self, width: u32, height: u32) -> Result<()> {
        self.graphics.set_resolution(width, height)
    }

    #[inline]
    fn resolution(&self) -> (u32, u32) {
        self.graphics.resolution()
    }

    #[inline]
    fn paint(&mut self, x: u32, y: u32, color: SimpleColor) {
        self.graphics.paint(x, y, color)
    }

    #[inline]
    fn get(&self, x: u32, y: u32) -> SimpleColor {
        self.graphics.get(x, y)
    }

    #[inline]
    fn render(&mut self) -> Result<()> {
        self.graphics.render()
    }
}

impl<I, M> SimpleAudio for System<I, M> {
    #[inline]
    fn configure(&mut self, frequency: u32, buffer_size: u16) -> Result<()> {
        self.audio.configure(frequency, buffer_size)
    }

    #[inline]
    fn play(&mut self) -> Result<()> {
        self.audio.play()
    }

    #[inline]
    fn pause(&mut self) -> Result<()> {
        self.audio.pause()
    }

    #[inline]
    fn buffer(&mut self) -> &mut [i16] {
        self.audio.buffer()
    }

    #[inline]
    fn queue_buffer(&mut self) -> Result<()> {
        self.audio.queue_buffer()
    }

    #[inline]
    fn clear(&mut self) -> Result<()> {
        self.audio.clear()
    }
}

macro_rules! impl_as_ref {
    ($typename: ty, $component_name: ident) => {
        impl<I, M> AsRef<$typename> for System<I, M> {
            #[inline]
            fn as_ref(&self) -> &$typename {
                &self.hardware.$component_name
            }
        }

        impl<I, M> AsMut<$typename> for System<I, M> {
            #[inline]
            fn as_mut(&mut self) -> &mut $typename {
                &mut self.hardware.$component_name
            }
        }
    }
}

impl_as_ref!{io_16_8::sms2::T, io}
impl_as_ref!{sn76489::real::T, sn76489}
impl_as_ref!{sms_vdp::simple::T, vdp}
impl_as_ref!{z80::interpreter::Interpreter<z80::interpreter::Safe>, interpreter}
impl_as_ref!{z80::state::T, z80}

macro_rules! impl_as_ref_memory_map {
    ($typename: ty) => {
        impl<I> AsRef<$typename> for System<I, $typename> {
            #[inline]
            fn as_ref(&self) -> &$typename {
                &self.hardware.memory
            }

        }

        impl<I> AsMut<$typename> for System<I, $typename> {
            #[inline]
            fn as_mut(&mut self) -> &mut $typename {
                &mut self.hardware.memory
            }
        }
    }
}

impl_as_ref_memory_map!{memory_16_8::sega::T}
impl_as_ref_memory_map!{memory_16_8::codemasters::T}

impl<I> memory_16_8::Impl for System<I, memory_16_8::sega::T>
where
    I: Inbox<memory_16_8::sega::Memo>,
{
    type Impler = memory_16_8::sega::T;
}

impl<I> memory_16_8::Impl for System<I, memory_16_8::codemasters::T>
where
    I: Inbox<memory_16_8::sega::Memo>,
{
    type Impler = memory_16_8::codemasters::T;
}

impl<I, M> io_16_8::Impl for System<I, M>
where
    I: Inbox<io_16_8::sms2::Memo>,
{
    type Impler = io_16_8::sms2::T;
}

impl<I, M> sms_vdp::internal::Impl for System<I, M> {
    type Impler = sms_vdp::simple::T;
}

impl<I, M> sms_vdp::machine::Impl for System<I, M> {
    type Impler = sms_vdp::machine::simple::T;
}

impl<I, M> Pausable for System<I, M>
where
    I: Pausable,
{
    #[inline]
    fn wants_pause(&self) -> bool {
        self.inbox.wants_pause()
    }

    #[inline]
    fn clear_pause(&mut self) {
        self.inbox.clear_pause()
    }
}

impl<I, M, T> Inbox<T> for System<I, M>
where
    I: Inbox<T>,
{
    #[inline]
    fn receive(&mut self, id: u32, memo: T) {
        self.inbox.receive(id, memo)
    }
}

impl<I, M> z80::internal::Impl for System<I, M> {
    type Impler = z80::state::T;
}

impl<I, M> z80::higher::T for System<I, M> {}

impl<I, M> z80::part::T for System<I, M>
where
    Self: memory_16_8::T + io_16_8::T
{
    #[inline]
    fn requesting_mi(&self) -> Option<u8> {
        <Self as sms_vdp::part::T>::requesting_mi(&self)
            .or_else(|| self.hardware.io.requesting_mi())
    }

    #[inline]
    fn requesting_nmi(&self) -> bool {
        self.hardware.io.requesting_nmi()
    }

    #[inline]
    fn clear_nmi(&mut self) {
        self.hardware.io.clear_nmi();
    }
}

impl<I, M> z80::machine::Impl for System<I, M>
where
    Self: z80::part::T,
{
    type Impler = z80::interpreter::Interpreter<z80::interpreter::Safe>;
}

impl<I, M> sn76489::hardware::Impl for System<I, M> {
    type Impler = sn76489::real::T;
}

impl<I, M> sn76489::machine::Impl for System<I, M> {
    type Impler = sn76489::real::T;
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
    vdp_kind: sms_vdp::Kind,
    vdp_tv_system: sms_vdp::TvSystem,
}

impl HardwareBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn vdp_sms(&mut self) -> &mut Self {
        self.vdp_kind(sms_vdp::Kind::Sms);
        self
    }

    pub fn vdp_sms2(&mut self) -> &mut Self {
        self.vdp_kind(sms_vdp::Kind::Sms2);
        self
    }

    pub fn vdp_gg(&mut self) -> &mut Self {
        self.vdp_kind(sms_vdp::Kind::Gg);
        self
    }

    pub fn vdp_kind(&mut self, vdp_kind: sms_vdp::Kind) -> &mut Self {
        self.vdp_kind = vdp_kind;
        self
    }

    pub fn vdp_tv_pal(&mut self) -> &mut Self {
        self.vdp_tv_system(sms_vdp::TvSystem::Pal);
        self
    }

    pub fn vdp_tv_ntsc(&mut self) -> &mut Self {
        self.vdp_tv_system(sms_vdp::TvSystem::Ntsc);
        self
    }

    pub fn vdp_tv_system(&mut self, vdp_tv_system: sms_vdp::TvSystem) -> &mut Self {
        self.vdp_tv_system = vdp_tv_system;
        self
    }

    pub fn build<M>(&self, memory: M) -> Hardware<M> {
        Hardware {
            z80: Default::default(),
            io: Default::default(),
            vdp: Default::default(),
            sn76489: Default::default(),
            interpreter: Default::default(),
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

//// Types needed for emulation

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
