.intel_syntax noprefix

.text
        .globl _euphrates_x64_supports_pattern_to_palette_indices
	.globl euphrates_x64_supports_pattern_to_palette_indices

_euphrates_x64_supports_pattern_to_palette_indices:
euphrates_x64_supports_pattern_to_palette_indices:
	push	rbx
	mov	eax, 7
	mov	ecx, 0
	cpuid
	and	ebx, 0x100
	shr	ebx, 8
	mov	eax, ebx
	pop	rbx
	ret

        .globl _euphrates_x64_pattern_to_palette_indices
	.globl euphrates_x64_pattern_to_palette_indices

_euphrates_x64_pattern_to_palette_indices:
euphrates_x64_pattern_to_palette_indices:
	mov	rax, rcx
	mov	rdx, rcx
	mov	r8, rcx
	shr	rax, 8
	shr	rdx, 16
	shr	rcx, 24
	mov	r9, 0x0101010101010101
	mov	r10, 0x0202020202020202
	mov	r11, 0x0404040404040404
	pdep	r8, r8, r9
	pdep	rax, rax, r10
	mov	r9, 0x0808080808080808
	pdep	rdx, rdx, r11
	pdep	rcx, rcx, r9
	or	rax, r8
	or	rdx, rcx
	or	rax, rdx
	bswap	rax
	ret
