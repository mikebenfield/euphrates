use memo::Inbox;

use hardware::memory16::Memory16;

use super::*;

use self::ConditionCode::*;
use self::Reg16::*;
use self::Reg8::*;

macro_rules! process_argument {
    ($nn:tt, $n:tt, $d:tt, $e:tt,n) => {
        $n
    };
    ($nn:tt, $n:tt, $d:tt, $e:tt,nn) => {
        $nn
    };
    ($nn:tt, $n:tt, $d:tt, $e:tt,d) => {
        $d
    };
    ($nn:tt, $n:tt, $d:tt, $e:tt,e) => {
        $e
    };
    ($nn:tt, $n:tt, $d:tt, $e:tt,(nn)) => {
        Address($nn)
    };
    ($nn:tt, $n:tt, $d:tt, $e:tt,($reg:ident + d)) => {
        Shift($reg, $d)
    };
    ($nn:tt, $n:tt, $d:tt, $e:tt,($address:expr)) => {
        Address($address)
    };
    ($nn:tt, $n:tt, $d:tt, $e:tt, $arg:expr) => {
        $arg
    };
}

macro_rules! process_arguments {
    ($self_: ident,
     $mnemonic: ident,
     $nn: tt,
     $n: tt,
     $d: tt,
     $e: tt,
     ($($args: tt),*)
    ) => {
        $self_.
        $mnemonic
        (
            $(
                process_argument!($nn, $n, $d, $e, $args)
            ),*
        )
    }
}

#[inline]
fn send_instruction_memo<Z>(z: &mut Z, instruction_start: u16, instruction_after: u16)
where
    Z: Memory16 + Inbox + ?Sized,
    Z::Memo: From<Z80Memo>,
{
    if !z.active() {
        return;
    }

    let opcode = match instruction_after.wrapping_sub(instruction_start) {
        1 => Opcode::OneByte([z.read(instruction_start)]),
        2 => Opcode::TwoBytes([
            z.read(instruction_start),
            z.read(instruction_start.wrapping_add(1)),
        ]),
        3 => Opcode::ThreeBytes([
            z.read(instruction_start),
            z.read(instruction_start.wrapping_add(1)),
            z.read(instruction_start.wrapping_add(2)),
        ]),
        4 => Opcode::FourBytes([
            z.read(instruction_start),
            z.read(instruction_start.wrapping_add(1)),
            z.read(instruction_start.wrapping_add(2)),
            z.read(instruction_start.wrapping_add(3)),
        ]),
        _ => panic!("opcode more than 4 bytes?"),
    };

    // make sure this is a real instruction and not just a prefix
    match opcode {
        Opcode::OneByte([0xFD]) => {}
        Opcode::OneByte([0xDD]) => {}
        Opcode::OneByte([0xED]) => {}
        Opcode::OneByte([0xCB]) => {}
        Opcode::TwoBytes([0xFD, 0xCB]) => {}
        Opcode::TwoBytes([0xDD, 0xCB]) => {}
        _ => z.receive(From::from(Z80Memo::Instruction {
            pc: instruction_start,
            opcode,
        })),
    }
}

