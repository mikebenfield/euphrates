use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use failure::Error;

use host_multimedia::SimpleAudio;
use memo::Inbox;
use utilities;

use super::*;

pub const NTSC_Z80_FREQUENCY: u64 = 10738580 / 3;

pub const PAL_Z80_FREQUENCY: u64 = 10640685 / 3;

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SmsState {
    pub z80: Z80State,
    pub vdp: SmsVdpState,
    pub memory: SmsMemoryState,
    pub player_input: SmsPlayerInput,
    pub pause_irq: SmsPauseInterruptState,
    pub sn76489: Sn76489State,
}

impl SmsState {
    pub fn from_rom(
        rom: Arc<Box<[u8]>>,
        mapper: SmsMemoryMapper,
        tv_system: TvSystem,
        vdp_kind: Kind,
    ) -> SmsState {
        let mut state = SmsState {
            z80: Default::default(),
            vdp: Default::default(),
            player_input: Default::default(),
            pause_irq: Default::default(),
            memory: SmsMemoryState {
                rom: rom,
                system_ram: Default::default(),
                main_cartridge_ram: Default::default(),
                half_cartridge_ram: Default::default(),
                pages: Default::default(),
                mapper,
            },
            sn76489: Default::default(),
        };
        state.vdp.set_tv_system(tv_system);
        state.vdp.set_kind(vdp_kind);

        // it seems many BIOSes leave SP at this value
        state.z80.set_reg16(Reg16::SP, 0xDFEE);

        sms_memory::default_mappings(&mut state.memory);

        state
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TimeStatus {
    /// Any time before now
    pub start_time: Instant,

    /// How many cycles on the clock at `start_time`?
    pub start_cycles: u64,

    /// Clock frequency in Hz
    pub frequency: Option<u64>,

    pub holding: bool,
}

impl TimeStatus {
    pub fn new(start_cycles: u64, frequency: Option<u64>) -> Self {
        TimeStatus {
            start_cycles,
            start_time: Instant::now(),
            frequency,
            holding: false,
        }
    }
}

#[derive(Clone)]
struct SmsS<Graphics, Audio, Sn76489, Mem, Inx> {
    z80: Z80State,
    vdp: SmsVdpState,
    memory: Mem,
    player_input: SmsPlayerInput,
    pause_irq: SmsPauseInterruptState,
    graphics: Graphics,
    audio: Audio,
    sn76489: Sn76489,
    time_status: TimeStatus,
    inbox: Inx,
}

pub trait Sms {
    fn z80(&self) -> &dyn Z80Internal;

    fn z80_mut(&mut self) -> &mut dyn Z80Internal;

    fn debugger(&mut self) -> Option<&mut dyn Debugger>;

    fn run_frame(&mut self, player_input: SmsPlayerInput) -> Result<(), SmsEmulationError>;

    fn state(&self) -> SmsState;

    fn hold(&mut self) -> Result<(), SmsEmulationError>;

    fn resume(&mut self) -> Result<(), SmsEmulationError>;
}

impl<Graphics, Audio, Sn76489, Mem, Inx> Sms for SmsS<Graphics, Audio, Sn76489, Mem, Inx>
where
    for<'a> SmsVdpGraphicsImpler<'a, SmsVdpState, Graphics>: SmsVdpLineImpler,
    Audio: SimpleAudio,
    Sn76489: Sn76489Interface + HasSn76489State,
    for<'a> Sn76489Impler<'a, Sn76489, Audio>: Sn76489Audio,
    Inx: Inbox<Memo = Z80Memo> + GetDebugger,
    Mem: Memory16 + SmsMemory,
{
    fn z80(&self) -> &dyn Z80Internal {
        &self.z80
    }

    fn z80_mut(&mut self) -> &mut dyn Z80Internal {
        &mut self.z80
    }

    fn debugger(&mut self) -> Option<&mut dyn Debugger> {
        self.inbox.debugger()
    }

    fn run_frame(&mut self, player_input: SmsPlayerInput) -> Result<(), SmsEmulationError> {
        self.player_input = player_input;
        run_frame(self)
    }

    fn state(&self) -> SmsState {
        SmsState {
            z80: self.z80.clone(),
            vdp: self.vdp.clone(),
            memory: self.memory.state(),
            player_input: self.player_input.clone(),
            pause_irq: self.pause_irq.clone(),
            sn76489: self.sn76489.state(),
        }
    }

    fn hold(&mut self) -> Result<(), SmsEmulationError> {
        self.time_status.holding = true;
        Ok(())
    }

    fn resume(&mut self) -> Result<(), SmsEmulationError> {
        self.time_status.start_time = Instant::now();
        self.time_status.start_cycles = self.z80.cycles();
        self.time_status.holding = false;

        // audio
        const AUDIO_BUFFER_SIZE: u16 = 0x800;
        if let Some(frequency) = self.time_status.frequency {
            self.audio
                .configure(frequency as u32 / 16, AUDIO_BUFFER_SIZE)
                .map_err(|s| SmsEmulationError::AudioError(s))?;
            self.audio
                .play()
                .map_err(|s| SmsEmulationError::AudioError(s))?;
        } else {
            self.audio
                .pause()
                .map_err(|s| SmsEmulationError::AudioError(s))?;
        };

        Ok(())
    }
}

#[derive(Debug)]
pub struct TypeWrap<M>(PhantomData<M>);

impl<M> Default for TypeWrap<M> {
    fn default() -> Self {
        TypeWrap(PhantomData)
    }
}

pub fn new_sms<Graphics: 'static, Audio: 'static, Sn76489: 'static, Memory: 'static, Inx: 'static>(
    frequency: Option<u64>,
    state: SmsState,
    graphics: Graphics,
    audio: Audio,
    inbox: Inx,
    _mem: TypeWrap<Memory>,
    _sn76489: TypeWrap<Sn76489>,
) -> Result<Box<dyn Sms>, SmsCreationError>
where
    for<'a> SmsVdpGraphicsImpler<'a, SmsVdpState, Graphics>: SmsVdpLineImpler,
    Audio: SimpleAudio,
    Sn76489: Sn76489Interface + HasSn76489State,
    for<'a> Sn76489Impler<'a, Sn76489, Audio>: Sn76489Audio,
    Inx: Inbox<Memo = Z80Memo> + GetDebugger,
    Memory: SmsMemory + SmsMemoryLoad,
{
    let time_status = TimeStatus::new(state.z80.cycles(), frequency);

    Ok(Box::new(SmsS {
        graphics,
        audio,
        inbox,
        time_status,
        player_input: state.player_input,
        pause_irq: state.pause_irq,
        vdp: state.vdp,
        memory: <Memory as SmsMemoryLoad>::load(state.memory)?,
        z80: state.z80,
        sn76489: Sn76489::load(state.sn76489),
    }))
}

// This superfluous module with the `allow` attribute is necessary until the
// `fail` crate begins using `dyn trait` syntax
#[allow(bare_trait_objects)]
mod sms_emulation_error {
    use super::*;

