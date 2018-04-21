use std;
use std::any::Any;
use std::convert::{AsMut, AsRef};
use std::time::{Duration, Instant};

use failure::{err_msg, Error, ResultExt};

use hardware::io16::{sms2::Sms2Io, Io16, Io16Impl};
use hardware::irq::Irq;
use hardware::memory16::codemasters::CodemastersMemoryMap;
use hardware::memory16::sega::{MasterSystemMemory, SegaMemoryMap};
use hardware::memory16::{Memory16, Memory16Impl};
use hardware::sms_vdp::{SimpleSmsVdp, SimpleSmsVdpInternal, SmsVdp, SmsVdpImpl, SmsVdpInternal,
                        SmsVdpInternalImpl, SmsVdpState};
use hardware::sn76489::{SimpleSn76489, Sn76489, Sn76489Impl, Sn76489InternalImpl};
use hardware::z80::{self, Z80, Z80Impl, Z80Internal, Z80InternalImpl, Z80Interpreter, Z80Irq,
                    Z80State};
use host_multimedia::{SimpleAudio, SimpleColor, SimpleGraphics};
use memo::{HoldableImpl, Inbox, InboxImpl, NothingInbox};
use utilities::{self, TimeInfo};

use super::inbox::{Debugger, DebuggerImpl, DebuggingInbox, HoldingDebugger};

pub type Result<T> = std::result::Result<T, Error>;