macro_rules! process_instruction {
    // instructions jrcc, jpcc, callcc, retcc, lddr, ldir, indr, inir otdr,
    // otir, cpdr, and cpir take a variable number of cycles depending on
    // whether their condition is met
    (@block
        [
            $opcode:expr ;
            $opcode_name:ident ;
            ;
            $mnemonic: ident () ;
            xx ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc,
            );

            self.$mnemonic();
            if self.reg16(BC) == 0 {
                self.inc_cycles(16);
            } else {
                self.inc_cycles(21);
            }
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            e ;
            jrcc ($cc: ident, e) ;
            xx ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);
            let e = self.read(pc) as i8;
            self.set_reg16(PC, pc.wrapping_add(1));

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc.wrapping_add(1),
            );

            self.jrcc($cc, e);
            if $cc.view(self) {
                self.inc_cycles(12);
            } else {
                self.inc_cycles(7);
            }
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            nn ;
            callcc ($cc: ident, nn) ;
            xx ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);
            let nn = Viewable::<u16>::view(Address(pc), self);
            self.set_reg16(PC, pc.wrapping_add(2));

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc.wrapping_add(2),
            );

            self.callcc($cc, nn);
            if $cc.view(self) {
                self.inc_cycles(17);
            } else {
                self.inc_cycles(10);
            }
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            ;
            retcc ($cc: ident) ;
            xx ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc,
            );

            self.retcc($cc);
            if $cc.view(self) {
                self.inc_cycles(11);
            } else {
                self.inc_cycles(5);
            }
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            ;
            cpir () ;
            xx ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc,
            );

            self.cpir();
            if self.reg16(BC) == 0 || self.is_set_flag(ZF) {
                self.inc_cycles(16);
            } else {
                self.inc_cycles(21);
            }
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            ;
            cpdr () ;
            xx ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc,
            );

            self.cpdr();
            if self.reg16(BC) == 0 || self.is_set_flag(ZF) {
                self.inc_cycles(16);
            } else {
                self.inc_cycles(21);
            }
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            ;
            $mnemonic: ident () ;
            xx ;
            $documented:ident ;
            z80
        ]
    ) => {
        process_instruction!{@block
            [ $opcode; $opcode_name; ; $mnemonic () ; xx ; $documented ; z80 ]
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            ;
            $mnemonic:ident($($arguments:tt)*) ;
            $cycles:expr ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc,
            );

            process_arguments!(self, $mnemonic, nn, n, d, e, ($($arguments)*));
            self.inc_cycles($cycles);
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            n ;
            $mnemonic:ident($($arguments:tt)*) ;
            $cycles:expr ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);
            let n = self.read(pc);
            self.set_reg16(PC, pc.wrapping_add(1));

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc.wrapping_add(1),
            );

            process_arguments!(self, $mnemonic, nn, n, d, e, ($($arguments)*));
            self.inc_cycles($cycles);
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            e ;
            $mnemonic:ident($($arguments:tt)*) ;
            $cycles:expr ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);
            let e = self.read(pc) as i8;
            self.set_reg16(PC, pc.wrapping_add(1));

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc.wrapping_add(1),
            );

            process_arguments!(self, $mnemonic, nn, n, d, e, ($($arguments)*));
            self.inc_cycles($cycles);
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            d ;
            $mnemonic:ident($($arguments:tt)*) ;
            $cycles:expr ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);
            let d = self.read(pc) as i8;
            self.set_reg16(PC, pc.wrapping_add(1));

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc.wrapping_add(1),
            );

            process_arguments!(self, $mnemonic, nn, n, d, e, ($($arguments)*));
            self.inc_cycles($cycles);
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            d, n ;
            $mnemonic:ident($($arguments:tt)*) ;
            $cycles:expr ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);
            let d = self.read(pc) as i8;
            let n = self.read(pc.wrapping_add(1));
            self.set_reg16(PC, pc.wrapping_add(2));

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc.wrapping_add(2),
            );

            process_arguments!(self, $mnemonic, nn, n, d, e, ($($arguments)*));
            self.inc_cycles($cycles);
        }
    };

    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            nn ;
            $mnemonic:ident($($arguments:tt)*) ;
            $cycles:expr ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self) {
            let pc = self.reg16(PC);
            let nn = Viewable::<u16>::view(Address(pc), self);
            self.set_reg16(PC, pc.wrapping_add(2));

            send_instruction_memo(
                self,
                pc.wrapping_sub(Self::PREFIX_SIZE + 1),
                pc.wrapping_add(2),
            );

            process_arguments!(self, $mnemonic, nn, n, d, e, ($($arguments)*));
            self.inc_cycles($cycles);
        }
    };

    ($anything: tt) => {};
}

