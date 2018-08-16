macro_rules! euphrates_z80_dd {
    ($mac: ident) => {
        euphrates_z80_dd! {@ $mac
(
[ 0x00 ; x00 ;    ; nop    ()         ;  4 ; undoc ; z80 ; no  ]
[ 0x01 ; x01 ; nn ; ld16   (BC, nn)   ; 10 ; undoc ; z80 ; mem ]
[ 0x02 ; x02 ;    ; ld     ((BC), A)  ;  7 ; undoc ; z80 ; mem ]
[ 0x03 ; x03 ;    ; inc16  (BC)       ;  6 ; undoc ; z80 ; no  ]
[ 0x04 ; x04 ;    ; inc    (B)        ;  4 ; undoc ; z80 ; mem ]
[ 0x05 ; x05 ;    ; dec    (B)        ;  4 ; undoc ; z80 ; mem ]
[ 0x06 ; x06 ; n  ; ld     (B, n)     ;  7 ; undoc ; z80 ; mem ]
[ 0x07 ; x07 ;    ; rlca   ()         ;  4 ; undoc ; z80 ; mem ]
[ 0x08 ; x08 ;    ; ex     (AF, AF0)  ;  4 ; undoc ; z80 ; mem ]

[ 0x09 ; x09 ;      ; add16 (IX, BC)    ; 11 ; doc   ; z80 ; no  ]

[ 0x0A ; x0A ;    ; ld     (A, (BC))  ;  7 ; undoc ; z80 ; mem  ]
[ 0x0B ; x0B ;    ; dec16  (BC)       ;  6 ; undoc ; z80 ; no   ]
[ 0x0C ; x0C ;    ; inc    (C)        ;  4 ; undoc ; z80 ; mem  ]
[ 0x0D ; x0D ;    ; dec    (C)        ;  4 ; undoc ; z80 ; mem  ]
[ 0x0E ; x0E ; n  ; ld     (C, n)     ;  7 ; undoc ; z80 ; mem  ]
[ 0x0F ; x0F ;    ; rrca   ()         ;  4 ; undoc ; z80 ; mem  ]
[ 0x10 ; x10 ; e  ; djnz   (e)        ;  8 ; undoc ; z80 ; no   ]
[ 0x11 ; x11 ; nn ; ld16   (DE, nn)   ; 10 ; undoc ; z80 ; mem  ]
[ 0x12 ; x12 ;    ; ld     ((DE), A)  ;  7 ; undoc ; z80 ; mem  ]
[ 0x13 ; x13 ;    ; inc16  (DE)       ;  6 ; undoc ; z80 ; no   ]
[ 0x14 ; x14 ;    ; inc    (D)        ;  4 ; undoc ; z80 ; mem  ]
[ 0x15 ; x15 ;    ; dec    (D)        ;  4 ; undoc ; z80 ; mem  ]
[ 0x16 ; x16 ; n  ; ld     (D, n)     ;  7 ; undoc ; z80 ; mem  ]
[ 0x17 ; x17 ;    ; rla    ()         ;  4 ; undoc ; z80 ; mem  ]
[ 0x18 ; x18 ; e  ; jr     (e)        ; 12 ; undoc ; z80 ; no   ]

[ 0x19 ; x19 ;      ; add16 (IX, DE)    ; 11 ; doc   ; z80 ; no  ]

[ 0x1A ; x1A ;    ; ld     (A, (DE))  ;  7 ; undoc ; z80 ; mem  ]
[ 0x1B ; x1B ;    ; dec16  (DE)       ;  6 ; undoc ; z80 ; no   ]
[ 0x1C ; x1C ;    ; inc    (E)        ;  4 ; undoc ; z80 ; mem  ]
[ 0x1D ; x1D ;    ; dec    (E)        ;  4 ; undoc ; z80 ; mem  ]
[ 0x1E ; x1E ; n  ; ld     (E, n)     ;  7 ; undoc ; z80 ; mem  ]
[ 0x1F ; x1F ;    ; rra    ()         ;  4 ; undoc ; z80 ; mem  ]
[ 0x20 ; x20 ; e  ; jrcc   (NZcc, e)  ; xx ; undoc ; z80 ; no   ]

[ 0x21 ; x21 ; nn   ; ld16  (IX, nn)    ; 10 ; doc   ; z80 ; mem  ]
[ 0x22 ; x22 ; nn   ; ld16  ((nn), IX)  ; 16 ; doc   ; z80 ; mem  ]
[ 0x23 ; x23 ;      ; inc16 (IX)        ;  6 ; doc   ; z80 ; no   ]
[ 0x24 ; x24 ;      ; inc   (IXH)       ;  4 ; undoc ; z80 ; mem  ]
[ 0x25 ; x25 ;      ; dec   (IXH)       ;  4 ; undoc ; z80 ; mem  ]
[ 0x26 ; x26 ; n    ; ld    (IXH, n)    ;  7 ; undoc ; z80 ; mem  ]

[ 0x27 ; x27 ;    ; daa    ()         ;  4 ; undoc ; z80 ; no  ]
[ 0x28 ; x28 ; e  ; jrcc   (Zcc, e)   ; xx ; undoc ; z80 ; no  ]

[ 0x29 ; x29 ;      ; add16 (IX, IX)    ; 11 ; doc   ; z80 ; no  ]
[ 0x2A ; x2A ; nn   ; ld16  (IX, (nn))  ; 16 ; doc   ; z80 ; mem ]
[ 0x2B ; x2B ;      ; dec16 (IX)        ;  6 ; doc   ; z80 ; no  ]
[ 0x2C ; x2C ;      ; inc   (IXL)       ;  4 ; undoc ; z80 ; mem ]
[ 0x2D ; x2D ;      ; dec   (IXL)       ;  4 ; undoc ; z80 ; mem ]
[ 0x2E ; x2E ; n    ; ld    (IXL, n)    ;  7 ; undoc ; z80 ; mem ]

[ 0x2F ; x2F ;    ; cpl    ()         ;  4 ; undoc ; z80 ; no    ]
[ 0x30 ; x30 ; e  ; jrcc   (NCcc, e)  ; xx ; undoc ; z80 ; no    ]
[ 0x31 ; x31 ; nn ; ld16   (SP, nn)   ; 10 ; undoc ; z80 ; mem   ]
[ 0x32 ; x32 ; nn ; ld     ((nn), A)  ; 13 ; undoc ; z80 ; mem   ]
[ 0x33 ; x33 ;    ; inc16  (SP)       ;  6 ; undoc ; z80 ; no    ]

[ 0x34 ; x34 ; d    ; inc   ((IX+d))    ; 19 ; doc   ; z80 ; mem   ]
[ 0x35 ; x35 ; d    ; dec   ((IX+d))    ; 19 ; doc   ; z80 ; mem   ]
[ 0x36 ; x36 ; d, n ; ld    ((IX+d), n) ; 15 ; doc   ; z80 ; mem   ]

[ 0x37 ; x37 ;    ; scf    ()         ;  4 ; undoc ; z80 ; no    ]
[ 0x38 ; x38 ; e  ; jrcc   (Ccc, e)   ; xx ; undoc ; z80 ; no    ]

[ 0x39 ; x39 ;      ; add16 (IX, SP)    ; 11 ; doc   ; z80 ; no   ]

[ 0x3A ; x3A ; nn ; ld     (A, (nn))  ; 13 ; undoc ; z80 ; mem   ]
[ 0x3B ; x3B ;    ; dec16  (SP)       ;  6 ; undoc ; z80 ; no    ]
[ 0x3C ; x3C ;    ; inc    (A)        ;  4 ; undoc ; z80 ; mem   ]
[ 0x3D ; x3D ;    ; dec    (A)        ;  4 ; undoc ; z80 ; mem   ]
[ 0x3E ; x3E ; n  ; ld     (A, n)     ;  7 ; undoc ; z80 ; mem   ]
[ 0x3F ; x3F ;    ; ccf    ()         ;  4 ; undoc ; z80 ; no    ]
[ 0x40 ; x40 ;    ; ld     (B, B)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x41 ; x41 ;    ; ld     (B, C)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x42 ; x42 ;    ; ld     (B, D)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x43 ; x43 ;    ; ld     (B, E)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x44 ; x44 ;      ; ld    (B, IXH)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x45 ; x45 ;      ; ld    (B, IXL)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x46 ; x46 ; d    ; ld    (B, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]

[ 0x47 ; x47 ;    ; ld     (B, A)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x48 ; x48 ;    ; ld     (C, B)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x49 ; x49 ;    ; ld     (C, C)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x4A ; x4A ;    ; ld     (C, D)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x4B ; x4B ;    ; ld     (C, E)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x4C ; x4C ;      ; ld    (C, IXH)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x4D ; x4D ;      ; ld    (C, IXL)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x4E ; x4E ; d    ; ld    (C, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]

[ 0x4F ; x4F ;    ; ld     (C, A)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x50 ; x50 ;    ; ld     (D, B)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x51 ; x51 ;    ; ld     (D, C)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x52 ; x52 ;    ; ld     (D, D)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x53 ; x53 ;    ; ld     (D, E)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x54 ; x54 ;      ; ld    (D, IXH)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x55 ; x55 ;      ; ld    (D, IXL)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x56 ; x56 ; d    ; ld    (D, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]

[ 0x57 ; x57 ;    ; ld     (D, A)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x58 ; x58 ;    ; ld     (E, B)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x59 ; x59 ;    ; ld     (E, C)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x5A ; x5A ;    ; ld     (E, D)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x5B ; x5B ;    ; ld     (E, E)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x5C ; x5C ;      ; ld    (E, IXH)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x5D ; x5D ;      ; ld    (E, IXL)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x5E ; x5E ; d    ; ld    (E, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]

[ 0x5F ; x5F ;    ; ld     (E, A)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x60 ; x60 ;      ; ld    (IXH, B)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x61 ; x61 ;      ; ld    (IXH, C)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x62 ; x62 ;      ; ld    (IXH, D)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x63 ; x63 ;      ; ld    (IXH, E)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x64 ; x64 ;      ; ld    (IXH, IXH)  ;  4 ; undoc ; z80 ; mem   ]
[ 0x65 ; x65 ;      ; ld    (IXH, IXL)  ;  4 ; undoc ; z80 ; mem   ]
[ 0x66 ; x66 ; d    ; ld    (H, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]
[ 0x67 ; x67 ;      ; ld    (IXH, A)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x68 ; x68 ;      ; ld    (IXL, B)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x69 ; x69 ;      ; ld    (IXL, C)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x6A ; x6A ;      ; ld    (IXL, D)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x6B ; x6B ;      ; ld    (IXL, E)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x6C ; x6C ;      ; ld    (IXL, IXH)  ;  4 ; undoc ; z80 ; mem   ]
[ 0x6D ; x6D ;      ; ld    (IXL, IXL)  ;  4 ; undoc ; z80 ; mem   ]
[ 0x6E ; x6E ; d    ; ld    (L, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]
[ 0x6F ; x6F ;      ; ld    (IXL, A)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x70 ; x70 ; d    ; ld    ((IX+d), B) ; 15 ; doc   ; z80 ; mem   ]
[ 0x71 ; x71 ; d    ; ld    ((IX+d), C) ; 15 ; doc   ; z80 ; mem   ]
[ 0x72 ; x72 ; d    ; ld    ((IX+d), D) ; 15 ; doc   ; z80 ; mem   ]
[ 0x73 ; x73 ; d    ; ld    ((IX+d), E) ; 15 ; doc   ; z80 ; mem   ]
[ 0x74 ; x74 ; d    ; ld    ((IX+d), H) ; 15 ; doc   ; z80 ; mem   ]
[ 0x75 ; x75 ; d    ; ld    ((IX+d), L) ; 15 ; doc   ; z80 ; mem   ]

[ 0x76 ; x76 ;    ; halt   ()         ;  4 ; undoc ; z80 ; no   ]

[ 0x77 ; x77 ; d    ; ld    ((IX+d), A) ; 15 ; doc   ; z80 ; mem   ]

[ 0x78 ; x78 ;    ; ld     (A, B)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x79 ; x79 ;    ; ld     (A, C)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x7A ; x7A ;    ; ld     (A, D)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x7B ; x7B ;    ; ld     (A, E)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x7C ; x7C ;      ; ld    (A, IXH)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x7D ; x7D ;      ; ld    (A, IXL)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x7E ; x7E ; d    ; ld    (A, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]

[ 0x7F ; x7F ;    ; ld     (A, A)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x80 ; x80 ;    ; add    (A, B)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x81 ; x81 ;    ; add    (A, C)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x82 ; x82 ;    ; add    (A, D)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x83 ; x83 ;    ; add    (A, E)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x84 ; x84 ;      ; add   (A, IXH)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x85 ; x85 ;      ; add   (A, IXL)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x86 ; x86 ; d    ; add   (A, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]

[ 0x87 ; x87 ;    ; add    (A, A)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x88 ; x88 ;    ; adc    (A, B)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x89 ; x89 ;    ; adc    (A, C)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x8A ; x8A ;    ; adc    (A, D)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x8B ; x8B ;    ; adc    (A, E)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x8C ; x8C ;      ; adc   (A, IXH)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x8D ; x8D ;      ; adc   (A, IXL)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x8E ; x8E ; d    ; adc   (A, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]

[ 0x8F ; x8F ;    ; adc    (A, A)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x90 ; x90 ;    ; sub    (A, B)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x91 ; x91 ;    ; sub    (A, C)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x92 ; x92 ;    ; sub    (A, D)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x93 ; x93 ;    ; sub    (A, E)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x94 ; x94 ;      ; sub   (A, IXH)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x95 ; x95 ;      ; sub   (A, IXL)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x96 ; x96 ; d    ; sub   (A, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]

[ 0x97 ; x97 ;    ; sub    (A, A)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x98 ; x98 ;    ; sbc    (A, B)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x99 ; x99 ;    ; sbc    (A, C)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x9A ; x9A ;    ; sbc    (A, D)     ;  4 ; undoc ; z80 ; mem   ]
[ 0x9B ; x9B ;    ; sbc    (A, E)     ;  4 ; undoc ; z80 ; mem   ]

[ 0x9C ; x9C ;      ; sbc   (A, IXH)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x9D ; x9D ;      ; sbc   (A, IXL)    ;  4 ; undoc ; z80 ; mem   ]
[ 0x9E ; x9E ; d    ; sbc   (A, (IX+d)) ; 15 ; doc   ; z80 ; mem   ]

[ 0x9F ; x9F ;    ; sbc    (A, A)     ;  4 ; undoc ; z80 ; mem   ]
[ 0xA0 ; xA0 ;    ; and    (B)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xA1 ; xA1 ;    ; and    (C)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xA2 ; xA2 ;    ; and    (D)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xA3 ; xA3 ;    ; and    (E)        ;  4 ; undoc ; z80 ; mem   ]

[ 0xA4 ; xA4 ;      ; and   (IXH)       ;  4 ; undoc ; z80 ; mem   ]
[ 0xA5 ; xA5 ;      ; and   (IXL)       ;  4 ; undoc ; z80 ; mem   ]
[ 0xA6 ; xA6 ; d    ; and   ((IX+d))    ; 15 ; doc   ; z80 ; mem   ]

[ 0xA7 ; xA7 ;    ; and    (A)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xA8 ; xA8 ;    ; xor    (B)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xA9 ; xA9 ;    ; xor    (C)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xAA ; xAA ;    ; xor    (D)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xAB ; xAB ;    ; xor    (E)        ;  4 ; undoc ; z80 ; mem   ]

[ 0xAC ; xAC ;      ; xor   (IXH)       ;  4 ; undoc ; z80 ; mem   ]
[ 0xAD ; xAD ;      ; xor   (IXL)       ;  4 ; undoc ; z80 ; mem   ]
[ 0xAE ; xAE ; d    ; xor   ((IX+d))    ; 15 ; doc   ; z80 ; mem   ]

[ 0xAF ; xAF ;    ; xor    (A)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xB0 ; xB0 ;    ; or     (B)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xB1 ; xB1 ;    ; or     (C)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xB2 ; xB2 ;    ; or     (D)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xB3 ; xB3 ;    ; or     (E)        ;  4 ; undoc ; z80 ; mem   ]

[ 0xB4 ; xB4 ;      ; or    (IXH)       ;  4 ; undoc ; z80 ; mem   ]
[ 0xB5 ; xB5 ;      ; or    (IXL)       ;  4 ; undoc ; z80 ; mem   ]
[ 0xB6 ; xB6 ; d    ; or    ((IX+d))    ; 15 ; doc   ; z80 ; mem   ]

[ 0xB7 ; xB7 ;    ; or     (A)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xB8 ; xB8 ;    ; cp     (B)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xB9 ; xB9 ;    ; cp     (C)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xBA ; xBA ;    ; cp     (D)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xBB ; xBB ;    ; cp     (E)        ;  4 ; undoc ; z80 ; mem   ]

[ 0xBC ; xBC ;      ; cp    (IXH)       ;  4 ; undoc ; z80 ; mem   ]
[ 0xBD ; xBD ;      ; cp    (IXL)       ;  4 ; undoc ; z80 ; mem   ]
[ 0xBE ; xBE ; d    ; cp    ((IX+d))    ; 15 ; doc   ; z80 ; mem   ]

[ 0xBF ; xBF ;    ; cp     (A)        ;  4 ; undoc ; z80 ; mem   ]
[ 0xC0 ; xC0 ;    ; retcc  (NZcc)     ;  5 ; undoc ; z80 ; mem   ]
[ 0xC1 ; xC1 ;    ; pop    (BC)       ; 10 ; undoc ; z80 ; mem   ]
[ 0xC2 ; xC2 ; nn ; jpcc   (NZcc, nn) ; 10 ; undoc ; z80 ; no     ]
[ 0xC3 ; xC3 ; nn ; jp     (nn)       ; 10 ; undoc ; z80 ; mem   ]
[ 0xC4 ; xC4 ; nn ; callcc (NZcc, nn) ; xx ; undoc ; z80 ; mem   ]
[ 0xC5 ; xC5 ;    ; push   (BC)       ; 11 ; undoc ; z80 ; mem   ]
[ 0xC6 ; xC6 ; n  ; add    (A, n)     ;  7 ; undoc ; z80 ; mem   ]
[ 0xC7 ; xC7 ;    ; rst    (0x00)     ; 11 ; undoc ; z80 ; mem   ]
[ 0xC8 ; xC8 ;    ; retcc  (Zcc)      ;  5 ; undoc ; z80 ; mem   ]
[ 0xC9 ; xC9 ;    ; ret    ()         ; 10 ; undoc ; z80 ; mem   ]
[ 0xCA ; xCA ; nn ; jpcc   (Zcc, nn)  ; 10 ; undoc ; z80 ; no     ]

[ 0xCB ; xCB ;      ; ddcb  ()          ;  0 ; doc ; z80 ; no   ]

[ 0xCC ; xCC ; nn ; callcc (Zcc, nn)  ; xx ; undoc ; z80 ; mem   ]
[ 0xCD ; xCD ; nn ; call   (nn)       ; 17 ; undoc ; z80 ; mem   ]
[ 0xCE ; xCE ; n  ; adc    (A, n)     ;  7 ; undoc ; z80 ; mem   ]
[ 0xCF ; xCF ;    ; rst    (0x08)     ; 11 ; undoc ; z80 ; mem   ]
[ 0xD0 ; xD0 ;    ; retcc  (NCcc)     ;  5 ; undoc ; z80 ; mem   ]
[ 0xD1 ; xD1 ;    ; pop    (DE)       ; 10 ; undoc ; z80 ; mem   ]
[ 0xD2 ; xD2 ; nn ; jpcc   (NCcc, nn) ; 10 ; undoc ; z80 ; no     ]
[ 0xD3 ; xD3 ; n  ; out_n  (n , A)    ; 11 ; undoc ; z80 ; io    ]
[ 0xD4 ; xD4 ; nn ; callcc (NCcc, nn) ; xx ; undoc ; z80 ; mem   ]
[ 0xD5 ; xD5 ;    ; push   (DE)       ; 11 ; undoc ; z80 ; mem   ]
[ 0xD6 ; xD6 ; n  ; sub    (A, n)     ;  7 ; undoc ; z80 ; mem   ]
[ 0xD7 ; xD7 ;    ; rst    (0x10)     ; 11 ; undoc ; z80 ; mem   ]
[ 0xD8 ; xD8 ;    ; retcc  (Ccc)      ;  5 ; undoc ; z80 ; mem   ]
[ 0xD9 ; xD9 ;    ; exx    ()         ;  4 ; undoc ; z80 ; no    ]
[ 0xDA ; xDA ; nn ; jpcc   (Ccc, nn)  ; 10 ; undoc ; z80 ; no     ]
[ 0xDB ; xDB ; n  ; in_n   (A, n)     ; 11 ; undoc ; z80 ; io    ]
[ 0xDC ; xDC ; nn ; callcc (Ccc, nn)  ; xx ; undoc ; z80 ; mem   ]
[ 0xDD ; xDD ;    ; dd     ()         ;  4 ; undoc ; z80 ; no    ]
[ 0xDE ; xDE ; n  ; sbc    (A, n)     ;  7 ; undoc ; z80 ; mem   ]
[ 0xDF ; xDF ;    ; rst    (0x18)     ; 11 ; undoc ; z80 ; mem   ]
[ 0xE0 ; xE0 ;    ; retcc  (POcc)     ;  5 ; undoc ; z80 ; mem   ]

[ 0xE1 ; xE1 ;      ; pop   (IX)        ; 10 ; doc   ; z80 ; mem   ]

[ 0xE2 ; xE2 ; nn ; jpcc   (POcc, nn) ; 10 ; undoc ; z80 ; no     ]

[ 0xE3 ; xE3 ;      ; ex    ((SP), IX)  ; 19 ; doc   ; z80 ; mem   ]

[ 0xE4 ; xE4 ; nn ; callcc (POcc, nn) ; xx ; undoc ; z80 ; mem   ]

[ 0xE5 ; xE5 ;      ; push  (IX)        ; 11 ; doc   ; z80 ; mem   ]

[ 0xE6 ; xE6 ; n  ; and    (n)        ;  7 ; undoc ; z80 ; mem   ]
[ 0xE7 ; xE7 ;    ; rst    (0x20)     ; 11 ; undoc ; z80 ; mem   ]
[ 0xE8 ; xE8 ;    ; retcc  (PEcc)     ;  5 ; undoc ; z80 ; mem   ]

[ 0xE9 ; xE9 ;      ; jp    (IX)        ;  4 ; doc   ; z80 ; mem   ]

[ 0xEA ; xEA ; nn ; jpcc   (PEcc, nn) ; 10 ; undoc ; z80 ; no    ]
[ 0xEB ; xEB ;    ; ex     (DE, HL)   ;  4 ; undoc ; z80 ; mem   ]
[ 0xEC ; xEC ; nn ; callcc (PEcc, nn) ; xx ; undoc ; z80 ; mem   ]
[ 0xED ; xED ;    ; ed     ()         ;  4 ; undoc ; z80 ; no    ]
[ 0xEE ; xEE ; n  ; xor    (n)        ;  7 ; undoc ; z80 ; mem   ]
[ 0xEF ; xEF ;    ; rst    (0x28)     ; 11 ; undoc ; z80 ; mem   ]
[ 0xF0 ; xF0 ;    ; retcc  (Pcc)      ;  5 ; undoc ; z80 ; mem   ]
[ 0xF1 ; xF1 ;    ; pop    (AF)       ; 10 ; undoc ; z80 ; mem   ]
[ 0xF2 ; xF2 ; nn ; jpcc   (Pcc, nn)  ; 10 ; undoc ; z80 ; no    ]
[ 0xF3 ; xF3 ;    ; di     ()         ;  4 ; undoc ; z80 ; no    ]
[ 0xF4 ; xF4 ; nn ; callcc (Pcc, nn)  ; xx ; undoc ; z80 ; mem   ]
[ 0xF5 ; xF5 ;    ; push   (AF)       ; 11 ; undoc ; z80 ; mem   ]
[ 0xF6 ; xF6 ; n  ; or     (n)        ;  7 ; undoc ; z80 ; mem   ]
[ 0xF7 ; xF7 ;    ; rst    (0x30)     ; 11 ; undoc ; z80 ; mem   ]
[ 0xF8 ; xF8 ;    ; retcc  (Mcc)      ;  5 ; undoc ; z80 ; mem   ]

[ 0xF9 ; xF9 ;      ; ld16  (SP, IX)    ;  6 ; doc   ; z80 ; mem   ]

[ 0xFA ; xFA ; nn ; jpcc   (Mcc, nn)  ; 10 ; undoc ; z80 ; no    ]
[ 0xFB ; xFB ;    ; ei     ()         ;  4 ; undoc ; z80 ; no    ]
[ 0xFC ; xFC ; nn ; callcc (Mcc, nn)  ; xx ; undoc ; z80 ; mem   ]
[ 0xFD ; xFD ;    ; fd     ()         ;  4 ; undoc ; z80 ; no    ]
[ 0xFE ; xFE ; n  ; cp     (n)        ;  7 ; undoc ; z80 ; mem   ]
[ 0xFF ; xFF ;    ; rst    (0x38)     ; 11 ; undoc ; z80 ; mem   ]
)
        }
    };
    (@ $mac: ident ($($inst: tt)*)) => {
        $(
            $mac!{$inst}
         )*
    };
}
