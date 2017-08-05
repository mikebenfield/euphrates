
use ::bits::*;

use super::instructions;
use hardware::z80::types::*;

// pub fn execute1<Z: Z80>(z: &mut Z) {
//     generated_dispatch::main_dispatch(z);
// }

pub fn execute1<Z: Z80>(z: &mut Z) {
    let mut opcode: u8;
    let mut n: u8;
    let mut nn: u16;
    let mut e: i8;
    let mut d: i8;

    fn read_pc<Z: Z80>(z: &mut Z) -> u8 {
        let pc = PC.get(z); 
        let opcode: u8 = Address(pc).get(z); 
        PC.set(z, pc + 1); 
        log_minor!(z, "Z80: opcode: 0x{:0>2X}", opcode); 
        opcode 
    }

    // macro_rules! process_instructions {
    //     (@
    //      $plain_instructions: expr,
    //      $ed_instructions: expr,
    //      $cb_instructions: expr,
    //      $dd_instructions: expr,
    //      $fd_instructions: expr,
    //      $ddcb_instructions: expr,
    //      $fdcb_instructions: expr,
    //     ) => {
    //         enum Prefix {
    //             NoPrefix,
    //             Ed,
    //             Cb,
    //             Dd,
    //             Fd,
    //             Ddcb,
    //             Fdcb,
    //         }

    //         let mut prefix = Prefix::NoPrefix;
            
    //         // It would be most natural to me to structure each prefixed
    //         // dispatch as a function, rather than in a loop like this. But they
    //         // can't easily be made functions since they need to capture the
    //         // surrounding environment (since, for instance, the arguments list
    //         // may contain nn, d, e, etc). And they can't easily be made
    //         // closures since they need to be mutually recursive. So, here's a
    //         // loop.
    //         loop {
    //             match prefix {
    //                 Prefix::NoPrefix => {
    //                     opcode = read_pc(z);
    //                     $plain_instructions;
    //                     if opcode == 0xED {
    //                         prefix = Prefix::Ed;
    //                         continue;
    //                     }
    //                     if opcode == 0xCB {
    //                         prefix = Prefix::Cb;
    //                         continue;
    //                     }
    //                     if opcode == 0xDD {
    //                         prefix = Prefix::Dd;
    //                         continue;
    //                     }
    //                     if opcode == 0xFD {
    //                         prefix = Prefix::Fd;
    //                         continue;
    //                     }
    //                     panic!("Z80: can't happen: missing opcode 0x{:0>2X}", opcode);
    //                 },
    //                 Prefix::Ed => {
    //                     opcode = read_pc(z);
    //                     $ed_instructions;
    //                     z.cycles(8);
    //                     return;
    //                 },
    //                 Prefix::Cb => {
    //                     opcode = read_pc(z);
    //                     $cb_instructions;
    //                     panic!("Can't happen!: missing opcode 0xCB 0x{:0>2X}", opcode);
    //                 },
    //                 Prefix::Ddcb => {
    //                     d = read_pc(z) as i8;
    //                     opcode = read_pc(z);
    //                     $ddcb_instructions;
    //                     z.cycles(12);
    //                     return;
    //                 },
    //                 Prefix::Fdcb => {
    //                     d = read_pc(z) as i8;
    //                     opcode = read_pc(z);
    //                     $fdcb_instructions;
    //                     z.cycles(12);
    //                     return;
    //                 },
    //                 Prefix::Dd => {
    //                     $dd_instructions;
    //                     if opcode == 0xCB {
    //                         prefix = Prefix::Ddcb;
    //                         continue;
    //                     }
    //                     if opcode == 0xDD {
    //                         z.cycles(4);
    //                         prefix = Prefix::Dd;
    //                         continue;
    //                     }
    //                     if opcode == 0xFD {
    //                         z.cycles(4);
    //                         prefix = Prefix::Fd;
    //                         continue;
    //                     }
    //                     z.cycles(4);
    //                     return;
    //                 },
    //                 Prefix::Fd => {
    //                     $fd_instructions;
    //                     if opcode == 0xCB {
    //                         prefix = Prefix::Ddcb;
    //                         continue;
    //                     }
    //                     if opcode == 0xDD {
    //                         z.cycles(4);
    //                         prefix = Prefix::Dd;
    //                         continue;
    //                     }
    //                     if opcode == 0xFD {
    //                         z.cycles(4);
    //                         prefix = Prefix::Fd;
    //                         continue;
    //                     }
    //                     z.cycles(4);
    //                     return;
    //                 },
    //             }
    //         }
    //     };


    macro_rules! apply_args {
        (@ ($x: tt + d)) => {
            Shift(apply_args!(@ $x), d)
        };
        (@ ($x: tt)) => {
            Address(apply_args!(@ $x))
        };
        (@ $x: tt) => {
            $x
        };
        ($mnemonic: ident, $($args: tt),*) => {
            instructions::$mnemonic
            (
                z,
                $(
                    apply_args!(@ $args)
                ),*
            )
        };
    }

    macro_rules! instruction_noprefix {
        ([$code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ([$code: expr, e] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                e = read_pc(z) as i8;
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ([$code: expr, d] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ([$code: expr, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                n = read_pc(z);
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ([$code: expr, n, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                nn = to16(n1, n2);
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_ed {
        ([0xED, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ([0xED, $code: expr, n, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                nn = to16(n1, n2);
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_cb {
        ([0xCB, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_ddcb {
        ([0xDD, 0xCB, d, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_fdcb {
        ([0xFD, 0xCB, d, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_dd {
        ([0xDD, $code: expr, d, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                n = read_pc(z);
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ([0xDD, $code: expr, d] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ([0xDD, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ($($ignore: tt)*) => {
        };
    }
 
    macro_rules! instruction_fd {
        ([0xFD, $code: expr, d, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                n = read_pc(z);
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ([0xFD, $code: expr, d] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ([0xFD, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $cycles: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                log_minor!(z, "Z80: op: {}", stringify!($mnemonic $($arguments,)*));
                apply_args!($mnemonic, $($arguments),*);
                z.cycles($cycles);
                return;
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    enum Prefix {
        NoPrefix,
        Ed,
        Cb,
        Dd,
        Fd,
        Ddcb,
        Fdcb,
    }

    let mut prefix = Prefix::NoPrefix;

    loop {
        match prefix {
            Prefix::NoPrefix => {
                opcode = read_pc(z);
                process_instructions!(instruction_noprefix, d, e, n, nn);
                if opcode == 0xED {
                    prefix = Prefix::Ed;
                    continue;
                }
                if opcode == 0xCB {
                    prefix = Prefix::Cb;
                    continue;
                }
                if opcode == 0xDD {
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    prefix = Prefix::Fd;
                    continue;
                }
                panic!("Z80: can't happen: missing opcode {:0>2X}", opcode);
            },
            Prefix::Ed => {
                opcode = read_pc(z);
                process_instructions!(instruction_ed, d, e, n, nn);
                z.cycles(8);
                return;
            },
            Prefix::Cb => {
                opcode = read_pc(z);
                process_instructions!(instruction_cb, d, e, n, nn);
                z.cycles(8);
                return;
            },
            Prefix::Ddcb => {
                d = read_pc(z) as i8;
                opcode = read_pc(z);
                process_instructions!(instruction_ddcb, d, e, n, nn);
                panic!("Z80: can't happen: missing opcode DD CB {:0>2X} {:0>2X}", d as u8, opcode);
            },
            Prefix::Fdcb => {
                d = read_pc(z) as i8;
                opcode = read_pc(z);
                process_instructions!(instruction_fdcb, d, e, n, nn);
                panic!("Z80: can't happen: missing opcode FD CB {:0>2X} {:0>2X}", d as u8, opcode);
            },
            Prefix::Dd => {
                opcode = read_pc(z);
                process_instructions!(instruction_dd, d, e, n, nn);
                if opcode == 0xED {
                    prefix = Prefix::Ed;
                    continue;
                }
                if opcode == 0xCB {
                    prefix = Prefix::Ddcb;
                    continue;
                }
                if opcode == 0xDD {
                    z.cycles(4);
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    z.cycles(4);
                    prefix = Prefix::Fd;
                    continue;
                }
                prefix = Prefix::NoPrefix;
                continue;
            },
            Prefix::Fd => {
                opcode = read_pc(z);
                process_instructions!(instruction_fd, d, e, n, nn);
                if opcode == 0xED {
                    prefix = Prefix::Ed;
                    continue;
                }
                if opcode == 0xCB {
                    prefix = Prefix::Fdcb;
                    continue;
                }
                if opcode == 0xDD {
                    z.cycles(4);
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    z.cycles(4);
                    prefix = Prefix::Fd;
                    continue;
                }
                prefix = Prefix::NoPrefix;
                continue;
            },
            _ => {
                unimplemented!();
            },
        }
    }
}
        //     Prefix::Ed => {
        //         opcode = read_pc(z);
        //         $ed_instructions;
        //         z.cycles(8);
        //         return;
        //     },
        //     Prefix::Cb => {
        //         opcode = read_pc(z);
        //         $cb_instructions;
        //         panic!("Can't happen!: missing opcode 0xCB 0x{:0>2X}", opcode);
        //     },
        //     Prefix::Ddcb => {
        //         d = read_pc(z) as i8;
        //         opcode = read_pc(z);
        //         $ddcb_instructions;
        //         z.cycles(12);
        //         return;
        //     },
        //     Prefix::Fdcb => {
        //         d = read_pc(z) as i8;
        //         opcode = read_pc(z);
        //         $fdcb_instructions;
        //         z.cycles(12);
        //         return;
        //     },
        //     Prefix::Dd => {
        //         $dd_instructions;
        //         if opcode == 0xCB {
        //             prefix = Prefix::Ddcb;
        //             continue;
        //         }
        //         if opcode == 0xDD {
        //             z.cycles(4);
        //             prefix = Prefix::Dd;
        //             continue;
        //         }
        //         if opcode == 0xFD {
        //             z.cycles(4);
        //             prefix = Prefix::Fd;
        //             continue;
        //         }
        //         z.cycles(4);
        //         return;
        //     },
        //     Prefix::Fd => {
        //         $fd_instructions;
        //         if opcode == 0xCB {
        //             prefix = Prefix::Ddcb;
        //             continue;
        //         }
        //         if opcode == 0xDD {
        //             z.cycles(4);
        //             prefix = Prefix::Dd;
        //             continue;
        //         }
        //         if opcode == 0xFD {
        //             z.cycles(4);
        //             prefix = Prefix::Fd;
        //             continue;
        //         }
        //         z.cycles(4);
        //         return;
        //     },
//     process_instructions_noprefix!{
// [[0x00]             ; nop        ; []          ;   4 ; false ;]
// [[0x01, n, n]         ; ld16       ; [BC,nn]       ;  10 ; false ;]
// [[0x06, n]           ; ld         ; [B,n]         ;   7 ; false ;]
// [[0x10, e]           ; djnz       ; [e]           ;   8 ; false ;]
//     }
// }