macro_rules! create_function {
    ($fn_name:ident, $mac_name:ident, $prefix_size:expr) => {
        pub fn $fn_name<Z>(z: &mut Z, opcode: u8)
        where
            Z: Z80Mem + Z80No + Z80Io + Z80Internal + Memory16 + Inbox + ?Sized,
            Z::Memo: From<Z80Memo>,
        {
            #[allow(non_snake_case)]
            trait ArrayTrait {
                const PREFIX_SIZE: u16 = $prefix_size;

                fn x00(&mut self);
                fn x01(&mut self);
                fn x02(&mut self);
                fn x03(&mut self);
                fn x04(&mut self);
                fn x05(&mut self);
                fn x06(&mut self);
                fn x07(&mut self);
                fn x08(&mut self);
                fn x09(&mut self);
                fn x0A(&mut self);
                fn x0B(&mut self);
                fn x0C(&mut self);
                fn x0D(&mut self);
                fn x0E(&mut self);
                fn x0F(&mut self);
                fn x10(&mut self);
                fn x11(&mut self);
                fn x12(&mut self);
                fn x13(&mut self);
                fn x14(&mut self);
                fn x15(&mut self);
                fn x16(&mut self);
                fn x17(&mut self);
                fn x18(&mut self);
                fn x19(&mut self);
                fn x1A(&mut self);
                fn x1B(&mut self);
                fn x1C(&mut self);
                fn x1D(&mut self);
                fn x1E(&mut self);
                fn x1F(&mut self);
                fn x20(&mut self);
                fn x21(&mut self);
                fn x22(&mut self);
                fn x23(&mut self);
                fn x24(&mut self);
                fn x25(&mut self);
                fn x26(&mut self);
                fn x27(&mut self);
                fn x28(&mut self);
                fn x29(&mut self);
                fn x2A(&mut self);
                fn x2B(&mut self);
                fn x2C(&mut self);
                fn x2D(&mut self);
                fn x2E(&mut self);
                fn x2F(&mut self);
                fn x30(&mut self);
                fn x31(&mut self);
                fn x32(&mut self);
                fn x33(&mut self);
                fn x34(&mut self);
                fn x35(&mut self);
                fn x36(&mut self);
                fn x37(&mut self);
                fn x38(&mut self);
                fn x39(&mut self);
                fn x3A(&mut self);
                fn x3B(&mut self);
                fn x3C(&mut self);
                fn x3D(&mut self);
                fn x3E(&mut self);
                fn x3F(&mut self);
                fn x40(&mut self);
                fn x41(&mut self);
                fn x42(&mut self);
                fn x43(&mut self);
                fn x44(&mut self);
                fn x45(&mut self);
                fn x46(&mut self);
                fn x47(&mut self);
                fn x48(&mut self);
                fn x49(&mut self);
                fn x4A(&mut self);
                fn x4B(&mut self);
                fn x4C(&mut self);
                fn x4D(&mut self);
                fn x4E(&mut self);
                fn x4F(&mut self);
                fn x50(&mut self);
                fn x51(&mut self);
                fn x52(&mut self);
                fn x53(&mut self);
                fn x54(&mut self);
                fn x55(&mut self);
                fn x56(&mut self);
                fn x57(&mut self);
                fn x58(&mut self);
                fn x59(&mut self);
                fn x5A(&mut self);
                fn x5B(&mut self);
                fn x5C(&mut self);
                fn x5D(&mut self);
                fn x5E(&mut self);
                fn x5F(&mut self);
                fn x60(&mut self);
                fn x61(&mut self);
                fn x62(&mut self);
                fn x63(&mut self);
                fn x64(&mut self);
                fn x65(&mut self);
                fn x66(&mut self);
                fn x67(&mut self);
                fn x68(&mut self);
                fn x69(&mut self);
                fn x6A(&mut self);
                fn x6B(&mut self);
                fn x6C(&mut self);
                fn x6D(&mut self);
                fn x6E(&mut self);
                fn x6F(&mut self);
                fn x70(&mut self);
                fn x71(&mut self);
                fn x72(&mut self);
                fn x73(&mut self);
                fn x74(&mut self);
                fn x75(&mut self);
                fn x76(&mut self);
                fn x77(&mut self);
                fn x78(&mut self);
                fn x79(&mut self);
                fn x7A(&mut self);
                fn x7B(&mut self);
                fn x7C(&mut self);
                fn x7D(&mut self);
                fn x7E(&mut self);
                fn x7F(&mut self);
                fn x80(&mut self);
                fn x81(&mut self);
                fn x82(&mut self);
                fn x83(&mut self);
                fn x84(&mut self);
                fn x85(&mut self);
                fn x86(&mut self);
                fn x87(&mut self);
                fn x88(&mut self);
                fn x89(&mut self);
                fn x8A(&mut self);
                fn x8B(&mut self);
                fn x8C(&mut self);
                fn x8D(&mut self);
                fn x8E(&mut self);
                fn x8F(&mut self);
                fn x90(&mut self);
                fn x91(&mut self);
                fn x92(&mut self);
                fn x93(&mut self);
                fn x94(&mut self);
                fn x95(&mut self);
                fn x96(&mut self);
                fn x97(&mut self);
                fn x98(&mut self);
                fn x99(&mut self);
                fn x9A(&mut self);
                fn x9B(&mut self);
                fn x9C(&mut self);
                fn x9D(&mut self);
                fn x9E(&mut self);
                fn x9F(&mut self);
                fn xA0(&mut self);
                fn xA1(&mut self);
                fn xA2(&mut self);
                fn xA3(&mut self);
                fn xA4(&mut self);
                fn xA5(&mut self);
                fn xA6(&mut self);
                fn xA7(&mut self);
                fn xA8(&mut self);
                fn xA9(&mut self);
                fn xAA(&mut self);
                fn xAB(&mut self);
                fn xAC(&mut self);
                fn xAD(&mut self);
                fn xAE(&mut self);
                fn xAF(&mut self);
                fn xB0(&mut self);
                fn xB1(&mut self);
                fn xB2(&mut self);
                fn xB3(&mut self);
                fn xB4(&mut self);
                fn xB5(&mut self);
                fn xB6(&mut self);
                fn xB7(&mut self);
                fn xB8(&mut self);
                fn xB9(&mut self);
                fn xBA(&mut self);
                fn xBB(&mut self);
                fn xBC(&mut self);
                fn xBD(&mut self);
                fn xBE(&mut self);
                fn xBF(&mut self);
                fn xC0(&mut self);
                fn xC1(&mut self);
                fn xC2(&mut self);
                fn xC3(&mut self);
                fn xC4(&mut self);
                fn xC5(&mut self);
                fn xC6(&mut self);
                fn xC7(&mut self);
                fn xC8(&mut self);
                fn xC9(&mut self);
                fn xCA(&mut self);
                fn xCB(&mut self);
                fn xCC(&mut self);
                fn xCD(&mut self);
                fn xCE(&mut self);
                fn xCF(&mut self);
                fn xD0(&mut self);
                fn xD1(&mut self);
                fn xD2(&mut self);
                fn xD3(&mut self);
                fn xD4(&mut self);
                fn xD5(&mut self);
                fn xD6(&mut self);
                fn xD7(&mut self);
                fn xD8(&mut self);
                fn xD9(&mut self);
                fn xDA(&mut self);
                fn xDB(&mut self);
                fn xDC(&mut self);
                fn xDD(&mut self);
                fn xDE(&mut self);
                fn xDF(&mut self);
                fn xE0(&mut self);
                fn xE1(&mut self);
                fn xE2(&mut self);
                fn xE3(&mut self);
                fn xE4(&mut self);
                fn xE5(&mut self);
                fn xE6(&mut self);
                fn xE7(&mut self);
                fn xE8(&mut self);
                fn xE9(&mut self);
                fn xEA(&mut self);
                fn xEB(&mut self);
                fn xEC(&mut self);
                fn xED(&mut self);
                fn xEE(&mut self);
                fn xEF(&mut self);
                fn xF0(&mut self);
                fn xF1(&mut self);
                fn xF2(&mut self);
                fn xF3(&mut self);
                fn xF4(&mut self);
                fn xF5(&mut self);
                fn xF6(&mut self);
                fn xF7(&mut self);
                fn xF8(&mut self);
                fn xF9(&mut self);
                fn xFA(&mut self);
                fn xFB(&mut self);
                fn xFC(&mut self);
                fn xFD(&mut self);
                fn xFE(&mut self);
                fn xFF(&mut self);

                const ARRAY: [fn(&mut Self); 0x100] = [
                    Self::x00,
                    Self::x01,
                    Self::x02,
                    Self::x03,
                    Self::x04,
                    Self::x05,
                    Self::x06,
                    Self::x07,
                    Self::x08,
                    Self::x09,
                    Self::x0A,
                    Self::x0B,
                    Self::x0C,
                    Self::x0D,
                    Self::x0E,
                    Self::x0F,
                    Self::x10,
                    Self::x11,
                    Self::x12,
                    Self::x13,
                    Self::x14,
                    Self::x15,
                    Self::x16,
                    Self::x17,
                    Self::x18,
                    Self::x19,
                    Self::x1A,
                    Self::x1B,
                    Self::x1C,
                    Self::x1D,
                    Self::x1E,
                    Self::x1F,
                    Self::x20,
                    Self::x21,
                    Self::x22,
                    Self::x23,
                    Self::x24,
                    Self::x25,
                    Self::x26,
                    Self::x27,
                    Self::x28,
                    Self::x29,
                    Self::x2A,
                    Self::x2B,
                    Self::x2C,
                    Self::x2D,
                    Self::x2E,
                    Self::x2F,
                    Self::x30,
                    Self::x31,
                    Self::x32,
                    Self::x33,
                    Self::x34,
                    Self::x35,
                    Self::x36,
                    Self::x37,
                    Self::x38,
                    Self::x39,
                    Self::x3A,
                    Self::x3B,
                    Self::x3C,
                    Self::x3D,
                    Self::x3E,
                    Self::x3F,
                    Self::x40,
                    Self::x41,
                    Self::x42,
                    Self::x43,
                    Self::x44,
                    Self::x45,
                    Self::x46,
                    Self::x47,
                    Self::x48,
                    Self::x49,
                    Self::x4A,
                    Self::x4B,
                    Self::x4C,
                    Self::x4D,
                    Self::x4E,
                    Self::x4F,
                    Self::x50,
                    Self::x51,
                    Self::x52,
                    Self::x53,
                    Self::x54,
                    Self::x55,
                    Self::x56,
                    Self::x57,
                    Self::x58,
                    Self::x59,
                    Self::x5A,
                    Self::x5B,
                    Self::x5C,
                    Self::x5D,
                    Self::x5E,
                    Self::x5F,
                    Self::x60,
                    Self::x61,
                    Self::x62,
                    Self::x63,
                    Self::x64,
                    Self::x65,
                    Self::x66,
                    Self::x67,
                    Self::x68,
                    Self::x69,
                    Self::x6A,
                    Self::x6B,
                    Self::x6C,
                    Self::x6D,
                    Self::x6E,
                    Self::x6F,
                    Self::x70,
                    Self::x71,
                    Self::x72,
                    Self::x73,
                    Self::x74,
                    Self::x75,
                    Self::x76,
                    Self::x77,
                    Self::x78,
                    Self::x79,
                    Self::x7A,
                    Self::x7B,
                    Self::x7C,
                    Self::x7D,
                    Self::x7E,
                    Self::x7F,
                    Self::x80,
                    Self::x81,
                    Self::x82,
                    Self::x83,
                    Self::x84,
                    Self::x85,
                    Self::x86,
                    Self::x87,
                    Self::x88,
                    Self::x89,
                    Self::x8A,
                    Self::x8B,
                    Self::x8C,
                    Self::x8D,
                    Self::x8E,
                    Self::x8F,
                    Self::x90,
                    Self::x91,
                    Self::x92,
                    Self::x93,
                    Self::x94,
                    Self::x95,
                    Self::x96,
                    Self::x97,
                    Self::x98,
                    Self::x99,
                    Self::x9A,
                    Self::x9B,
                    Self::x9C,
                    Self::x9D,
                    Self::x9E,
                    Self::x9F,
                    Self::xA0,
                    Self::xA1,
                    Self::xA2,
                    Self::xA3,
                    Self::xA4,
                    Self::xA5,
                    Self::xA6,
                    Self::xA7,
                    Self::xA8,
                    Self::xA9,
                    Self::xAA,
                    Self::xAB,
                    Self::xAC,
                    Self::xAD,
                    Self::xAE,
                    Self::xAF,
                    Self::xB0,
                    Self::xB1,
                    Self::xB2,
                    Self::xB3,
                    Self::xB4,
                    Self::xB5,
                    Self::xB6,
                    Self::xB7,
                    Self::xB8,
                    Self::xB9,
                    Self::xBA,
                    Self::xBB,
                    Self::xBC,
                    Self::xBD,
                    Self::xBE,
                    Self::xBF,
                    Self::xC0,
                    Self::xC1,
                    Self::xC2,
                    Self::xC3,
                    Self::xC4,
                    Self::xC5,
                    Self::xC6,
                    Self::xC7,
                    Self::xC8,
                    Self::xC9,
                    Self::xCA,
                    Self::xCB,
                    Self::xCC,
                    Self::xCD,
                    Self::xCE,
                    Self::xCF,
                    Self::xD0,
                    Self::xD1,
                    Self::xD2,
                    Self::xD3,
                    Self::xD4,
                    Self::xD5,
                    Self::xD6,
                    Self::xD7,
                    Self::xD8,
                    Self::xD9,
                    Self::xDA,
                    Self::xDB,
                    Self::xDC,
                    Self::xDD,
                    Self::xDE,
                    Self::xDF,
                    Self::xE0,
                    Self::xE1,
                    Self::xE2,
                    Self::xE3,
                    Self::xE4,
                    Self::xE5,
                    Self::xE6,
                    Self::xE7,
                    Self::xE8,
                    Self::xE9,
                    Self::xEA,
                    Self::xEB,
                    Self::xEC,
                    Self::xED,
                    Self::xEE,
                    Self::xEF,
                    Self::xF0,
                    Self::xF1,
                    Self::xF2,
                    Self::xF3,
                    Self::xF4,
                    Self::xF5,
                    Self::xF6,
                    Self::xF7,
                    Self::xF8,
                    Self::xF9,
                    Self::xFA,
                    Self::xFB,
                    Self::xFC,
                    Self::xFD,
                    Self::xFE,
                    Self::xFF,
                ];
            }
            impl<T> ArrayTrait for T
            where
                T: Z80Mem + Z80No + Z80Io + Z80Internal + Memory16 + Inbox + ?Sized,
                T::Memo: From<Z80Memo>,
            {
                $mac_name!{process_instruction}
            }
            <Z as ArrayTrait>::ARRAY[opcode as usize](z);
        }
    };
}

