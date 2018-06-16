use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Instant;

use failure::Error;

use host_multimedia::{SimpleAudio, SimpleAudioImpl, SimpleGraphics, SimpleGraphicsImpl};
use impler::{Cref, Impl, Mref, Ref};
use memo::{Inbox, InboxImpl};
use utilities;

use super::*;

pub const NTSC_Z80_FREQUENCY: u64 = 10738580 / 3;

pub const PAL_Z80_FREQUENCY: u64 = 10640685 / 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum MemoryMapperType {
    Sega,
    Codemasters,
    Sg1000(usize),
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SmsState {
    pub z80: Z80State,
    pub vdp: SmsVdpState,
    pub memory: SmsMemoryState,
    pub player_input: SmsPlayerInputState,
    pub memory_mapper_type: MemoryMapperType,
    pub pause_irq: SmsPauseInterruptState,
}

impl SmsState {
    pub fn from_rom(
        rom: Arc<Box<[[u8; 0x4000]]>>,
        memory_mapper_type: MemoryMapperType,
        tv_system: TvSystem,
        vdp_kind: Kind,
    ) -> SmsState {
        let mut state = SmsState {
            z80: Default::default(),
            vdp: Default::default(),
            player_input: Default::default(),
            pause_irq: Default::default(),
            memory_mapper_type,
            memory: SmsMemoryState {
                rom: rom,
                system_ram: Default::default(),
                main_cartridge_ram: Default::default(),
                half_cartridge_ram: Default::default(),
                pages: Default::default(),
            },
        };
        state.vdp.set_tv_system(tv_system);
        state.vdp.set_kind(vdp_kind);

        // it seems many BIOSes leave SP at this value
        sms.z80.set_reg16(Reg16::SP, 0xDFEE);

        match memory_mapper_type {
            MemoryMapperType::Sega => SegaMapper.default_mappings(&mut state.memory),
            MemoryMapperType::Codemasters => CodemastersMapper.default_mappings(&mut state.memory),
            MemoryMapperType::Sg1000(n) => Sg1000Mapper(n).default_mappings(&mut state.memory),
        }

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

macro_rules! implement_impl {
    ([$($impl_params: tt)*] $impl_name: ident for
     $type_name: ident [$($type_params: tt)*]
     [$($where_clause: tt)*]
     $impler_name: ty, $_self: ident, $f: ident,
     $make_body: expr, $make_mut_body: expr) => {
        impl<$($impl_params)*> Impl<$impl_name> for $type_name <$($type_params)*>
        where
            $($where_clause)*
        {
            type Impler = $impler_name;

            #[inline(always)]
            fn make<'a>(&'a $_self) -> Cref<'a, Self::Impler> {
                $make_body
            }

            #[inline(always)]
            fn make_mut<'a>(&'a mut $_self) -> Mref<'a, Self::Impler> {
                $make_mut_body
            }
        }
    }
}

pub struct SmsVdpGraphicsI<Graphics: ?Sized> {
    vdp: Ref<SmsVdpState>,
    graphics: Ref<Graphics>,
}

mod _impl_sms_vdp_graphics1 {
    use super::*;

    impl<Graphics: ?Sized> SmsVdpGraphicsI<Graphics> {
        #[inline(always)]
        pub fn new<'a>(vdp: &'a SmsVdpState, graphics: &'a Graphics) -> Cref<'a, Self> {
            Cref::Own(SmsVdpGraphicsI {
                vdp: unsafe { Ref::new(vdp) },
                graphics: unsafe { Ref::new(graphics) },
            })
        }

        #[inline(always)]
        pub fn new_mut<'a>(vdp: &'a mut SmsVdpState, graphics: &'a mut Graphics) -> Mref<'a, Self> {
            Mref::Own(SmsVdpGraphicsI {
                vdp: unsafe { Ref::new_mut(vdp) },
                graphics: unsafe { Ref::new_mut(graphics) },
            })
        }
    }

    implement_impl! {
        [Graphics] SmsVdpInternalImpl for SmsVdpGraphicsI [Graphics]
        [] SmsVdpState, self, f,
        { Cref::Const(self.vdp._0()) },
        { Mref::Mut(self.vdp.mut_0()) }
    }

    implement_impl! {
        [Graphics] SimpleGraphicsImpl for SmsVdpGraphicsI [Graphics]
        [Graphics: SimpleGraphics] Graphics, self, f,
        { Cref::Const(self.graphics._0()) },
        { Mref::Mut(self.graphics.mut_0()) }
    }

