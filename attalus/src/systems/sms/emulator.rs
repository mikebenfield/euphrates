use std::marker::PhantomData;
use std::path::Path;
use std::time::Instant;

use failure::Error;

use host_multimedia::{SimpleAudio, SimpleAudioImpl, SimpleGraphics, SimpleGraphicsImpl};
use impler::Impler;
use memo::{Inbox, InboxImpl, NothingInbox};
use utilities;

use super::*;

pub const NTSC_Z80_FREQUENCY: u64 = 10738580 / 3;

pub const PAL_Z80_FREQUENCY: u64 = 10640685 / 3;

pub trait MasterSystem: Z80Internal + SmsVdpInternal + Debugger {
    fn run_frame(&mut self, player_input: SmsPlayerInputState) -> Result<(), SmsEmulationError>;

    fn state(&self) -> SmsState;

    fn hold(&mut self) -> Result<(), SmsEmulationError>;

    fn resume(&mut self) -> Result<(), SmsEmulationError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum MemoryMapperType {
    Sega,
    Codemasters,
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SmsState {
    pub z80: Z80State,
    pub vdp: SmsVdpState,
    pub mem: SmsMemoryState,
    pub player_input: SmsPlayerInputState,
    pub memory_mapper_type: MemoryMapperType,
    pub irq_state: bool,
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
pub struct Sms<Sn76489, Sg, Sa, Mapper, Mem, Inx> {
    z80: Z80State,
    vdp: SmsVdpState,
    mem: Mem,
    player_input: SmsPlayerInputState,

    // just need this so we can produce an `SmsState`
    memory_mapper_type: MemoryMapperType,
    irq_state: bool,
    graphics: Sg,
    audio: Sa,
    sn76489: Sn76489,
    time_status: TimeStatus,
    inbox: Inx,
    _mapper: PhantomData<Mapper>,
}

#[derive(Debug, Fail)]
pub enum SmsEmulationError {
    // XXX
    #[fail(display = "SomeError")]
    SomeError,
    #[fail(display = "Audio Error {}", _0)]
    AudioError(Error),
}

#[derive(Debug, Fail)]
pub enum SmsCreationError {
    #[fail(display = "ROM error: {}", _0)]
    RomError(#[cause] SmsRomError),

    #[fail(display = "memory load error: {}", _0)]
    MemoryLoadError(#[cause] SmsMemoryLoadError),
}

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

pub struct MasterSystemCreate<Sn76489, Mem> {
    _p: PhantomData<Sn76489>,
    _m: PhantomData<Mem>,
}

impl<Sn76489, Mem> MasterSystemCreate<Sn76489, Mem>
where
    Sn76489: Default + 'static,
    Mem: SmsMemory + SmsMemoryLoad + 'static,
{
    fn from_state_help<Sg, Sa>(
        state: SmsState,
        sg: Sg,
        sa: Sa,
        debug: bool,
        frequency: Option<u64>,
        default_mappings: bool,
    ) -> Result<Box<MasterSystem>, SmsCreationError>
    where
        Sg: 'static,
        Sa: 'static,
        Sms<Sn76489, Sg, Sa, SegaMapper, Mem, NothingInbox<SmsMemo>>: MasterSystem,
        Sms<Sn76489, Sg, Sa, SegaMapper, Mem, DebuggingInbox>: MasterSystem,
        Sms<Sn76489, Sg, Sa, CodemastersMapper, Mem, NothingInbox<SmsMemo>>: MasterSystem,
        Sms<Sn76489, Sg, Sa, CodemastersMapper, Mem, DebuggingInbox>: MasterSystem,
    {
        let mut mem = SmsMemoryLoad::load(state.mem)?;
        let time_status = TimeStatus::new(state.z80.cycles(), frequency);

        macro_rules! ret {
            ($mapper:ident, $inbox:ty) => {{
                if default_mappings {
                    $mapper::default_mappings(&mut mem);
                }
                let mut sms: Sms<Sn76489, Sg, Sa, $mapper, Mem, $inbox> = Sms {
                    z80: state.z80,
                    vdp: state.vdp,
                    mem,
                    player_input: state.player_input,
                    memory_mapper_type: state.memory_mapper_type,
                    irq_state: state.irq_state,
                    time_status,
                    inbox: Default::default(),
                    graphics: sg,
                    audio: sa,
                    sn76489: Default::default(),
                    _mapper: PhantomData,
                };

                // It seems most BIOSes leave SP as 0xDFEE
                if default_mappings {
                    sms.z80.set_reg16(Reg16::SP, 0xDF00);
                }
                return Ok(Box::new(sms));
            }};
        }
        match (state.memory_mapper_type, debug) {
            (MemoryMapperType::Sega, false) => ret!(SegaMapper, NothingInbox<SmsMemo>),
            (MemoryMapperType::Sega, true) => ret!(SegaMapper, DebuggingInbox),
            (MemoryMapperType::Codemasters, false) => {
                ret!(CodemastersMapper, NothingInbox<SmsMemo>)
            }
            (MemoryMapperType::Codemasters, true) => ret!(CodemastersMapper, DebuggingInbox),
        }
    }

    pub fn from_state<Sg, Sa>(
        state: SmsState,
        sg: Sg,
        sa: Sa,
        debug: bool,
        frequency: Option<u64>,
    ) -> Result<Box<MasterSystem>, SmsCreationError>
    where
        Sg: 'static,
        Sa: 'static,
        Sms<Sn76489, Sg, Sa, SegaMapper, Mem, NothingInbox<SmsMemo>>: MasterSystem,
        Sms<Sn76489, Sg, Sa, SegaMapper, Mem, DebuggingInbox>: MasterSystem,
        Sms<Sn76489, Sg, Sa, CodemastersMapper, Mem, NothingInbox<SmsMemo>>: MasterSystem,
        Sms<Sn76489, Sg, Sa, CodemastersMapper, Mem, DebuggingInbox>: MasterSystem,
    {
        Self::from_state_help(state, sg, sa, debug, frequency, false)
    }

    pub fn from_file<P, Sg, Sa>(
        path: P,
        sg: Sg,
        sa: Sa,
        memory_mapper_type: MemoryMapperType,
        debug: bool,
        frequency: Option<u64>,
    ) -> Result<Box<MasterSystem>, SmsCreationError>
    where
        P: AsRef<Path>,
        Sg: 'static,
        Sa: 'static,
        Sms<Sn76489, Sg, Sa, SegaMapper, Mem, NothingInbox<SmsMemo>>: MasterSystem,
        Sms<Sn76489, Sg, Sa, SegaMapper, Mem, DebuggingInbox>: MasterSystem,
        Sms<Sn76489, Sg, Sa, CodemastersMapper, Mem, NothingInbox<SmsMemo>>: MasterSystem,
        Sms<Sn76489, Sg, Sa, CodemastersMapper, Mem, DebuggingInbox>: MasterSystem,
    {
        use hardware::sms_roms::from_file;
        Self::from_rom(
            from_file(path)?,
            sg,
            sa,
            memory_mapper_type,
            debug,
            frequency,
        )
    }

    pub fn from_rom<Sg, Sa>(
        rom: Box<[[u8; 0x4000]]>,
        sg: Sg,
        sa: Sa,
        memory_mapper_type: MemoryMapperType,
        debug: bool,
        frequency: Option<u64>,
    ) -> Result<Box<MasterSystem>, SmsCreationError>
    where
        Sg: 'static,
        Sa: 'static,
        Sms<Sn76489, Sg, Sa, SegaMapper, Mem, NothingInbox<SmsMemo>>: MasterSystem,
        Sms<Sn76489, Sg, Sa, SegaMapper, Mem, DebuggingInbox>: MasterSystem,
        Sms<Sn76489, Sg, Sa, CodemastersMapper, Mem, NothingInbox<SmsMemo>>: MasterSystem,
        Sms<Sn76489, Sg, Sa, CodemastersMapper, Mem, DebuggingInbox>: MasterSystem,
    {
        let state = SmsState {
            memory_mapper_type,
            z80: Default::default(),
            vdp: Default::default(),
            mem: SmsMemoryLoad::from_rom(rom)?,
            player_input: Default::default(),
            irq_state: false,
        };

        Self::from_state_help(state, sg, sa, debug, frequency, true)
    }
}

impl<Sn76489, Sg, Sa, Mapper, Mem, Inx> MasterSystem for Sms<Sn76489, Sg, Sa, Mapper, Mem, Inx>
where
    Mem: SmsMemory,
    Self: Sn76489Audio
        + SimpleAudio
        + Debugger
        + Z80Internal
        + Z80Emulator
        + SmsVdpInternal
        + SmsVdpGraphics
        + AsRef<TimeStatus>,
{
    fn run_frame(&mut self, player_input: SmsPlayerInputState) -> Result<(), SmsEmulationError> {
        self.player_input = player_input;
        run_frame(self)
    }

    fn state(&self) -> SmsState {
        SmsState {
            z80: self.z80.clone(),
            vdp: self.vdp.clone(),
            mem: self.mem.state(),
            player_input: self.player_input.clone(),
            memory_mapper_type: self.memory_mapper_type,
            irq_state: self.irq_state.clone(),
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
            self.configure(frequency as u32 / 16, AUDIO_BUFFER_SIZE)
                .map_err(|s| SmsEmulationError::AudioError(s))?;
        };
        self.play().map_err(|s| SmsEmulationError::AudioError(s))?;

        Ok(())
    }
}

impl<Sn76489, Sg, Sa, Mapper, Mem, Inx> AsRef<TimeStatus>
    for Sms<Sn76489, Sg, Sa, Mapper, Mem, Inx>
{
    fn as_ref(&self) -> &TimeStatus {
        &self.time_status
    }
}

impl<Sn76489, Sg, Sa, Mapper, Mem, Inx> InboxImpl for Sms<Sn76489, Sg, Sa, Mapper, Mem, Inx>
where
    Inx: Inbox<Memo = SmsMemo>,
{
    type Impler = Inx;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T,
    {
        f(&self.inbox)
    }

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T,
    {
        f(&mut self.inbox)
    }
}

macro_rules! implement_impl {
    ([$($impl_params: tt)*] $impl_name: ident for
     $type_name: ident [$($type_params: tt)*]
     [$($where_clause: tt)*]
     $impler_name: ty, $_self: ident, $f: ident,
     $close_body: expr, $close_mut_body: expr) => {
        impl<$($impl_params)*> $impl_name for $type_name <$($type_params)*>
        where
            $($where_clause)*
        {
            type Impler = $impler_name;
            fn close<F, T>(&$_self, $f: F) -> T
            where
                F: FnOnce(&Self::Impler) -> T,
            {
                $close_body
            }
            fn close_mut<F, T>(&mut $_self, $f: F) -> T
            where
                F: FnOnce(&mut Self::Impler) -> T,
            {
                $close_mut_body
            }
        }
    }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] DebuggerImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Inx: Debugger] Inx, self, f,
    { f(&self.inbox) },
    { f(&mut self.inbox) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] SmsVdpInternalImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
    [] SmsVdpState, self, f,
    { f(&self.vdp) },
    { f(&mut self.vdp) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] SmsVdpInterfaceImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
    [] SmsVdpState, self, f,
    { f(&self.vdp) },
    { f(&mut self.vdp) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] SimpleGraphicsImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
    [Sg: SimpleGraphics] Sg, self, f,
    { f(&self.graphics) },
    { f(&mut self.graphics) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] SmsVdpGraphicsImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
    [Sg: SimpleGraphics] SimpleSmsVdpGraphicsImpler<Self>, self, f,
    { SimpleSmsVdpGraphicsImpler::iclose(self, |z| f(z)) },
    { SimpleSmsVdpGraphicsImpler::iclose_mut(self, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Memory16Impl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Mapper: SmsMapper<Mem>,
         Mem: SmsMemory + Memory16,
        ]
        SmsMapMemory16Impler<Mem, Mapper>, self, f,
    { SmsMapMemory16Impler::iclose(&self.mem, |z| f(z)) },
    { SmsMapMemory16Impler::iclose_mut(&mut self.mem, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] SmsPlayerInputImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        []
        SmsPlayerInputState, self, f,
    { f(&self.player_input) },
    { f(&mut self.player_input) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] SmsVdpIrqImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        []
        SmsVdpState, self, f,
    { f(&self.vdp) },
    { f(&mut self.vdp) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Z80InternalImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        []
        Z80State, self, f,
    { f(&self.z80) },
    { f(&mut self.z80) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Z80NoImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        []
        Z80NoImpler<Self>, self, f,
    { Z80NoImpler::iclose(self, |z| f(z)) },
    { Z80NoImpler::iclose_mut(self, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Z80MemImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Mapper: SmsMapper<Mem>,
         Mem: Memory16 + SmsMemory,
        ]
        Z80MemImpler<Self>, self, f,
    { Z80MemImpler::iclose(self, |z| f(z)) },
    { Z80MemImpler::iclose_mut(self, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Sn76489InterfaceImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Sn76489: Sn76489Interface]
        Sn76489, self, f,
    { f(&self.sn76489) },
    { f(&mut self.sn76489) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] SimpleAudioImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Sa: SimpleAudio]
        Sa, self, f,
    { f(&self.audio) },
    { f(&mut self.audio) }
}

impl<Sg, Sa, Mapper, Mem, Inx> AsRef<Sn76489State> for Sms<Sn76489State, Sg, Sa, Mapper, Mem, Inx> {
    fn as_ref(&self) -> &Sn76489State {
        &self.sn76489
    }
}

impl<Sg, Sa, Mapper, Mem, Inx> AsMut<Sn76489State> for Sms<Sn76489State, Sg, Sa, Mapper, Mem, Inx> {
    fn as_mut(&mut self) -> &mut Sn76489State {
        &mut self.sn76489
    }
}

implement_impl! {
    [Sg, Sa, Mapper, Mem, Inx] Sn76489AudioImpl for Sms[Sn76489State, Sg, Sa, Mapper, Mem, Inx]
        [Self: Sn76489Interface + SimpleAudio]
        SimpleSn76489AudioImpler<Self>, self, f,
    { SimpleSn76489AudioImpler::iclose(self, |z| f(z)) },
    { SimpleSn76489AudioImpler::iclose_mut(self, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Io16Impl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Self: Sn76489Interface]
        SmsIo16Impler<Self>, self, f,
    { SmsIo16Impler::iclose(self, |z| f(z)) },
    { SmsIo16Impler::iclose_mut(self, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Z80IoImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Self: Z80Internal + Io16 + Memory16]
        Z80IoImpler<Self>, self, f,
    { Z80IoImpler::iclose(self, |z| f(z)) },
    { Z80IoImpler::iclose_mut(self, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] SmsZ80IrqStateImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        []
        bool, self, f,
    { f(&self.irq_state) },
    { f(&mut self.irq_state) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Z80RunImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Self: Z80Internal + Z80No + Z80Mem + Z80Io + Memory16 + Inbox<Memo=SmsMemo>]
        Z80RunInterpreterImpler<Self>, self, f,
    { Z80RunInterpreterImpler::iclose(self, |z| f(z)) },
    { Z80RunInterpreterImpler::iclose_mut(self, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Z80IrqImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Self: SmsVdpIrq + SmsPlayerInput + SmsZ80IrqState]
        SmsZ80IrqImpler<Self>, self, f,
    { SmsZ80IrqImpler::iclose(self, |z| f(z)) },
    { SmsZ80IrqImpler::iclose_mut(self, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Z80InterruptImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Self: Z80Mem + Z80Irq + Z80Internal]
        Z80InterruptImpler<Self>, self, f,
    { Z80InterruptImpler::iclose(self, |z| f(z)) },
    { Z80InterruptImpler::iclose_mut(self, |z| f(z)) }
}

implement_impl! {
    [Sn76489, Sg, Sa, Mapper, Mem, Inx] Z80EmulatorImpl for Sms[Sn76489, Sg, Sa, Mapper, Mem, Inx]
        [Self: Z80Internal + Z80Run + Z80Interrupt]
        Z80EmulatorImpler<Self>, self, f,
    { Z80EmulatorImpler::iclose(self, |z| f(z)) },
    { Z80EmulatorImpler::iclose_mut(self, |z| f(z)) }
}

pub fn run_frame<Sms>(sms: &mut Sms) -> Result<(), SmsEmulationError>
where
    Sms: Sn76489Audio
        + Z80Internal
        + Z80Emulator
        + SmsVdpInternal
        + SmsVdpGraphics
        + AsRef<TimeStatus>,
{
    loop {
        sms_vdp::line(sms).expect("Fix this! XXX");

        let vdp_cycles = SmsVdpInternal::cycles(sms);
        let z80_target_cycles = 2 * vdp_cycles / 3;

        while Z80Internal::cycles(sms) < z80_target_cycles {
            sms.emulate(z80_target_cycles);
            // XXX holding
        }

        if sms.v() == 0 {
            // we've just finished a frame

            let time_status = *AsRef::<TimeStatus>::as_ref(sms);

            if let Some(f) = time_status.frequency {
                // Sound
                let sound_target_cycles = Z80Internal::cycles(sms) / 16;
                sms.queue(sound_target_cycles)
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