create_function!{execute_noprefix, euphrates_z80_noprefix, 0}

create_function!{execute_cb, euphrates_z80_cb, 1}

create_function!{execute_ed, euphrates_z80_ed, 1}

create_function!{execute_dd, euphrates_z80_dd, 1}

create_function!{execute_fd, euphrates_z80_fd, 1}

macro_rules! process_instruction_double_prefix {
    (
        [
            $opcode:expr ;
            $opcode_name:ident ;
            ;
            $mnemonic:ident($($arguments:tt)*) ;
            $cycles:expr ;
            $documented:ident ;
            z80
        ]
    ) => {
        fn $opcode_name(&mut self, d: i8) {
            let pc = self.reg16(PC);

            send_instruction_memo(self, pc.wrapping_sub(4), pc);

            process_arguments!(self, $mnemonic, nn, n, d, e, ($($arguments)*));
            self.inc_cycles($cycles);
        }
    };
}

macro_rules! create_function_double_prefix {
    ($fn_name:ident, $mac_name:ident) => {
        pub fn $fn_name<Z>(z: &mut Z, opcode: u8)
        where
            Z: Z80Mem + Z80No + Z80Io + Z80Internal + Memory16 + Inbox + ?Sized,
            Z::Memo: From<Z80Memo>,
        {
            #[allow(non_snake_case)]
            trait ArrayTrait {
                fn x00(&mut self, d: i8);
                fn x01(&mut self, d: i8);
                fn x02(&mut self, d: i8);
                fn x03(&mut self, d: i8);
                fn x04(&mut self, d: i8);
                fn x05(&mut self, d: i8);
                fn x06(&mut self, d: i8);
                fn x07(&mut self, d: i8);
                fn x08(&mut self, d: i8);
                fn x09(&mut self, d: i8);
                fn x0A(&mut self, d: i8);
                fn x0B(&mut self, d: i8);
                fn x0C(&mut self, d: i8);
                fn x0D(&mut self, d: i8);
                fn x0E(&mut self, d: i8);
                fn x0F(&mut self, d: i8);
                fn x10(&mut self, d: i8);
                fn x11(&mut self, d: i8);
                fn x12(&mut self, d: i8);
                fn x13(&mut self, d: i8);
                fn x14(&mut self, d: i8);
                fn x15(&mut self, d: i8);
                fn x16(&mut self, d: i8);
                fn x17(&mut self, d: i8);
                fn x18(&mut self, d: i8);
                fn x19(&mut self, d: i8);
                fn x1A(&mut self, d: i8);
                fn x1B(&mut self, d: i8);
                fn x1C(&mut self, d: i8);
                fn x1D(&mut self, d: i8);
                fn x1E(&mut self, d: i8);
                fn x1F(&mut self, d: i8);
                fn x20(&mut self, d: i8);
                fn x21(&mut self, d: i8);
                fn x22(&mut self, d: i8);
                fn x23(&mut self, d: i8);
                fn x24(&mut self, d: i8);
                fn x25(&mut self, d: i8);
                fn x26(&mut self, d: i8);
                fn x27(&mut self, d: i8);
                fn x28(&mut self, d: i8);
                fn x29(&mut self, d: i8);
                fn x2A(&mut self, d: i8);
                fn x2B(&mut self, d: i8);
                fn x2C(&mut self, d: i8);
                fn x2D(&mut self, d: i8);
                fn x2E(&mut self, d: i8);
                fn x2F(&mut self, d: i8);
                fn x30(&mut self, d: i8);
                fn x31(&mut self, d: i8);
                fn x32(&mut self, d: i8);
                fn x33(&mut self, d: i8);
                fn x34(&mut self, d: i8);
                fn x35(&mut self, d: i8);
                fn x36(&mut self, d: i8);
                fn x37(&mut self, d: i8);
                fn x38(&mut self, d: i8);
                fn x39(&mut self, d: i8);
                fn x3A(&mut self, d: i8);
                fn x3B(&mut self, d: i8);
                fn x3C(&mut self, d: i8);
                fn x3D(&mut self, d: i8);
                fn x3E(&mut self, d: i8);
                fn x3F(&mut self, d: i8);
                fn x40(&mut self, d: i8);
                fn x41(&mut self, d: i8);
                fn x42(&mut self, d: i8);
                fn x43(&mut self, d: i8);
                fn x44(&mut self, d: i8);
                fn x45(&mut self, d: i8);
                fn x46(&mut self, d: i8);
                fn x47(&mut self, d: i8);
                fn x48(&mut self, d: i8);
                fn x49(&mut self, d: i8);
                fn x4A(&mut self, d: i8);
                fn x4B(&mut self, d: i8);
                fn x4C(&mut self, d: i8);
                fn x4D(&mut self, d: i8);
                fn x4E(&mut self, d: i8);
                fn x4F(&mut self, d: i8);
                fn x50(&mut self, d: i8);
                fn x51(&mut self, d: i8);
                fn x52(&mut self, d: i8);
                fn x53(&mut self, d: i8);
                fn x54(&mut self, d: i8);
                fn x55(&mut self, d: i8);
                fn x56(&mut self, d: i8);
                fn x57(&mut self, d: i8);
                fn x58(&mut self, d: i8);
                fn x59(&mut self, d: i8);
                fn x5A(&mut self, d: i8);
                fn x5B(&mut self, d: i8);
                fn x5C(&mut self, d: i8);
                fn x5D(&mut self, d: i8);
                fn x5E(&mut self, d: i8);
                fn x5F(&mut self, d: i8);
                fn x60(&mut self, d: i8);
                fn x61(&mut self, d: i8);
                fn x62(&mut self, d: i8);
                fn x63(&mut self, d: i8);
                fn x64(&mut self, d: i8);
                fn x65(&mut self, d: i8);
                fn x66(&mut self, d: i8);
                fn x67(&mut self, d: i8);
                fn x68(&mut self, d: i8);
                fn x69(&mut self, d: i8);
                fn x6A(&mut self, d: i8);
                fn x6B(&mut self, d: i8);
                fn x6C(&mut self, d: i8);
                fn x6D(&mut self, d: i8);
                fn x6E(&mut self, d: i8);
                fn x6F(&mut self, d: i8);
                fn x70(&mut self, d: i8);
                fn x71(&mut self, d: i8);
                fn x72(&mut self, d: i8);
                fn x73(&mut self, d: i8);
                fn x74(&mut self, d: i8);
                fn x75(&mut self, d: i8);
                fn x76(&mut self, d: i8);
                fn x77(&mut self, d: i8);
                fn x78(&mut self, d: i8);
                fn x79(&mut self, d: i8);
                fn x7A(&mut self, d: i8);
                fn x7B(&mut self, d: i8);
                fn x7C(&mut self, d: i8);
                fn x7D(&mut self, d: i8);
                fn x7E(&mut self, d: i8);
                fn x7F(&mut self, d: i8);
                fn x80(&mut self, d: i8);
                fn x81(&mut self, d: i8);
                fn x82(&mut self, d: i8);
                fn x83(&mut self, d: i8);
                fn x84(&mut self, d: i8);
                fn x85(&mut self, d: i8);
                fn x86(&mut self, d: i8);
                fn x87(&mut self, d: i8);
                fn x88(&mut self, d: i8);
                fn x89(&mut self, d: i8);
                fn x8A(&mut self, d: i8);
                fn x8B(&mut self, d: i8);
                fn x8C(&mut self, d: i8);
                fn x8D(&mut self, d: i8);
                fn x8E(&mut self, d: i8);
                fn x8F(&mut self, d: i8);
                fn x90(&mut self, d: i8);
                fn x91(&mut self, d: i8);
                fn x92(&mut self, d: i8);
                fn x93(&mut self, d: i8);
                fn x94(&mut self, d: i8);
                fn x95(&mut self, d: i8);
                fn x96(&mut self, d: i8);
                fn x97(&mut self, d: i8);
                fn x98(&mut self, d: i8);
                fn x99(&mut self, d: i8);
                fn x9A(&mut self, d: i8);
                fn x9B(&mut self, d: i8);
                fn x9C(&mut self, d: i8);
                fn x9D(&mut self, d: i8);
                fn x9E(&mut self, d: i8);
                fn x9F(&mut self, d: i8);
                fn xA0(&mut self, d: i8);
                fn xA1(&mut self, d: i8);
                fn xA2(&mut self, d: i8);
                fn xA3(&mut self, d: i8);
                fn xA4(&mut self, d: i8);
                fn xA5(&mut self, d: i8);
                fn xA6(&mut self, d: i8);
                fn xA7(&mut self, d: i8);
                fn xA8(&mut self, d: i8);
                fn xA9(&mut self, d: i8);
                fn xAA(&mut self, d: i8);
                fn xAB(&mut self, d: i8);
                fn xAC(&mut self, d: i8);
                fn xAD(&mut self, d: i8);
                fn xAE(&mut self, d: i8);
                fn xAF(&mut self, d: i8);
                fn xB0(&mut self, d: i8);
                fn xB1(&mut self, d: i8);
                fn xB2(&mut self, d: i8);
                fn xB3(&mut self, d: i8);
                fn xB4(&mut self, d: i8);
                fn xB5(&mut self, d: i8);
                fn xB6(&mut self, d: i8);
                fn xB7(&mut self, d: i8);
                fn xB8(&mut self, d: i8);
                fn xB9(&mut self, d: i8);
                fn xBA(&mut self, d: i8);
                fn xBB(&mut self, d: i8);
                fn xBC(&mut self, d: i8);
                fn xBD(&mut self, d: i8);
                fn xBE(&mut self, d: i8);
                fn xBF(&mut self, d: i8);
                fn xC0(&mut self, d: i8);
                fn xC1(&mut self, d: i8);
                fn xC2(&mut self, d: i8);
                fn xC3(&mut self, d: i8);
                fn xC4(&mut self, d: i8);
                fn xC5(&mut self, d: i8);
                fn xC6(&mut self, d: i8);
                fn xC7(&mut self, d: i8);
                fn xC8(&mut self, d: i8);
                fn xC9(&mut self, d: i8);
                fn xCA(&mut self, d: i8);
                fn xCB(&mut self, d: i8);
                fn xCC(&mut self, d: i8);
                fn xCD(&mut self, d: i8);
                fn xCE(&mut self, d: i8);
                fn xCF(&mut self, d: i8);
                fn xD0(&mut self, d: i8);
                fn xD1(&mut self, d: i8);
                fn xD2(&mut self, d: i8);
                fn xD3(&mut self, d: i8);
                fn xD4(&mut self, d: i8);
                fn xD5(&mut self, d: i8);
                fn xD6(&mut self, d: i8);
                fn xD7(&mut self, d: i8);
                fn xD8(&mut self, d: i8);
                fn xD9(&mut self, d: i8);
                fn xDA(&mut self, d: i8);
                fn xDB(&mut self, d: i8);
                fn xDC(&mut self, d: i8);
                fn xDD(&mut self, d: i8);
                fn xDE(&mut self, d: i8);
                fn xDF(&mut self, d: i8);
                fn xE0(&mut self, d: i8);
                fn xE1(&mut self, d: i8);
                fn xE2(&mut self, d: i8);
                fn xE3(&mut self, d: i8);
                fn xE4(&mut self, d: i8);
                fn xE5(&mut self, d: i8);
                fn xE6(&mut self, d: i8);
                fn xE7(&mut self, d: i8);
                fn xE8(&mut self, d: i8);
                fn xE9(&mut self, d: i8);
                fn xEA(&mut self, d: i8);
                fn xEB(&mut self, d: i8);
                fn xEC(&mut self, d: i8);
                fn xED(&mut self, d: i8);
                fn xEE(&mut self, d: i8);
                fn xEF(&mut self, d: i8);
                fn xF0(&mut self, d: i8);
                fn xF1(&mut self, d: i8);
                fn xF2(&mut self, d: i8);
                fn xF3(&mut self, d: i8);
                fn xF4(&mut self, d: i8);
                fn xF5(&mut self, d: i8);
                fn xF6(&mut self, d: i8);
                fn xF7(&mut self, d: i8);
                fn xF8(&mut self, d: i8);
                fn xF9(&mut self, d: i8);
                fn xFA(&mut self, d: i8);
                fn xFB(&mut self, d: i8);
                fn xFC(&mut self, d: i8);
                fn xFD(&mut self, d: i8);
                fn xFE(&mut self, d: i8);
                fn xFF(&mut self, d: i8);

                const ARRAY: [fn(&mut Self, i8); 0x100] = [
                    Self::x00,
                    Self::x01,
                    Self::x02,
                    Self::x03,
                    Self::x04,
                    Self::x05,
                    Self::x06,
                    Self::x07,
                    Self::x08,
                    Self::x09,
                    Self::x0A,
                    Self::x0B,
                    Self::x0C,
                    Self::x0D,
                    Self::x0E,
                    Self::x0F,
                    Self::x10,
                    Self::x11,
                    Self::x12,
                    Self::x13,
                    Self::x14,
                    Self::x15,
                    Self::x16,
                    Self::x17,
                    Self::x18,
                    Self::x19,
                    Self::x1A,
                    Self::x1B,
                    Self::x1C,
                    Self::x1D,
                    Self::x1E,
                    Self::x1F,
                    Self::x20,
                    Self::x21,
                    Self::x22,
                    Self::x23,
                    Self::x24,
                    Self::x25,
                    Self::x26,
                    Self::x27,
                    Self::x28,
                    Self::x29,
                    Self::x2A,
                    Self::x2B,
                    Self::x2C,
                    Self::x2D,
                    Self::x2E,
                    Self::x2F,
                    Self::x30,
                    Self::x31,
                    Self::x32,
                    Self::x33,
                    Self::x34,
                    Self::x35,
                    Self::x36,
                    Self::x37,
                    Self::x38,
                    Self::x39,
                    Self::x3A,
                    Self::x3B,
                    Self::x3C,
                    Self::x3D,
                    Self::x3E,
                    Self::x3F,
                    Self::x40,
                    Self::x41,
                    Self::x42,
                    Self::x43,
                    Self::x44,
                    Self::x45,
                    Self::x46,
                    Self::x47,
                    Self::x48,
                    Self::x49,
                    Self::x4A,
                    Self::x4B,
                    Self::x4C,
                    Self::x4D,
                    Self::x4E,
                    Self::x4F,
                    Self::x50,
                    Self::x51,
                    Self::x52,
                    Self::x53,
                    Self::x54,
                    Self::x55,
                    Self::x56,
                    Self::x57,
                    Self::x58,
                    Self::x59,
                    Self::x5A,
                    Self::x5B,
                    Self::x5C,
                    Self::x5D,
                    Self::x5E,
                    Self::x5F,
                    Self::x60,
                    Self::x61,
                    Self::x62,
                    Self::x63,
                    Self::x64,
                    Self::x65,
                    Self::x66,
                    Self::x67,
                    Self::x68,
                    Self::x69,
                    Self::x6A,
                    Self::x6B,
                    Self::x6C,
                    Self::x6D,
                    Self::x6E,
                    Self::x6F,
                    Self::x70,
                    Self::x71,
                    Self::x72,
                    Self::x73,
                    Self::x74,
                    Self::x75,
                    Self::x76,
                    Self::x77,
                    Self::x78,
                    Self::x79,
                    Self::x7A,
                    Self::x7B,
                    Self::x7C,
                    Self::x7D,
                    Self::x7E,
                    Self::x7F,
                    Self::x80,
                    Self::x81,
                    Self::x82,
                    Self::x83,
                    Self::x84,
                    Self::x85,
                    Self::x86,
                    Self::x87,
                    Self::x88,
                    Self::x89,
                    Self::x8A,
                    Self::x8B,
                    Self::x8C,
                    Self::x8D,
                    Self::x8E,
                    Self::x8F,
                    Self::x90,
                    Self::x91,
                    Self::x92,
                    Self::x93,
                    Self::x94,
                    Self::x95,
                    Self::x96,
                    Self::x97,
                    Self::x98,
                    Self::x99,
                    Self::x9A,
                    Self::x9B,
                    Self::x9C,
                    Self::x9D,
                    Self::x9E,
                    Self::x9F,
                    Self::xA0,
                    Self::xA1,
                    Self::xA2,
                    Self::xA3,
                    Self::xA4,
                    Self::xA5,
                    Self::xA6,
                    Self::xA7,
                    Self::xA8,
                    Self::xA9,
                    Self::xAA,
                    Self::xAB,
                    Self::xAC,
                    Self::xAD,
                    Self::xAE,
                    Self::xAF,
                    Self::xB0,
                    Self::xB1,
                    Self::xB2,
                    Self::xB3,
                    Self::xB4,
                    Self::xB5,
                    Self::xB6,
                    Self::xB7,
                    Self::xB8,
                    Self::xB9,
                    Self::xBA,
                    Self::xBB,
                    Self::xBC,
                    Self::xBD,
                    Self::xBE,
                    Self::xBF,
                    Self::xC0,
                    Self::xC1,
                    Self::xC2,
                    Self::xC3,
                    Self::xC4,
                    Self::xC5,
                    Self::xC6,
                    Self::xC7,
                    Self::xC8,
                    Self::xC9,
                    Self::xCA,
                    Self::xCB,
                    Self::xCC,
                    Self::xCD,
                    Self::xCE,
                    Self::xCF,
                    Self::xD0,
                    Self::xD1,
                    Self::xD2,
                    Self::xD3,
                    Self::xD4,
                    Self::xD5,
                    Self::xD6,
                    Self::xD7,
                    Self::xD8,
                    Self::xD9,
                    Self::xDA,
                    Self::xDB,
                    Self::xDC,
                    Self::xDD,
                    Self::xDE,
                    Self::xDF,
                    Self::xE0,
                    Self::xE1,
                    Self::xE2,
                    Self::xE3,
                    Self::xE4,
                    Self::xE5,
                    Self::xE6,
                    Self::xE7,
                    Self::xE8,
                    Self::xE9,
                    Self::xEA,
                    Self::xEB,
                    Self::xEC,
                    Self::xED,
                    Self::xEE,
                    Self::xEF,
                    Self::xF0,
                    Self::xF1,
                    Self::xF2,
                    Self::xF3,
                    Self::xF4,
                    Self::xF5,
                    Self::xF6,
                    Self::xF7,
                    Self::xF8,
                    Self::xF9,
                    Self::xFA,
                    Self::xFB,
                    Self::xFC,
                    Self::xFD,
                    Self::xFE,
                    Self::xFF,
                ];
            }

            impl<T> ArrayTrait for T
            where
                T: Z80Mem + Z80No + Z80Io + Z80Internal + Memory16 + Inbox + ?Sized,
                T::Memo: From<Z80Memo>,
            {
                $mac_name!{process_instruction_double_prefix}
            }

            let pc = z.reg16(PC);
            let d = z.read(pc.wrapping_sub(2)) as i8;
            <Z as ArrayTrait>::ARRAY[opcode as usize](z, d);
        }
    };
}

create_function_double_prefix!{execute_ddcb, euphrates_z80_ddcb}

create_function_double_prefix!{execute_fdcb, euphrates_z80_fdcb}