pub trait MasterSystem<R>:
    Z80
    + SmsVdp
    + Memory16
    + Io16
    + Sn76489
    + Debugger
    + Inbox
    + AsMut<Sms2Io>
    + AsRef<TimeStatus>
    + AsMut<TimeStatus>
{
    fn run_frame(&mut self, player_status: PlayerStatus) -> Result<EmulationResult> {
        <Self as AsMut<Sms2Io>>::as_mut(self).set_joypad_a(player_status.joypad_a);
        <Self as AsMut<Sms2Io>>::as_mut(self).set_joypad_b(player_status.joypad_b);
        <Self as AsMut<Sms2Io>>::as_mut(self).set_pause(player_status.pause);

        let z80_frequency = self.frequency();

        loop {
            if self.holding() {
                return Ok(EmulationResult::FrameInterrupted);
            }

            self.draw_line()
                .with_context(|e| err_msg(format!("SimpleSystem emulation: VDP error {}", e)))?;

            let vdp_cycles = <Self as SmsVdpInternal>::cycles(self);
            let z80_target_cycles = 2 * vdp_cycles / 3;

            while Z80Internal::cycles(self) < z80_target_cycles {
                self.run(z80_target_cycles);
                if self.holding() {
                    return Ok(EmulationResult::FrameInterrupted);
                }
            }

            if self.v() == 0 {
                if let Some(_) = z80_frequency {
                    let sound_target_cycles = Z80Internal::cycles(self) / 16;
                    Sn76489::queue(self, sound_target_cycles).with_context(|e| {
                        format_err!("SimpleSystem emulation: error queueing audio {}", e)
                    })?;
                }

                let time_status = *AsRef::<TimeStatus>::as_ref(self);

                let time_info = TimeInfo {
                    total_cycles: Z80Internal::cycles(self),
                    cycles_start: time_status.cycles_start,
                    frequency: z80_frequency,
                    start_time: time_status.start_time,
                    hold_duration: time_status.hold_duration,
                };

                utilities::time_govern(time_info);

                return Ok(EmulationResult::FrameCompleted);
            }
        }
    }

    fn init(&mut self) -> Result<()>;

    fn state(&self) -> MasterSystemState;

    fn borrow_resource(&self) -> &R;

    fn borrow_resource_mut(&mut self) -> &mut R;

    fn reclaim_resource(self: Box<Self>) -> R;

    fn frequency(&self) -> Option<u64>;
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MasterSystemMemoryType {
    Sega(SegaMemoryMap),
    Codemasters(CodemastersMemoryMap),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MasterSystemState {
    pub z80: Z80State,
    pub memory: MasterSystemMemoryType,
    pub io: Sms2Io,
    pub sms_vdp: SmsVdpState,
    pub sn76489: SimpleSn76489,
}

impl MasterSystemState {
    pub fn new_with_sega_memory_map(rom: &[u8]) -> Result<MasterSystemState> {
        let mem = <SegaMemoryMap as MasterSystemMemory>::new(rom)?;
        Ok(MasterSystemState {
            z80: Default::default(),
            memory: MasterSystemMemoryType::Sega(mem),
            io: Default::default(),
            sms_vdp: Default::default(),
            sn76489: Default::default(),
        })
    }

    pub fn new_with_codemasters_memory_map(rom: &[u8]) -> Result<MasterSystemState> {
        let mem = <CodemastersMemoryMap as MasterSystemMemory>::new(rom)?;
        Ok(MasterSystemState {
            z80: Default::default(),
            memory: MasterSystemMemoryType::Codemasters(mem),
            io: Default::default(),
            sms_vdp: Default::default(),
            sn76489: Default::default(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimpleMultimediaResource<A, G> {
    pub audio: A,
    pub graphics: G,
    pub debug: bool,
    pub frequency: Frequency,
}

struct SimpleSystem<I, M, A, G> {
    time_status: TimeStatus,
    inbox: I,
    resource: SimpleMultimediaResource<A, G>,
    z80: Z80State,
    memory: M,
    io: Sms2Io,
    sms_vdp: SmsVdpState,
    sn76489: SimpleSn76489,
    interpreter: Z80Interpreter<z80::Safe>,
}

pub trait MasterSystemResource<'a>
where
    Self: 'a,
{
    fn create(
        self,
        state: MasterSystemState,
        time_status: TimeStatus,
    ) -> Box<MasterSystem<Self> + 'a>;
}

impl<'a, A, G> MasterSystemResource<'a> for SimpleMultimediaResource<A, G>
where
    A: SimpleAudio + 'a,
    G: SimpleGraphics + 'a,
{
    fn create(
        self,
        state: MasterSystemState,
        time_status: TimeStatus,
    ) -> Box<MasterSystem<Self> + 'a> {
        macro_rules! new_system {
            ($debug_type:ident, $memory:ident) => {
                Box::new(SimpleSystem {
                    time_status,
                    inbox: $debug_type::default(),
                    resource: self,
                    z80: state.z80,
                    memory: $memory,
                    io: state.io,
                    sms_vdp: state.sms_vdp,
                    sn76489: state.sn76489,
                    interpreter: Default::default(),
                })
            };
        }

        match state.memory {
            MasterSystemMemoryType::Sega(memory) => if self.debug {
                new_system!(DebuggingInbox, memory)
            } else {
                new_system!(HoldingDebugger, memory)
            },
            MasterSystemMemoryType::Codemasters(memory) => if self.debug {
                new_system!(DebuggingInbox, memory)
            } else {
                new_system!(HoldingDebugger, memory)
            },
        }
    }
}

impl<I, M, A, G> MasterSystem<SimpleMultimediaResource<A, G>> for SimpleSystem<I, M, A, G>
where
    Self: Z80
        + SmsVdp
        + Memory16
        + Io16
        + Sn76489
        + Debugger
        + Inbox
        + SimpleAudio
        + SimpleGraphics,
    M: Any + 'static,
{
    fn init(&mut self) -> Result<()> {
        const AUDIO_BUFFER_SIZE: u16 = 0x800;

        if let Some(frequency) = self.frequency() {
            self.configure(frequency as u32 / 16, AUDIO_BUFFER_SIZE)
                .map_err(|s| {
                    format_err!("SimpleSystem emulation: error configuring audio: {}", s)
                })?;
        };

        self.time_status.cycles_start = Z80Internal::cycles(self);
        self.time_status.start_time = Instant::now();
        self.play()?;

        Ok(())
    }

    fn state(&self) -> MasterSystemState {
        let any: &Any = &self.memory;
        match (
            any.downcast_ref::<SegaMemoryMap>(),
            any.downcast_ref::<CodemastersMemoryMap>(),
        ) {
            (Some(mm), _) => MasterSystemState {
                z80: self.z80.clone(),
                memory: MasterSystemMemoryType::Sega(mm.clone()),
                io: self.io.clone(),
                sms_vdp: self.sms_vdp.clone(),
                sn76489: self.sn76489.clone(),
            },
            (_, Some(mm)) => MasterSystemState {
                z80: self.z80.clone(),
                memory: MasterSystemMemoryType::Codemasters(mm.clone()),
                io: self.io.clone(),
                sms_vdp: self.sms_vdp.clone(),
                sn76489: self.sn76489.clone(),
            },
            _ => unreachable!("Unknown memory map?"),
        }
    }

    fn borrow_resource(&self) -> &SimpleMultimediaResource<A, G> {
        &self.resource
    }

    fn borrow_resource_mut(&mut self) -> &mut SimpleMultimediaResource<A, G> {
        &mut self.resource
    }

    fn reclaim_resource(self: Box<Self>) -> SimpleMultimediaResource<A, G> {
        self.resource
    }

    fn frequency(&self) -> Option<u64> {
        self.resource.frequency.to_z80_frequency()
    }
}

impl<I, M, A, G> SimpleGraphics for SimpleSystem<I, M, A, G>
where
    G: SimpleGraphics,
{
    #[inline]
    fn set_resolution(&mut self, width: u32, height: u32) -> Result<()> {
        self.resource.graphics.set_resolution(width, height)
    }

    #[inline]
    fn resolution(&self) -> (u32, u32) {
        self.resource.graphics.resolution()
    }

    #[inline]
    fn paint(&mut self, x: u32, y: u32, color: SimpleColor) {
        self.resource.graphics.paint(x, y, color)
    }

    #[inline]
    fn get(&self, x: u32, y: u32) -> SimpleColor {
        self.resource.graphics.get(x, y)
    }

    #[inline]
    fn render(&mut self) -> Result<()> {
        self.resource.graphics.render()
    }
}

impl<I, M, A, G> SimpleAudio for SimpleSystem<I, M, A, G>
where
    A: SimpleAudio,
{
    #[inline]
    fn configure(&mut self, frequency: u32, buffer_size: u16) -> Result<()> {
        self.resource.audio.configure(frequency, buffer_size)
    }

    #[inline]
    fn play(&mut self) -> Result<()> {
        self.resource.audio.play()
    }

    #[inline]
    fn pause(&mut self) -> Result<()> {
        self.resource.audio.pause()
    }

    #[inline]
    fn buffer(&mut self) -> &mut [i16] {
        self.resource.audio.buffer()
    }

    #[inline]
    fn queue_buffer(&mut self) -> Result<()> {
        self.resource.audio.queue_buffer()
    }

    #[inline]
    fn clear(&mut self) -> Result<()> {
        self.resource.audio.clear()
    }
}

macro_rules! impl_as_ref {
    ($typename:ty, $component_name:ident) => {
        impl<I, M, A, G> AsRef<$typename> for SimpleSystem<I, M, A, G> {
            #[inline]
            fn as_ref(&self) -> &$typename {
                &self.$component_name
            }
        }

        impl<I, M, A, G> AsMut<$typename> for SimpleSystem<I, M, A, G> {
            #[inline]
            fn as_mut(&mut self) -> &mut $typename {
                &mut self.$component_name
            }
        }
    };
}

impl_as_ref!{Sms2Io, io}
impl_as_ref!{SimpleSn76489, sn76489}
impl_as_ref!{SmsVdpState, sms_vdp}
impl_as_ref!{Z80Interpreter<z80::Safe>, interpreter}
impl_as_ref!{Z80State, z80}
impl_as_ref!{TimeStatus, time_status}

macro_rules! impl_as_ref_memory_map {
    ($typename:ty) => {
        impl<I, A, G> AsRef<$typename> for SimpleSystem<I, $typename, A, G> {
            #[inline]
            fn as_ref(&self) -> &$typename {
                &self.memory
            }
        }

        impl<I, A, G> AsMut<$typename> for SimpleSystem<I, $typename, A, G> {
            #[inline]
            fn as_mut(&mut self) -> &mut $typename {
                &mut self.memory
            }
        }
    };
}

impl_as_ref_memory_map!{SegaMemoryMap}
impl_as_ref_memory_map!{CodemastersMemoryMap}

macro_rules! impl_as_ref_inbox {
    ($typename:ty) => {
        impl<M, A, G> AsRef<$typename> for SimpleSystem<$typename, M, A, G> {
            #[inline]
            fn as_ref(&self) -> &$typename {
                &self.inbox
            }
        }

        impl<M, A, G> AsMut<$typename> for SimpleSystem<$typename, M, A, G> {
            #[inline]
            fn as_mut(&mut self) -> &mut $typename {
                &mut self.inbox
            }
        }
    };
}

impl_as_ref_inbox!{HoldingDebugger}
impl_as_ref_inbox!{DebuggingInbox}

impl<M, A, G> HoldableImpl for SimpleSystem<HoldingDebugger, M, A, G> {
    type Impler = HoldingDebugger;
}

impl<M, A, G> InboxImpl for SimpleSystem<HoldingDebugger, M, A, G> {
    type Impler = HoldingDebugger;
}

impl<M, A, G> HoldableImpl for SimpleSystem<DebuggingInbox, M, A, G> {
    type Impler = DebuggingInbox;
}

impl<M, A, G> InboxImpl for SimpleSystem<DebuggingInbox, M, A, G> {
    type Impler = DebuggingInbox;
}

impl<M, A, G> InboxImpl for SimpleSystem<NothingInbox, M, A, G> {
    type Impler = NothingInbox;
}

impl<M, A, G> DebuggerImpl for SimpleSystem<HoldingDebugger, M, A, G> {
    type Impler = HoldingDebugger;
}

impl<M, A, G> DebuggerImpl for SimpleSystem<DebuggingInbox, M, A, G> {
    type Impler = DebuggingInbox;
}

impl<I, A, G> Memory16Impl for SimpleSystem<I, SegaMemoryMap, A, G>
where
    Self: Inbox,
{
    type Impler = SegaMemoryMap;
}

impl<I, A, G> Memory16Impl for SimpleSystem<I, CodemastersMemoryMap, A, G> {
    type Impler = CodemastersMemoryMap;
}

impl<I, M, A, G> Io16Impl for SimpleSystem<I, M, A, G> {
    type Impler = Sms2Io;
}

impl<I, M, A, G> SmsVdpInternalImpl for SimpleSystem<I, M, A, G> {
    type Impler = SimpleSmsVdpInternal;
}

impl<I, M, A, G> SmsVdpImpl for SimpleSystem<I, M, A, G>
where
    G: SimpleGraphics,
{
    type Impler = SimpleSmsVdp;
}

impl<I, M, A, G> Z80InternalImpl for SimpleSystem<I, M, A, G> {
    type Impler = Z80State;
}

impl<I, M, A, G> Z80Irq for SimpleSystem<I, M, A, G>
where
    Self: Memory16 + Io16,
{
    #[inline]
    fn requesting_mi(&self) -> Option<u8> {
        <Self as SmsVdpInternal>::requesting_mi(&self).or_else(|| self.io.requesting_mi())
    }

    #[inline]
    fn requesting_nmi(&self) -> bool {
        self.io.requesting_nmi()
    }

    #[inline]
    fn clear_nmi(&mut self) {
        self.io.clear_nmi();
    }
}

impl<I, M, A, G> Z80Impl for SimpleSystem<I, M, A, G>
where
    Self: Inbox + Z80Internal + Memory16 + Z80Irq,
{
    type Impler = Z80Interpreter<z80::Safe>;
}

impl<I, M, A, G> Sn76489InternalImpl for SimpleSystem<I, M, A, G> {
    type Impler = SimpleSn76489;
}

impl<I, M, A, G> Sn76489Impl for SimpleSystem<I, M, A, G>
where
    A: SimpleAudio,
{
    type Impler = SimpleSn76489;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Frequency {
    Ntsc,
    Pal,
    MasterFrequency(u64),
    Z80Frequency(u64),
    Unlimited,
}

impl Frequency {
    #[inline]
    pub fn to_z80_frequency(self) -> Option<u64> {
        match self {
            Frequency::Ntsc => Some(NTSC_MASTER_FREQUENCY / 3),
            Frequency::Pal => Some(PAL_MASTER_FREQUENCY / 3),
            Frequency::MasterFrequency(x) => Some(x / 3),
            Frequency::Z80Frequency(x) => Some(x),
            Frequency::Unlimited => None,
        }
    }
}

impl Default for Frequency {
    fn default() -> Self {
        Frequency::Ntsc
    }
}

pub const NTSC_MASTER_FREQUENCY: u64 = 10738580;

pub const PAL_MASTER_FREQUENCY: u64 = 10640685;

//// Types needed for emulation

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PlayerStatus {
    pub joypad_a: u8,
    pub joypad_b: u8,
    pub pause: bool,
}

impl Default for PlayerStatus {
    #[inline]
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
    pub hold: Option<Instant>,
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
            hold: None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EmulationResult {
    FrameCompleted,
    FrameInterrupted,
}
