
operations! {
    execute, x, y,

    // 8-Bit Load Group
    ///////////////////

    // LD r, r'
    [
        r r, (0x40 | x << 3 | y),
        [4],
        load8(x, y)
    ]

    // LD p, p'
    [
        0xDD p p, (0x40 | x << 3 | y),
        [4, 4],
        load8(x, y)
    ]

    // LD q, q'
    [
        0xFD q q, (0x40 | x << 3 | y),
        [4, 4],
        load8(x, y)
    ]

    // LD r, n
    [
        r, (x << 3 | 6),
        [4,3],
        read_n!(load8(x, @))
    ]

    // LD p, n
    [
        0xDD p, (x << 3 | 6),
        [4, 4, 3],
        read_n!(load8(x, @))
    ]

    // LD q, n
    [
        0xFD q, (x << 3 | 6),
        [4, 4, 3],
        read_n!(load8(x, @))
    ]

    // LD r, (HL)
    [
        r, (0x40 | (x << 3) | 6),
        [4, 3],
        load8(x, Address(HL))
    ]

    // LD r, (IX+d)
    [
        0xDD r, (0x40 | (x << 3) | 6),
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, load8(x, @))
    ]

    // LD r, (IY+d)
    [
        0xFD r, (0x40 | (x << 3) | 6),
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, load8(x, @))
    ]

    // LD (HL), r
    [
        r, (0x70 | x),
        [4,3],
        load8(Address(HL), x)
    ]

    // LD (IX+d), r
    [
        0xDD r, (0x70 | x),
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, load8(@, x))
    ]

    // LD (IY+d), r
    [
        0xFD r, (0x70 | x),
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, load8(@, x))
    ]

    // LD (HL), n
    [
        0x36,
        [4,3,3],
        read_n!(load8(Address(HL), @))
    ]

    // LD (IX+d), n
    [
        0xDD 0x36,
        [4,4,3,5,3],
        shift_d_read_n!(IX, load8(@, @))
    ]

    // LD (IY+d), n
    [
        0xFD 0x36,
        [4,4,3,5,3],
        shift_d_read_n!(IY, load8(@, @))
    ]

    // LD A, (BC)
    [
        0x0A,
        [4,3],
        load8(A, Address(BC))
    ]

    // LD A, (DE)
    [
        0x1A,
        [4,3],
        load8(A, Address(DE))
    ]

    // LD A, (nn)
    [
        0x3A,
        [4,3,3,3],
        read_pnn!(load8(A, @))
    ]

    // LD (BC), A
    [
        0x02,
        [4,3],
        load8(Address(BC), A)
    ]

    // LD (DE), A
    [
        0x12,
        [4,3],
        load8(Address(DE), A)
    ]

    // LD (nn), A
    [
        0x32,
        [4,3,3,3],
        read_pnn!(load8(@, A))
    ]

    // LD A, I
    [
        0xED 0x57,
        [4,5],
        load8_ir(I)
    ]

    // LD A, R
    [
        0xED 0x5F,
        [4,5],
        load8_ir(R)
    ]

    // LD I, A
    [
        0xED 0x47,
        [4,5],
        load8(I, A)
    ]

    // LD R, A
    [
        0xED 0x4F,
        [4,5],
        load8(R, A)
    ]

    // 16-Bit Load Group
    ////////////////////

    // LD dd, nn
    [
        dd, (x << 4 | 1),
        [4,3,3],
        read_nn!(load16(x, @))
    ]

    // LD IX, nn
    [
        0xDD 0x21,
        [4,4,3,3],
        read_nn!(load16(IX, @))
    ]

    // LD IY, nn
    [
        0xFD 0x21,
        [4,4,3,3],
        read_nn!(load16(IY, @))
    ]

    // LD dd, (nn)
    [
        0xED dd, (x << 4 | 0x4B),
        [4,4,3,3,3,3],
        read_pnn!(load16(x, @))
    ]

    // LD HL, (nn)
    [
        0x2A,
        [4,3,3,3,3],
        read_pnn!(load16(HL, @))
    ]

    // LD IX, (nn)
    [
        0xDD 0x2A,
        [4,4,3,3,3,3],
        read_pnn!(load16(IX, @))
    ]

    // LD IY, (nn)
    [
        0xFD 0x2A,
        [4,4,3,3,3,3],
        read_pnn!(load16(IY, @))
    ]

    // LD (nn), HL
    [
        0x22,
        [4,3,3,3,3],
        read_pnn!(load16(@, HL))
    ]

    // LD (nn), dd
    [
        0xED ss, (0x43 | x << 4),
        [4,4,3,3,3,3],
        read_pnn!(load16(@, x))
    ]

    // LD (nn), IX
    [
        0xDD 0x22,
        [4,4,3,3,3,3],
        read_pnn!(load16(@, IX))
    ]

    // LD (nn), IY
    [
        0xFD 0x22,
        [4,4,3,3,3,3],
        read_pnn!(load16(@, IY))
    ]

    // LD SP, HL
    [
        0xF9,
        [6],
        load16(SP, HL)
    ]

    // LD SP, IX
    [
        0xDD 0xF9,
        [4,6],
        load16(SP, IX)
    ]

    // LD SP, IY
    [
        0xFD 0xF9,
        [4,6],
        load16(SP, IY)
    ]

    // PUSH qq
    [
        qq1, (x << 4 | 0xC5),
        [5,3,3],
        push(x)
    ]

    // PUSH IX
    [
        0xDD 0xE5,
        [4,5,3,3],
        push(IX)
    ]

    // PUSH IY
    [
        0xFD 0xE5,
        [4,5,3,3],
        push(IY)
    ]

    // POP qq
    [
        qq1, (x << 4 | 0xC1),
        [4,3,3],
        pop(x)
    ]

    // POP IX
    [
        0xDD 0xE1,
        [4,4,3,3],
        pop(IX)
    ]

    // POP IY
    [
        0xFD 0xE1,
        [4,4,3,3],
        pop(IY)
    ]

    //// Exchange, Block Transfer, Search Group
    ///////////////////////////////////////////

    // EX DE, HL
    [
        0xEB,
        [4],
        ex(DE, HL)
    ]

    // EX AF, AF'
    [
        0x08,
        [4],
        ex(AF, AF0)
    ]

    // EXX
    [
        0xD9,
        [4],
        exx()
    ]

    // EX (SP), HL
    [
        0xE3,
        [4,3,4,3,5],
        ex(Address(SP), HL)
    ]

    // EX (SP), IX
    [
        0xDD 0xE3,
        [4,4,3,4,3,5],
        ex(Address(SP), IX)
    ]

    // EX (SP), IY
    [
        0xFD 0xE3,
        [4,4,3,4,3,5],
        ex(Address(SP), IY)
    ]

    // LDI
    [
        0xED 0xA0,
        [4,4,3,5],
        ldi()
    ]

    // LDIR
    [
        0xED 0xB0,
        [],
        ldir()
    ]

    // LDD
    [
        0xED 0xA8,
        [4,4,3,5],
        ldd()
    ]

    // LDDR
    [
        0xED 0xB8,
        [],
        lddr()
    ]

    // CPI
    [
        0xED 0xA1,
        [4,4,3,5],
        cpi()
    ]

    // CPIR
    [
        0xED 0xB1,
        [],
        cpir()
    ]

    // CPD
    [
        0xED 0xA9,
        [4,4,3,5],
        cpd()
    ]

    // CPDR
    [
        0xED 0xB9,
        [],
        cpdr()
    ]

    //// 8-Bit Arithmetic Group
    ///////////////////////////

    // ADD A, r
    [
        r, (x | 0x80),
        [4],
        add8(x)
    ]

    // ADD A, p
    [
        0xDD p, (x | 0x80),
        [4,4],
        add8(x)
    ]

    // ADD A, q
    [
        0xFD q, (x | 0x80),
        [4,4],
        add8(x)
    ]

    // ADD A, n
    [
        0xC6,
        [4,3],
        read_n!(add8(@))
    ]

    // ADD A, (HL)
    [
        0x86,
        [4,3],
        add8(Address(HL))
    ]

    // ADD A, (IX+d)
    [
        0xDD 0x86,
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, add8(@))
    ]

    // ADD A, (IY+d)
    [
        0xFD 0x86,
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, add8(@))
    ]

    // ADC A, r
    [
        r, (x | 0x88),
        [4],
        add8(x)
    ]

    // ADC A, p
    [
        0xDD p, (x | 0x88),
        [4,4],
        adc8(x)
    ]

    // ADC A, q
    [
        0xFD q, (x | 0x88),
        [4,4],
        adc8(x)
    ]

    // ADC A, n
    [
        0xCE,
        [4,3],
        read_n!(adc8(@))
    ]

    // ADC A, (HL)
    [
        0x8E,
        [4,3],
        adc8(Address(HL))
    ]

    // ADC A, (IX+d)
    [
        0xDD 0x8E,
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, adc8(@))
    ]

    // ADC A, (IY+d)
    [
        0xFD 0x8E,
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, adc8(@))
    ]

    // SUB r
    [
        r, (x | 0x90),
        [4],
        sub8(x)
    ]

    // SUB p
    [
        0xDD p, (x | 0x90),
        [4,4],
        sub8(x)
    ]

    // SUB q
    [
        0xFD q, (x | 0x90),
        [4,4],
        sub8(x)
    ]

    // SUB n
    [
        0xD6,
        [4,3],
        read_n!(sub8(@))
    ]

    // SUB (HL)
    [
        0x96,
        [4,3],
        sub8(Address(HL))
    ]

    // SUB (IX+d)
    [
        0xDD 0x96,
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, sub8(@))
    ]

    // SUB (IY+d)
    [
        0xFD 0x96,
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, sub8(@))
    ]

    // SBC A, r
    [
        r, (x | 0x98),
        [4],
        sbc8(x)
    ]

    // SBC A, p
    [
        0xDD p, (x | 0x98),
        [4,4],
        sbc8(x)
    ]

    // SBC A, q
    [
        0xFD q, (x | 0x98),
        [4,4],
        sbc8(x)
    ]

    // SBC A, n
    [
        0xDE,
        [4,3],
        read_n!(sbc8(@))
    ]

    // SBC A, (HL)
    [
        0x9E,
        [4,3],
        sbc8(Address(HL))
    ]

    // SBC A, (IX+d)
    [
        0xDD 0x9E,
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, sbc8(@))
    ]

    // SBC A, (IY+d)
    [
        0xFD 0x9E,
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, sbc8(@))
    ]

    // AND r
    [
        r, (x | 0xA0),
        [4],
        and(x)
    ]

    // AND p
    [
        0xDD p, (x | 0xA0),
        [4,4],
        and(x)
    ]

    // AND q
    [
        0xFD q, (x | 0xA0),
        [4,4],
        and(x)
    ]

    // AND n
    [
        0xE6,
        [4,3],
        read_n!(and(@))
    ]

    // AND (HL)
    [
        0xA6,
        [4,3],
        and(Address(HL))
    ]

    // AND (IX+d)
    [
        0xDD 0xA6,
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, and(@))
    ]

    // AND (IY+d)
    [
        0xFD 0xA6,
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, and(@))
    ]

    // OR r
    [
        r, (x | 0xB0),
        [4],
        or(x)
    ]

    // OR p
    [
        0xDD p, (x | 0xB0),
        [4,4],
        or(x)
    ]

    // OR q
    [
        0xFD q, (x | 0xB0),
        [4,4],
        or(x)
    ]

    // OR n
    [
        0xF6,
        [4,3],
        read_n!(or(@))
    ]

    // OR (HL)
    [
        0xB6,
        [4,3],
        or(Address(HL))
    ]

    // OR (IX+d)
    [
        0xDD 0xB6,
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, or(@))
    ]

    // OR (IY+d)
    [
        0xFD 0xB6,
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, or(@))
    ]

    // XOR r
    [
        r, (x | 0xA8),
        [4],
        xor(x)
    ]

    // XOR p
    [
        0xDD p, (x | 0xA8),
        [4,4],
        xor(x)
    ]

    // XOR q
    [
        0xFD q, (x | 0xA8),
        [4,4],
        xor(x)
    ]

    // XOR n
    [
        0xEE,
        [4,3],
        read_n!(xor(@))
    ]

    // XOR (HL)
    [
        0xAE,
        [4,3],
        xor(Address(HL))
    ]

    // XOR (IX+d)
    [
        0xDD 0xAE,
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, xor(@))
    ]

    // XOR (IY+d)
    [
        0xFD 0xAE,
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, xor(@))
    ]

    // CP r
    [
        r, (x | 0xB8),
        [4],
        cp(x)
    ]

    // CP p
    [
        0xDD p, (x | 0xB8),
        [4,4],
        cp(x)
    ]

    // CP q
    [
        0xFD q, (x | 0xB8),
        [4,4],
        cp(x)
    ]

    // CP n
    [
        0xFE,
        [4,3],
        read_n!(cp(@))
    ]

    // CP (HL)
    [
        0xBE,
        [4,3],
        cp(Address(HL))
    ]

    // CP (IX+d)
    [
        0xDD 0xBE,
        [4,4,3,5,3],
        shift_d!(IX, 0, 1, cp(@))
    ]

    // CP (IY+d)
    [
        0xFD 0xBE,
        [4,4,3,5,3],
        shift_d!(IY, 0, 1, cp(@))
    ]

    // INC r
    [
        r, (x << 3 | 4),
        [4],
        inc8(x)
    ]

    // INC p
    [
        0xDD r, (x << 3 | 4),
        [4,4],
        inc8(x)
    ]

    // INC q
    [
        0xFD r, (x << 3 | 4),
        [4,4],
        inc8(x)
    ]

    // INC (HL)
    [
        0x34,
        [4,4,3],
        inc8(Address(HL))
    ]

    // INC (IX+d)
    [
        0xDD 0x34,
        [4,4,3,5,4,3],
        shift_d!(IX, 0, 1, inc8(@))
    ]

    // INC (IY+d)
    [
        0xFD 0x34,
        [4,4,3,5,4,3],
        shift_d!(IY, 0, 1, inc8(@))
    ]

    // DEC r
    [
        r, (x << 3 | 5),
        [4],
        dec8(x)
    ]

    // DEC p
    [
        0xDD r, (x << 3 | 5),
        [4,4],
        dec8(x)
    ]

    // DEC q
    [
        0xFD r, (x << 3 | 5),
        [4,4],
        dec8(x)
    ]

    // DEC (HL)
    [
        0x35,
        [4,4,3],
        dec8(Address(HL))
    ]

    // DEC (IX+d)
    [
        0xDD 0x35,
        [4,4,3,5,4,3],
        shift_d!(IX, 0, 1, dec8(@))
    ]

    // DEC (IY+d)
    [
        0xFD 0x35,
        [4,4,3,5,4,3],
        shift_d!(IY, 0, 1, dec8(@))
    ]

    //// General-Purpose Arithmetic Group
    /////////////////////////////

    // DAA
    [
        0x27,
        [4],
        daa()
    ]


    // CPL
    [
        0x2F,
        [4],
        cpl()
    ]

    // NEG
    [
        0xED 0x44,
        [4,4],
        neg()
    ]

    // CCF
    [
        0x3F,
        [4],
        ccf()
    ]

    // SCF
    [
        0x37,
        [4],
        scf()
    ]

    // NOP
    [
        0,
        [4],
        nop()
    ]

    // HALT
    [
        0x76,
        [4],
        halt()
    ]


    // DI
    [
        0xF3,
        [4],
        di()
    ]

    // EI
    [
        0xFB,
        [4],
        ei()
    ]

    // IM 0
    [
        0xED 0x46,
        [4,4],
        im0()
    ]

    // IM 1
    [
        0xED 0x56,
        [4,4],
        im1()
    ]

    // IM 2
    [
        0xED 0x5E,
        [4,4],
        im2()
    ]

    // 16-Bit Arithmetic Group
    //////////////////////////

    // ADD HL, ss
    [
        ss, (x << 4 | 9),
        [4,4,3],
        add16(HL, x)
    ]

    // ADC HL, ss
    [
        0xED ss, (x << 4 | 0x4A),
        [4,4,4,3],
        adc16(HL, x)
    ]

    // SBC HL, ss
    [
        0xED ss, (x << 4 | 0x42),
        [4,4,4,3],
        sbc16(HL, x)
    ]

    // ADD IX, ss
    [
        0xDD pp, (x << 4 | 9),
        [4,4,4,3],
        add16(IX, x)
    ]

    // ADD IY, ss
    [
        0xFD qq2, (x << 4 | 9),
        [4,4,4,3],
        add16(IY, x)
    ]

    // INC ss
    [
        ss, (x << 4 | 3),
        [6],
        inc16(x)
    ]

    // INC IX
    [
        0xDD 0x23,
        [4,6],
        inc16(IX)
    ]

    // INC IY
    [
        0xFD 0x23,
        [4,6],
        inc16(IY)
    ]

    // DEC ss
    [
        ss, (x << 4 | 0x0B),
        [6],
        dec16(x)
    ]

    // DEC IX
    [
        0xDD 0x2B,
        [4,6],
        dec16(IX)
    ]

    // DEC IY
    [
        0xFD 0x2B,
        [4,6],
        dec16(IY)
    ]

    //// Rotate and Shift Group
    ///////////////////////////

    // RLCA
    [
        0x07,
        [4],
        rlca()
    ]

    // RLC r
    [
        0xCB r, (x),
        [4,4],
        rlc(x)
    ]

    // RLC (HL)
    [
        0xCB 0x06,
        [4,4,4,3],
        rlc(Address(HL))
    ]

    // RLC (IX+d)
    [
        0xDD 0xCB 0x06,
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, rlc(@))
    ]

    // RLC (IY+d)
    [
        0xFD 0xCB 0x06,
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, rlc(@))
    ]


    // RLC (IX+d), r
    [
        0xDD 0xCB r, (x),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, rlc_store(@, x))
    ]

    // RLC (IY+d), r
    [
        0xFD 0xCB r, (x),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, rlc_store(@, x))
    ]

    // RLA
    [
        0x17,
        [4],
        rla()
    ]

    // RL r
    [
        0xCB r, (x | 0x10),
        [4,4],
        rl(x)
    ]

    // RL (HL)
    [
        0xCB 0x16,
        [4,4,4,3],
        rl(Address(HL))
    ]

    // RL (IX+d)
    [
        0xDD 0xCB 0x16,
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, rl(@))
    ]

    // RL (IY+d)
    [
        0xFD 0xCB 0x16,
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, rl(@))
    ]

    // RL (IX+d), r
    [
        0xDD 0xCB r, (x | 0x10),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, rl_store(@, x))
    ]

    // RL (IY+d), r
    [
        0xDD 0xCB r, (x | 0x10),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, rl_store(@, x))
    ]

    // RRCA
    [
        0x0F,
        [4],
        rrca()
    ]

    // RRC r
    [
        0xCB r, (x | 8),
        [4,4],
        rrc(x)
    ]

    // RRC (HL)
    [
        0xCB 0x0E,
        [4,4,4,3],
        rrc(Address(HL))
    ]

    // RRC (IX+d)
    [
        0xDD 0xCB 0x0E,
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, rrc(@))
    ]

    // RRC (IY+d)
    [
        0xFD 0xCB 0x0E,
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, rrc(@))
    ]

    // RRC (IX+d), r
    [
        0xDD 0xCB r, (x | 8),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, rrc_store(@, x))
    ]

    // RRC (IY+d), r
    [
        0xFD 0xCB r, (x | 8),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, rrc_store(@, x))
    ]

    // RRA
    [
        0x1F,
        [4],
        rra()
    ]

    // RR r
    [
        0xCB r, (x | 0x18),
        [4,4],
        rr(x)
    ]

    // RR (HL)
    [
        0xCB 0x1E,
        [4,4,4,3],
        rr(Address(HL))
    ]

    // RR (IX+d)
    [
        0xDD 0xCB 0x1E,
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, rr(@))
    ]

    // RR (IY+d)
    [
        0xFD 0xCB 0x1E,
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, rr(@))
    ]

    // RR (IX+d), r
    [
        0xDD 0xCB r, (x | 0x18),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, rr_store(@, x))
    ]

    // RR (IY+d), r
    [
        0xFD 0xCB r, (x | 0x18),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, rr_store(@, x))
    ]

    // SLA r
    [
        0xCB r, (x | 0x20),
        [4,4],
        sla(x)
    ]

    // SLA (HL)
    [
        0xCB 0x26,
        [4,4,4,3],
        sla(Address(HL))
    ]

    // SLA (IX+d)
    [
        0xDD 0xCB 0x26,
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, sla(@))
    ]

    // SLA (IY+d)
    [
        0xFD 0xCB 0x26,
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, sla(@))
    ]

    // SLA (IX+d), r
    [
        0xDD 0xCB r, (x | 0x20),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, sla_store(@, x))
    ]

    // SLA (IY+d), r
    [
        0xFD 0xCB r, (x | 0x20),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, sla_store(@, x))
    ]

    // SLL r
    [
        0xCB r, (x | 0x30),
        [4,4],
        sll(x)
    ]

    // SLL (HL)
    [
        0xCB 0x36,
        [4,4,3,3],
        sll(Address(HL))
    ]

    // SLL (IX+d)
    [
        0xDD 0xCB 0x36,
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, sll(@))
    ]

    // SLL (IY+d)
    [
        0xFD 0xCB 0x36,
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, sll(@))
    ]

    // SLL (IX+d), r
    [
        0xDD 0xCB r, (x | 0x30),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, sll_store(@, x))
    ]

    // SLL (IY+d), r
    [
        0xFD 0xCB r, (x | 0x30),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, sll_store(@, x))
    ]

    // SRA r
    [
        0xCB r, (x | 0x28),
        [4,4],
        sra(x)
    ]

    // SRA (HL)
    [
        0xCB 0x2E,
        [4,4,4,3],
        sra(Address(HL))
    ]

    // SRA (IX+d)
    [
        0xDD 0xCB 0x2E,
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, sra(@))
    ]

    // SRA (IY+d)
    [
        0xFD 0xCB 0x2E,
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, sra(@))
    ]

    // SRA (IX+d), r
    [
        0xDD 0xCB r, (x | 0x28),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, sra_store(@, x))
    ]

    // SRA (IY+d), r
    [
        0xFD 0xCB r, (x | 0x28),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, sra_store(@, x))
    ]

    // SRL r
    [
        0xCB r, (x | 0x38),
        [4,4],
        srl(x)
    ]

    // SRL (HL)
    [
        0xCB 0x3E,
        [4,4,4,3],
        srl(Address(HL))
    ]

    // SRL (IX+d)
    [
        0xDD 0xCB 0x3E,
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, srl(@))
    ]

    // SRL (IY+d)
    [
        0xFD 0xCB 0x3E,
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, srl(@))
    ]

    // SRL (IX+d), r
    [
        0xDD 0xCB r, (x | 0x38),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, srl_store(@, x))
    ]

    // SRL (IY+d), r
    [
        0xFD 0xCB r, (x | 0x38),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, srl_store(@, x))
    ]

    // RLD
    [
        0xED 0x6F,
        [4,4,3,4,3],
        rld()
    ]

    // RLD
    [
        0xED 0x67,
        [4,4,3,4,3],
        rrd()
    ]

    //// Bit Set, Reset, and Test Group
    ///////////////////////////////////

    // BIT b, r
    [
        0xCB b r, (0x40 | x << 3 | y),
        [4,4],
        bit(x, y)
    ]

    // BIT b, (HL)
    [
        0xCB b, (0x46 | x << 3),
        [4,4,4],
        bit(x, Address(HL))
    ]

    // BIT b, (IX+d)
    [
        0xDD 0xCB b r, (0x46 | x << 3),
        [4,4,3,5,4],
        shift_d!(IX, (-2i16) as u16, 0, bit(x, @))
    ]

    // BIT b, (IX+d)
    // These are undocumented opcodes for BIT b, (IX+d)
    // see Young, 3.5.
    [
        0xDD 0xCB b r, (0x40 | x << 3 | y),
        [4,4,3,5,4],
        shift_d!(IX, (-2i16) as u16, 0, bit(x, @))
    ]

    // BIT b, (IY+d)
    [
        0xFD 0xCB b r, (0x46 | x << 3),
        [4,4,3,5,4],
        shift_d!(IY, (-2i16) as u16, 0, bit(x, @))
    ]

    // BIT b, (IY+d)
    // These are undocumented opcodes for BIT b, (IY+d)
    // see Young, 3.5.
    [
        0xFD 0xCB b r, (0x40 | x << 3 | y),
        [4,4,3,5,4],
        shift_d!(IY, (-2i16) as u16, 0, bit(x, @))
    ]

    // SET b, r
    [
        0xCB b r, (0xC0 | x << 3 | y),
        [4,4],
        set(x, y)
    ]

    // SET b, (HL)
    [
        0xCB b, (0xC6 | x << 3),
        [4,4,4,3],
        set(x, Address(HL))
    ]

    // SET b, (IX+d)
    [
        0xDD 0xCB b, (0xC6 | x << 3),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, set(x, @))
    ]

    // Undocumented -- see Young, 3.5.
    // SET b, (IX+d), r
    [
        0xDD 0xCB b r, (0xC0 | x << 3 | y),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, set_store(x, @, y))
    ]

    // SET b, (IY+d)
    [
        0xFD 0xCB b, (0xC6 | x << 3),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, set(x, @))
    ]

    // Undocumented -- see Young, 3.5.
    // SET b, (IX+d), r
    [
        0xFD 0xCB b r, (0xC0 | x << 3 | y),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, set_store(x, @, y))
    ]

    // RES b, r
    [
        0xCB b r, (0x80 | x << 3 | y),
        [4,4],
        res(x, y)
    ]
    
    // RES b, (HL)
    [
        0xCB b, (0x86 | x << 3),
        [4,4,3,5,4,3],
        res(x, Address(HL))
    ]

    // RES b, (IX+d)
    [
        0xDD 0xCB b, (0x86 | x << 3),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, res(x, @))
    ]

    // Undocumented -- see Young, 3.5.
    // RES b, (IX+d), r
    [
        0xDD 0xCB b r, (0x80 | x << 3 | y),
        [4,4,3,5,4,3],
        shift_d!(IX, (-2i16) as u16, 0, res_store(x, @, y))
    ]

    // RES b, (IY+d)
    [
        0xFD 0xCB b, (0x86 | x << 3),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, res(x, @))
    ]

    // Undocumented -- see Young, 3.5.
    // RES b, (IX+d), r
    [
        0xFD 0xCB b r, (0x80 | x << 3 | y),
        [4,4,3,5,4,3],
        shift_d!(IY, (-2i16) as u16, 0, res_store(x, @, y))
    ]

    //// Jump Group
    ///////////////

    // JP nn
    [
        0xC3,
        [4,3,3],
        read_nn!(jp(@))
    ]

    // JP cc, nn
    [
        b, (x << 3 | 0xC2),
        [4,3,3],
        jp_cc(x)
    ]

    // JR e
    [
        0x18,
        [4,3,5],
        jr()
    ]

    // JR C, e
    [
        0x38,
        [],
        jr_c()
    ]

    // JR NC, e
    [
        0x30,
        [],
        jr_nc()
    ]

    // JR Z, e
    [
        0x28,
        [],
        jr_z()
    ]

    // JR NZ, e
    [
        0x20,
        [],
        jr_nz()
    ]

    // note that although the mnemonic says (HL), there is no memory access.
    // Ditto for the next two
    // JP (HL)
    [
        0xE9,
        [4],
        jp(HL)
    ]

    // JP (IX)
    [
        0xDD 0xE9,
        [4,4],
        jp(IX)
    ]

    // JP (IY)
    [
        0xFD 0xE9,
        [4,4],
        jp(IY)
    ]

    // DJNZ, e
    [
        0x10,
        [],
        djnz()
    ]

    //// Call and Return Group

    // CALL nn
    [
        0xCD,
        [4,3,4,3,3],
        read_nn!(call_nn(@))
    ]

    // CALL cc, nn
    [
        b, (x << 3 | 0xC4),
        [],
        read_nn!(call_cc_nn(x, @))
    ]

    // RET
    [
        0xC9,
        [4,3,3],
        ret()
    ]

    // RET cc
    [
        b, (x << 3 | 0xC0),
        [],
        ret_cc(x)
    ]

    // RETI
    [
        0xED 0x4D,
        [4,4,3,3],
        reti()
    ]

    // RETN
    [
        0xED 0x45,
        [4,4,3,3],
        retn()
    ]

    // RST p
    [
        b, (x << 3 | 0xC7),
        [5,3,3],
        rst(x)
    ]

    //// Input and Output Group
    ///////////////////////////

    // IN A, (n)
    [
        0xDB,
        [4,3,4],
        read_n!(in_a(@))
    ]

    // IN r, (C)
    [
        0xED r, (0x40 | x << 3),
        [4,4,4],
        in_c(x)
    ]

    // IN F, (C)
    [
        0xED 0xB0,
        [4,4,4],
        in0()
    ]

    // INI
    [
        0xED 0xA2,
        [4,5,3,4],
        ini()
    ]

    // INIR
    [
        0xED 0xB2,
        [],
        inir()
    ]

    // IND
    [
        0xED 0xAA,
        [4,5,3,4],
        ind()
    ]

    // INDR
    [
        0xED 0xBA,
        [],
        indr()
    ]

    // OUT (n), A
    [
        0xD3,
        [4,3,4],
        read_n!(out(@))
    ]

    // OUT (C), r
    [
        0xED r, (0x41 | x << 3),
        [4,4,4],
        out_c(x)
    ]

    // OUTI
    [
        0xED 0xA3,
        [4,5,3,4],
        outi()
    ]

    // OTIR
    [
        0xED 0xB3,
        [],
        otir()
    ]

    // OUTD
    [
        0xED 0xAB,
        [4,5,3,4],
        outd()
    ]

    // OTDR
    [
        0xED 0xBB,
        [],
        otdr()
    ]
}