    #[derive(Debug, Fail)]
    pub enum SmsEmulationError {
        // XXX
        #[fail(display = "SomeError")]
        SomeError,
        #[fail(display = "Audio Error {}", _0)]
        AudioError(Error),
        #[fail(display = "Graphics Error {}", _0)]
        GraphicsError(#[cause] SmsVdpGraphicsError),
    }
}

pub use self::sms_emulation_error::SmsEmulationError;

impl From<SmsVdpGraphicsError> for SmsEmulationError {
    fn from(x: SmsVdpGraphicsError) -> Self {
        SmsEmulationError::GraphicsError(x)
    }
}

// This superfluous module with the `allow` attribute is necessary until the
// `fail` crate begins using `dyn trait` syntax
#[allow(bare_trait_objects)]
mod sms_creation_error {
    use super::*;

    #[derive(Debug, Fail)]
    pub enum SmsCreationError {
        #[fail(display = "ROM error: {}", _0)]
        RomError(#[cause] SmsRomError),

        #[fail(display = "memory load error: {}", _0)]
        MemoryLoadError(#[cause] SmsMemoryLoadError),
    }
}

pub use self::sms_creation_error::SmsCreationError;

impl From<SmsRomError> for SmsCreationError {
    fn from(x: SmsRomError) -> Self {
        SmsCreationError::RomError(x)
    }
}

impl From<SmsMemoryLoadError> for SmsCreationError {
    fn from(x: SmsMemoryLoadError) -> Self {
        SmsCreationError::MemoryLoadError(x)
    }
}

fn run_frame<Graphics, Audio, Sn76489, Mem, Inx>(
    sms: &mut SmsS<Graphics, Audio, Sn76489, Mem, Inx>,
) -> Result<(), SmsEmulationError>
where
    for<'a> SmsVdpGraphicsImpler<'a, SmsVdpState, Graphics>: SmsVdpLineImpler,
    Audio: SimpleAudio,
    Sn76489: Sn76489Interface,
    for<'a> Sn76489Impler<'a, Sn76489, Audio>: Sn76489Audio,
    Inx: Inbox<Memo = Z80Memo>,
    Mem: Memory16 + SmsMemory,
{
    sms.pause_irq.pause_pressed(sms.player_input.pause());

    loop {
        sms_vdp::line(&mut SmsVdpGraphicsImpler {
            graphics: &mut sms.graphics,
            vdp: &mut sms.vdp,
        })?;
        let vdp_cycles = sms.vdp.cycles();
        let z80_target_cycles = 2 * vdp_cycles / 3;
        while sms.z80.cycles() < z80_target_cycles {
            // use a trait object for this to cut down on code bloat
            let sn76489: &mut dyn Sn76489Interface = &mut sms.sn76489;
            let rc_vdp = Rc::new(RefCell::new(&mut sms.vdp));
            let irq = &mut SmsZ80IrqImpler {
                pause_interrupt: &mut sms.pause_irq,
                vdp: rc_vdp.clone(),
            };
            let io = &mut SmsIo16Impler {
                vdp: rc_vdp,
                player_input: sms.player_input,
                sn76489,
            };
            Z80RunImpler {
                z80: &mut sms.z80,
                memory: &mut sms.memory,
                inbox: &mut sms.inbox,
                irq,
                io,
            }.run(z80_target_cycles);
        }
        if sms.vdp.v() == 0 {
            // we've just finished a frame

            let time_status = sms.time_status;

            if let Some(f) = time_status.frequency {
                // Sound
                let sound_target_cycles = sms.z80.cycles() / 16;
                Sn76489Impler {
                    sn76489: &mut sms.sn76489,
                    audio: &mut sms.audio,
                }.queue(sound_target_cycles)
                    .map_err(|s| SmsEmulationError::AudioError(s))?;

                // sleep to sync time
                utilities::time_govern2(
                    time_status.start_time,
                    time_status.start_cycles,
                    z80_target_cycles,
                    f,
                );
            }

            return Ok(());
        }
    }
}