    implement_impl! {
        [Graphics] SmsVdpGraphicsImpl for SmsVdpGraphicsI [Graphics]
        [Graphics: SimpleGraphics] SimpleSmsVdpGraphicsImpler<Self>, self, f,
        { SimpleSmsVdpGraphicsImpler::new(self) },
        { SimpleSmsVdpGraphicsImpler::new_mut(self) }
    }
}

pub struct SmsAudioI<Sn76489: ?Sized, Audio: ?Sized> {
    sn76489: Ref<Sn76489>,
    audio: Ref<Audio>,
}

mod _impl_sms_audio_i {
    use super::*;

    impl<Sn76489: ?Sized, Audio: ?Sized> SmsAudioI<Sn76489, Audio> {
        #[inline(always)]
        pub fn new<'a>(sn76489: &'a Sn76489, audio: &'a Audio) -> Cref<'a, Self> {
            Cref::Own(SmsAudioI {
                sn76489: unsafe { Ref::new(sn76489) },
                audio: unsafe { Ref::new(audio) },
            })
        }

        #[inline(always)]
        pub fn new_mut<'a>(sn76489: &'a mut Sn76489, audio: &'a mut Audio) -> Mref<'a, Self> {
            Mref::Own(SmsAudioI {
                sn76489: unsafe { Ref::new_mut(sn76489) },
                audio: unsafe { Ref::new_mut(audio) },
            })
        }
    }

    implement_impl! {
        [Sn76489, Audio] Sn76489InterfaceImpl for SmsAudioI [Sn76489, Audio]
        [Sn76489: Sn76489Interface] Sn76489, self, f,
        { Cref::Const(self.sn76489._0()) },
        { Mref::Mut(self.sn76489.mut_0()) }
    }

    impl<Audio> AsRef<Sn76489State> for SmsAudioI<Sn76489State, Audio> {
        #[inline]
        fn as_ref(&self) -> &Sn76489State {
            self.sn76489._0()
        }
    }

    impl<Audio> AsMut<Sn76489State> for SmsAudioI<Sn76489State, Audio> {
        #[inline]
        fn as_mut(&mut self) -> &mut Sn76489State {
            self.sn76489.mut_0()
        }
    }

    implement_impl! {
        [Audio, Sn76489] SimpleAudioImpl for SmsAudioI [Sn76489, Audio]
        [Audio: SimpleAudio] Audio, self, f,
        { Cref::Const(self.audio._0()) },
        { Mref::Mut(self.audio.mut_0()) }
    }

    implement_impl! {
        [Audio] Sn76489AudioImpl for SmsAudioI [Sn76489State, Audio]
        [Audio: SimpleAudio] SimpleSn76489AudioImpler<Self>, self, f,
        { SimpleSn76489AudioImpler::new(self) },
        { SimpleSn76489AudioImpler::new_mut(self) }
    }

    implement_impl! {
        [Audio] Sn76489AudioImpl for SmsAudioI [FakeSn76489, Audio]
        [] FakeSn76489, self, f,
        { Cref::Const(self.sn76489._0()) },
        { Mref::Mut(self.sn76489.mut_0()) }
    }
}

pub struct SmsZ80RunI<Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized> {
    inbox: Ref<Inx>,
    memory: Ref<Mem>,
    mapper: Ref<Mapper>,
    z80: Ref<Z80State>,
    pause_irq: Ref<SmsPauseInterruptState>,
    vdp: Ref<SmsVdpState>,
    player_input: Ref<SmsPlayerInputState>,
    sn76489: Ref<Sn76489>,
}

mod _impl_sms_z80_run_i {
    use super::*;

