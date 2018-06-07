macro_rules! attalus_z80_ed {
    ($mac: ident) => {
        attalus_z80_ed! {@ $mac
(
[ 0x00 ; x00 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x01 ; x01 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x02 ; x02 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x03 ; x03 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x04 ; x04 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x05 ; x05 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x06 ; x06 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x07 ; x07 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x08 ; x08 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x09 ; x09 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x0A ; x0A ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x0B ; x0B ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x0C ; x0C ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x0D ; x0D ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x0E ; x0E ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x0F ; x0F ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x10 ; x10 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x11 ; x11 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x12 ; x12 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x13 ; x13 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x14 ; x14 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x15 ; x15 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x16 ; x16 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x17 ; x17 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x18 ; x18 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x19 ; x19 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x1A ; x1A ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x1B ; x1B ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x1C ; x1C ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x1D ; x1D ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x1E ; x1E ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x1F ; x1F ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x20 ; x20 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x21 ; x21 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x22 ; x22 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x23 ; x23 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x24 ; x24 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x25 ; x25 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x26 ; x26 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x27 ; x27 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x28 ; x28 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x29 ; x29 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x2A ; x2A ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x2B ; x2B ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x2C ; x2C ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x2D ; x2D ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x2E ; x2E ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x2F ; x2F ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x30 ; x30 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x31 ; x31 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x32 ; x32 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x33 ; x33 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x34 ; x34 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x35 ; x35 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x36 ; x36 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x37 ; x37 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x38 ; x38 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x39 ; x39 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x3A ; x3A ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x3B ; x3B ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x3C ; x3C ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x3D ; x3D ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x3E ; x3E ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x3F ; x3F ; ; nop () ;  8 ; undoc ; z80   ]

[ 0x40 ; x40 ;    ; in_c  (B, C)     ; 12 ; doc   ; z80  ]
[ 0x41 ; x41 ;    ; out_c (C, B)     ; 12 ; doc   ; z80  ]
[ 0x42 ; x42 ;    ; sbc16 (HL, BC)   ; 15 ; doc   ; z80  ]
[ 0x43 ; x43 ; nn ; ld16  ((nn), BC) ; 20 ; doc   ; z80  ]
[ 0x44 ; x44 ;    ; neg   ()         ;  8 ; doc   ; z80  ]
[ 0x45 ; x45 ;    ; retn  ()         ; 14 ; doc   ; z80  ]
[ 0x46 ; x46 ;    ; im    (0)        ;  8 ; doc   ; z80  ]
[ 0x47 ; x47 ;    ; ld    (I, A)     ;  9 ; doc   ; z80  ]
[ 0x48 ; x48 ;    ; in_c  (C, C)     ; 12 ; doc   ; z80  ]
[ 0x49 ; x49 ;    ; out_c (C, C)     ; 12 ; doc   ; z80  ]
[ 0x4A ; x4A ;    ; adc16 (HL, BC)   ; 15 ; doc   ; z80  ]
[ 0x4B ; x4B ; nn ; ld16  (BC, (nn)) ; 20 ; doc   ; z80  ]
[ 0x4C ; x4C ;    ; neg   ()         ;  8 ; undoc ; z80  ]
[ 0x4D ; x4D ;    ; reti  ()         ; 14 ; doc   ; z80  ]
[ 0x4E ; x4E ;    ; im    (0)        ;  8 ; undoc ; z80  ]
[ 0x4F ; x4F ;    ; ld    (R, A)     ;  9 ; doc   ; z80  ]
[ 0x50 ; x50 ;    ; in_c  (D, C)     ; 12 ; doc   ; z80  ]
[ 0x51 ; x51 ;    ; out_c (C, D)     ; 12 ; doc   ; z80  ]
[ 0x52 ; x52 ;    ; sbc16 (HL, DE)   ; 15 ; doc   ; z80  ]
[ 0x53 ; x53 ; nn ; ld16  ((nn), DE) ; 20 ; doc   ; z80  ]
[ 0x54 ; x54 ;    ; neg   ()         ;  8 ; undoc ; z80  ]
[ 0x55 ; x55 ;    ; retn  ()         ; 14 ; undoc ; z80  ]
[ 0x56 ; x56 ;    ; im    (1)        ;  8 ; doc   ; z80  ]
[ 0x57 ; x57 ;    ; ld_ir (A, I)     ;  9 ; doc   ; z80  ]
[ 0x58 ; x58 ;    ; in_c  (E, C)     ; 12 ; doc   ; z80  ]
[ 0x59 ; x59 ;    ; out_c (C, E)     ; 12 ; doc   ; z80  ]
[ 0x5A ; x5A ;    ; adc16 (HL, DE)   ; 15 ; doc   ; z80  ]
[ 0x5B ; x5B ; nn ; ld16  (DE, (nn)) ; 20 ; doc   ; z80  ]
[ 0x5C ; x5C ;    ; neg   ()         ;  8 ; undoc ; z80  ]
[ 0x5D ; x5D ;    ; retn  ()         ; 14 ; undoc ; z80  ]
[ 0x5E ; x5E ;    ; im    (2)        ;  8 ; doc   ; z80  ]
[ 0x5F ; x5F ;    ; ld_ir (A, R)     ;  9 ; doc   ; z80  ]
[ 0x60 ; x60 ;    ; in_c  (H, C)     ; 12 ; doc   ; z80  ]
[ 0x61 ; x61 ;    ; out_c (C, H)     ; 12 ; doc   ; z80  ]
[ 0x62 ; x62 ;    ; sbc16 (HL, HL)   ; 15 ; doc   ; z80  ]
[ 0x63 ; x63 ; nn ; ld16  ((nn), HL) ; 20 ; doc   ; z80  ]
[ 0x64 ; x64 ;    ; neg   ()         ;  8 ; undoc ; z80  ]
[ 0x65 ; x65 ;    ; retn  ()         ; 14 ; undoc ; z80  ]
[ 0x66 ; x66 ;    ; im    (0)        ;  8 ; undoc ; z80  ]
[ 0x67 ; x67 ;    ; rrd   ()         ; 18 ; doc   ; z80  ]
[ 0x68 ; x68 ;    ; in_c  (L, C)     ; 12 ; doc   ; z80  ]
[ 0x69 ; x69 ;    ; out_c (C, L)     ; 12 ; doc   ; z80  ]
[ 0x6A ; x6A ;    ; adc16 (HL, HL)   ; 15 ; doc   ; z80  ]
[ 0x6B ; x6B ; nn ; ld16  (HL, (nn)) ; 20 ; doc   ; z80  ]
[ 0x6C ; x6C ;    ; neg   ()         ;  8 ; undoc ; z80  ]
[ 0x6D ; x6D ;    ; retn  ()         ; 14 ; undoc ; z80  ]
[ 0x6E ; x6E ;    ; im    (0)        ;  8 ; undoc ; z80  ]
[ 0x6F ; x6F ;    ; rld   ()         ; 18 ; doc   ; z80  ]
[ 0x70 ; x70 ;    ; in_f  (C)        ; 12 ; undoc ; z80  ]
[ 0x71 ; x71 ;    ; out_c (C, 0)     ; 12 ; undoc ; z80  ]
[ 0x72 ; x72 ;    ; sbc16 (HL, SP)   ; 15 ; doc   ; z80  ]
[ 0x73 ; x73 ; nn ; ld16  ((nn), SP) ; 20 ; doc   ; z80  ]
[ 0x74 ; x74 ;    ; neg   ()         ;  8 ; undoc ; z80  ]
[ 0x75 ; x75 ;    ; retn  ()         ; 14 ; undoc ; z80  ]
[ 0x76 ; x76 ;    ; im    (1)        ;  8 ; undoc ; z80  ]

[ 0x77 ; x77 ; ; nop () ;  8 ; undoc ; z80   ]

[ 0x78 ; x78 ;    ; in_c  (A, C)     ; 12 ; doc   ; z80  ]
[ 0x79 ; x79 ;    ; out_c (C, A)     ; 12 ; doc   ; z80  ]
[ 0x7A ; x7A ;    ; adc16 (HL, SP)   ; 15 ; doc   ; z80  ]
[ 0x7B ; x7B ; nn ; ld16  (SP, (nn)) ; 20 ; doc   ; z80  ]
[ 0x7C ; x7C ;    ; neg   ()         ;  8 ; undoc ; z80  ]
[ 0x7D ; x7D ;    ; retn  ()         ; 14 ; undoc ; z80  ]
[ 0x7E ; x7E ;    ; im    (2)        ;  8 ; undoc ; z80  ]

[ 0x7F ; x7F ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x80 ; x80 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x81 ; x81 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x82 ; x82 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x83 ; x83 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x84 ; x84 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x85 ; x85 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x86 ; x86 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x87 ; x87 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x88 ; x88 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x89 ; x89 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x8A ; x8A ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x8B ; x8B ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x8C ; x8C ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x8D ; x8D ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x8E ; x8E ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x8F ; x8F ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x90 ; x90 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x91 ; x91 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x92 ; x92 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x93 ; x93 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x94 ; x94 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x95 ; x95 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x96 ; x96 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x97 ; x97 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x98 ; x98 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x99 ; x99 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x9A ; x9A ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x9B ; x9B ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x9C ; x9C ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x9D ; x9D ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x9E ; x9E ; ; nop () ;  8 ; undoc ; z80   ]
[ 0x9F ; x9F ; ; nop () ;  8 ; undoc ; z80   ]

[ 0xA0 ; xA0 ; ; ldi  () ; 16 ; doc   ; z80  ]
[ 0xA1 ; xA1 ; ; cpi  () ; 16 ; doc   ; z80  ]
[ 0xA2 ; xA2 ; ; ini  () ; 16 ; doc   ; z80  ]
[ 0xA3 ; xA3 ; ; outi () ; 16 ; doc   ; z80  ]

[ 0xA4 ; xA4 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xA5 ; xA5 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xA6 ; xA6 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xA7 ; xA7 ; ; nop () ;  8 ; undoc ; z80   ]

[ 0xA8 ; xA8 ; ; ldd  () ; 16 ; doc   ; z80  ]
[ 0xA9 ; xA9 ; ; cpd  () ; 16 ; doc   ; z80  ]
[ 0xAA ; xAA ; ; ind  () ; 16 ; doc   ; z80  ]
[ 0xAB ; xAB ; ; outd () ; 16 ; doc   ; z80  ]

[ 0xAC ; xAC ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xAD ; xAD ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xAE ; xAE ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xAF ; xAF ; ; nop () ;  8 ; undoc ; z80   ]

[ 0xB0 ; xB0 ; ; ldir () ; xx ; doc   ; z80  ]
[ 0xB1 ; xB1 ; ; cpir () ; xx ; doc   ; z80  ]
[ 0xB2 ; xB2 ; ; inir () ; xx ; doc   ; z80  ]
[ 0xB3 ; xB3 ; ; otir () ; xx ; doc   ; z80  ]

[ 0xB4 ; xB4 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xB5 ; xB5 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xB6 ; xB6 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xB7 ; xB7 ; ; nop () ;  8 ; undoc ; z80   ]

[ 0xB8 ; xB8 ; ; lddr () ; xx ; doc   ; z80  ]
[ 0xB9 ; xB9 ; ; cpdr () ; xx ; doc   ; z80  ]
[ 0xBA ; xBA ; ; indr () ; xx ; doc   ; z80  ]
[ 0xBB ; xBB ; ; otdr () ; xx ; doc   ; z80  ]

[ 0xBC ; xBC ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xBD ; xBD ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xBE ; xBE ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xBF ; xBF ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC0 ; xC0 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC1 ; xC1 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC2 ; xC2 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC3 ; xC3 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC4 ; xC4 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC5 ; xC5 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC6 ; xC6 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC7 ; xC7 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC8 ; xC8 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xC9 ; xC9 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xCA ; xCA ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xCB ; xCB ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xCC ; xCC ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xCD ; xCD ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xCE ; xCE ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xCF ; xCF ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD0 ; xD0 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD1 ; xD1 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD2 ; xD2 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD3 ; xD3 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD4 ; xD4 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD5 ; xD5 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD6 ; xD6 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD7 ; xD7 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD8 ; xD8 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xD9 ; xD9 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xDA ; xDA ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xDB ; xDB ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xDC ; xDC ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xDD ; xDD ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xDE ; xDE ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xDF ; xDF ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE0 ; xE0 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE1 ; xE1 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE2 ; xE2 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE3 ; xE3 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE4 ; xE4 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE5 ; xE5 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE6 ; xE6 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE7 ; xE7 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE8 ; xE8 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xE9 ; xE9 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xEA ; xEA ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xEB ; xEB ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xEC ; xEC ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xED ; xED ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xEE ; xEE ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xEF ; xEF ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF0 ; xF0 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF1 ; xF1 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF2 ; xF2 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF3 ; xF3 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF4 ; xF4 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF5 ; xF5 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF6 ; xF6 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF7 ; xF7 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF8 ; xF8 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xF9 ; xF9 ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xFA ; xFA ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xFB ; xFB ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xFC ; xFC ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xFD ; xFD ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xFE ; xFE ; ; nop () ;  8 ; undoc ; z80   ]
[ 0xFF ; xFF ; ; nop () ;  8 ; undoc ; z80   ]
)
        }
    };
    (@ $mac: ident ($($inst: tt)*)) => {
        $(
            $mac!{$inst}
         )*
    };
}