    impl<Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized>
        SmsZ80RunI<Sn76489, Mapper, Mem, Inx>
    {
        #[inline(always)]
        pub fn new<'a>(
            mapper: &'a Mapper,
            memory: &'a Mem,
            inbox: &'a Inx,
            z80: &'a Z80State,
            pause_irq: &'a SmsPauseInterruptState,
            vdp: &'a SmsVdpState,
            player_input: &'a SmsPlayerInputState,
            sn76489: &'a Sn76489,
        ) -> Cref<'a, Self> {
            Cref::Own(SmsZ80RunI {
                mapper: unsafe { Ref::new(mapper) },
                memory: unsafe { Ref::new(memory) },
                inbox: unsafe { Ref::new(inbox) },
                z80: unsafe { Ref::new(z80) },
                pause_irq: unsafe { Ref::new(pause_irq) },
                vdp: unsafe { Ref::new(vdp) },
                player_input: unsafe { Ref::new(player_input) },
                sn76489: unsafe { Ref::new(sn76489) },
            })
        }

        #[inline(always)]
        pub fn new_mut<'a>(
            mapper: &'a mut Mapper,
            memory: &'a mut Mem,
            inbox: &'a mut Inx,
            z80: &'a mut Z80State,
            pause_irq: &'a mut SmsPauseInterruptState,
            vdp: &'a mut SmsVdpState,
            player_input: &'a mut SmsPlayerInputState,
            sn76489: &'a mut Sn76489,
        ) -> Mref<'a, Self> {
            Mref::Own(SmsZ80RunI {
                mapper: unsafe { Ref::new_mut(mapper) },
                memory: unsafe { Ref::new_mut(memory) },
                inbox: unsafe { Ref::new_mut(inbox) },
                z80: unsafe { Ref::new_mut(z80) },
                pause_irq: unsafe { Ref::new_mut(pause_irq) },
                vdp: unsafe { Ref::new_mut(vdp) },
                player_input: unsafe { Ref::new_mut(player_input) },
                sn76489: unsafe { Ref::new_mut(sn76489) },
            })
        }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx] SmsPauseInterruptImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        []
        SmsPauseInterruptState, self, f,
        { Cref::Const(self.pause_irq._0()) },
        { Mref::Mut(self.pause_irq.mut_0()) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx] InboxImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [Inx: Inbox<Memo=SmsMemo>] Inx, self, f,
        { Cref::Const(self.inbox._0()) },
        { Mref::Mut(self.inbox.mut_0()) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Z80InternalImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [] Z80State, self, f,
        { Cref::Const(self.z80._0()) },
        { Mref::Mut(self.z80.mut_0()) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Z80NoImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [] Z80State, self, f,
        { Cref::Const(self.z80._0()) },
        { Mref::Mut(self.z80.mut_0()) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Memory16Impl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [Mapper: SmsMapper<Mem>, Mem: SmsMemory + Memory16]
        SmsMapMemory16Impler<Mem, Mapper>, self, f,
        { SmsMapMemory16Impler::new(self.memory._0(), self.mapper._0()) },
        { SmsMapMemory16Impler::new_mut(self.memory.mut_0(), self.mapper.mut_0()) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Z80MemImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [Mapper: SmsMapper<Mem>, Mem: SmsMemory + Memory16]
        Z80MemImpler<Self>, self, f,
        { Z80MemImpler::new(self) },
        { Z80MemImpler::new_mut(self) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] SmsVdpInterfaceImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [] SmsVdpInterfaceImpler<SmsVdpState>, self, f,
        { SmsVdpInterfaceImpler::new(self.vdp._0()) },
        { SmsVdpInterfaceImpler::new_mut(self.vdp.mut_0()) }
    }

    implement_impl! {
        [Sn76489, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Sn76489InterfaceImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [Sn76489: Sn76489Interface] Sn76489, self, f,
        { Cref::Const(self.sn76489._0()) },
        { Mref::Mut(self.sn76489.mut_0()) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] SmsVdpInternalImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [] SmsVdpState, self, f,
        { Cref::Const(self.vdp._0()) },
        { Mref::Mut(self.vdp.mut_0()) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] SmsPlayerInputImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [] SmsPlayerInputState, self, f,
        { Cref::Const(self.player_input._0()) },
        { Mref::Mut(self.player_input.mut_0()) }
    }

    implement_impl! {
        [Sn76489, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Io16Impl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [Sn76489: Sn76489Interface] SmsIo16Impler<Self>, self, f,
        { SmsIo16Impler::new(self) },
        { SmsIo16Impler::new_mut(self) }
    }

    implement_impl! {
        [Sn76489, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Z80IoImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [Sn76489: Sn76489Interface] Z80IoImpler<Self>, self, f,
        { Z80IoImpler::new(self) },
        { Z80IoImpler::new_mut(self) }
    }

    implement_impl! {
        [Sn76489, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Z80IrqImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [] SmsZ80IrqImpler<Self>, self, f,
        { SmsZ80IrqImpler::new(self) },
        { SmsZ80IrqImpler::new_mut(self) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Z80InterruptImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [] Z80InterruptImpler<Self>, self, f,
        { Z80InterruptImpler::new(self) },
        { Z80InterruptImpler::new_mut(self) }
    }

    implement_impl! {
        [Sn76489: ?Sized, Mapper: ?Sized, Mem: ?Sized, Inx: ?Sized] Z80RunImpl for SmsZ80RunI
        [Sn76489, Mapper, Mem, Inx]
        [] Z80RunInterpreterImpler<Self>, self, f,
        { Z80RunInterpreterImpler::new(self) },
        { Z80RunInterpreterImpler::new_mut(self) }
    }
}

#[derive(Clone)]
struct SmsS<Graphics, Audio, Sn76489, Mapper, Mem, Inx> {
    z80: Z80State,
    vdp: SmsVdpState,
    memory: Mem,
    player_input: SmsPlayerInputState,

    // just need this so we can produce an `SmsState`
    memory_mapper_type: MemoryMapperType,
    pause_irq: SmsPauseInterruptState,
    graphics: Graphics,
    audio: Audio,
    sn76489: Sn76489,
    time_status: TimeStatus,
    inbox: Inx,
    mapper: Mapper,
}

pub trait Sms: Z80Internal + Memory16 + SmsMemory + SmsVdpInternal {
    fn run_frame(&mut self, player_input: SmsPlayerInputState) -> Result<(), SmsEmulationError>;

    fn state(&self) -> SmsState;

    fn hold(&mut self) -> Result<(), SmsEmulationError>;

    fn resume(&mut self) -> Result<(), SmsEmulationError>;
}

mod _impl_smst {
    use super::*;

    implement_impl! {
        [Graphics, Audio, Sn76489, Mapper, Mem, Inx] Z80InternalImpl for SmsS
        [Graphics, Audio, Sn76489, Mapper, Mem, Inx]
        [] Z80State, self, f,
        { Cref::Const(&self.z80) },
        { Mref::Mut(&mut self.z80) }
    }

    implement_impl! {
        [Graphics, Audio, Sn76489, Mapper, Mem, Inx] Memory16Impl for SmsS
        [Graphics, Audio, Sn76489, Mapper, Mem, Inx]
        [Mapper: SmsMapper<Mem>, Mem: SmsMemory + Memory16]
        SmsMapMemory16Impler<Mem, Mapper>, self, f,
        { SmsMapMemory16Impler::new(&self.memory, &self.mapper) },
        { SmsMapMemory16Impler::new_mut(&mut self.memory, &mut self.mapper) }
    }

    implement_impl! {
        [Graphics, Audio, Sn76489, Mapper, Mem, Inx] SmsMemoryImpl for SmsS
        [Graphics, Audio, Sn76489, Mapper, Mem, Inx]
        [Mem: SmsMemory]
        Mem, self, f,
        { Cref::Const(&self.memory) },
        { Mref::Mut(&mut self.memory) }
    }

    implement_impl! {
        [Graphics, Audio, Sn76489, Mapper, Mem, Inx]
        SmsVdpInternalImpl for SmsS
        [Graphics, Audio, Sn76489, Mapper, Mem, Inx]
        [] SmsVdpState, self, f,
        { Cref::Const(& self.vdp) },
        { Mref::Mut(&mut self.vdp) }
    }
}

impl<Graphics, Audio, Sn76489, Mapper, Mem, Inx> Sms
    for SmsS<Graphics, Audio, Sn76489, Mapper, Mem, Inx>
where
    Mem: SmsMemory + Memory16,
    Mapper: SmsMapper<Mem>,
    SmsVdpGraphicsI<Graphics>: SmsVdpGraphics,
    SmsZ80RunI<Sn76489, Mapper, Mem, Inx>: Z80Run,
    SmsAudioI<Sn76489, Audio>: Sn76489Audio + SimpleAudio,
{
    fn run_frame(&mut self, player_input: SmsPlayerInputState) -> Result<(), SmsEmulationError> {
        self.player_input = player_input;
        run_frame(self)
    }

    fn state(&self) -> SmsState {
        SmsState {
            z80: self.z80.clone(),
            vdp: self.vdp.clone(),
            memory: self.memory.state(),
            player_input: self.player_input.clone(),
            memory_mapper_type: self.memory_mapper_type.clone(),
            pause_irq: self.pause_irq.clone(),
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
        let mut audio = SmsAudioI::new_mut(&mut self.sn76489, &mut self.audio);
        const AUDIO_BUFFER_SIZE: u16 = 0x800;
        if let Some(frequency) = self.time_status.frequency {
            audio
                .configure(frequency as u32 / 16, AUDIO_BUFFER_SIZE)
                .map_err(|s| SmsEmulationError::AudioError(s))?;
            audio.play().map_err(|s| SmsEmulationError::AudioError(s))?;
        } else {
            audio.pause().map_err(|s| SmsEmulationError::AudioError(s))?;
        };

        Ok(())
    }
}

pub struct MemWrap<M>(PhantomData<M>);

impl<M> Default for MemWrap<M> {
    #[inline(always)]
    fn default() -> Self {
        MemWrap(PhantomData)
    }
}

pub fn new_sms<Graphics: 'static, Audio: 'static, Sn76489: 'static, Memory: 'static, Inx: 'static>(
    frequency: Option<u64>,
    state: SmsState,
    graphics: Graphics,
    audio: Audio,
    sn76489: Sn76489,
    inbox: Inx,
    _memory_type: MemWrap<Memory>,
) -> Result<Box<Sms>, SmsCreationError>
where
    Memory: SmsMemory + Memory16 + SmsMemoryLoad,
    SmsVdpGraphicsI<Graphics>: SmsVdpGraphics,
    SmsZ80RunI<Sn76489, SegaMapper, Memory, Inx>: Z80Run,
    SmsZ80RunI<Sn76489, CodemastersMapper, Memory, Inx>: Z80Run,
    SmsZ80RunI<Sn76489, Sg1000Mapper, Memory, Inx>: Z80Run,
    SmsAudioI<Sn76489, Audio>: Sn76489Audio + SimpleAudio,
{
    let time_status = TimeStatus::new(state.z80.cycles(), frequency);

    macro_rules! make {
        ($mapper:expr) => {
            Ok(Box::new(SmsS {
                graphics,
                audio,
                sn76489,
                inbox,
                time_status,
                mapper: $mapper,
                player_input: state.player_input,
                memory_mapper_type: state.memory_mapper_type,
                pause_irq: state.pause_irq,
                vdp: state.vdp,
                memory: <Memory as SmsMemoryLoad>::load(state.memory)?,
                z80: state.z80,
            }))
        };
    }

    match state.memory_mapper_type {
        MemoryMapperType::Sega => make!(SegaMapper::default()),
        MemoryMapperType::Codemasters => make!(CodemastersMapper::default()),
        _ => make!(Sg1000Mapper::default()),
    }
}

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

impl From<SmsVdpGraphicsError> for SmsEmulationError {
    fn from(x: SmsVdpGraphicsError) -> Self {
        SmsEmulationError::GraphicsError(x)
    }
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

fn run_frame<Graphics, Audio, Sn76489, Mapper, Mem, Inx>(
    sms: &mut SmsS<Graphics, Audio, Sn76489, Mapper, Mem, Inx>,
) -> Result<(), SmsEmulationError>
where
    SmsVdpGraphicsI<Graphics>: SmsVdpGraphics,
    SmsZ80RunI<Sn76489, Mapper, Mem, Inx>: Z80Run,
    SmsAudioI<Sn76489, Audio>: Sn76489Audio,
{
    use std::ops::DerefMut;

    loop {
        sms_vdp::line(SmsVdpGraphicsI::new_mut(&mut sms.vdp, &mut sms.graphics).deref_mut())?;

        let vdp_cycles = sms.vdp.cycles();
        let z80_target_cycles = 2 * vdp_cycles / 3;

        while sms.z80.cycles() < z80_target_cycles {
            SmsZ80RunI::new_mut(
                &mut sms.mapper,
                &mut sms.memory,
                &mut sms.inbox,
                &mut sms.z80,
                &mut sms.pause_irq,
                &mut sms.vdp,
                &mut sms.player_input,
                &mut sms.sn76489,
            ).run(z80_target_cycles);
        }

        if sms.vdp.v() == 0 {
            // we've just finished a frame

            let time_status = sms.time_status;

            if let Some(f) = time_status.frequency {
                // Sound
                let sound_target_cycles = sms.z80.cycles() / 16;
                SmsAudioI::new_mut(&mut sms.sn76489, &mut sms.audio)
                    .queue(sound_target_cycles)
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
