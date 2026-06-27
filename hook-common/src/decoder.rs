// dnf-login: a launcher for Dungeon & Fighter written in Rust.
// Copyright (C) 2026 llnut
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Pure x86 instruction-length and trampoline-stub decoders.

/// Returns the absolute jump target encoded by a 5-byte trampoline stub
/// at `at`. Recognised forms:
///
///   `E9 rel32`       -- near JMP
///   `68 imm32 C3`    -- PUSH imm32 ; RET
///
/// `FF 25 [addr32]` is not decoded here; the inline-hook copy path
/// handles indirect targets that may be unmapped.
pub fn decode_hook_target(at: usize, bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 6 {
        return None;
    }
    if bytes[0] == 0xE9 {
        let rel = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        return Some((at as i32).wrapping_add(5).wrapping_add(rel) as usize);
    }
    if bytes[0] == 0x68 && bytes[5] == 0xC3 {
        return Some(u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize);
    }
    None
}

/// Returns the byte length of the x86 instruction at `code[0]`. Covers
/// opcodes commonly found in 32-bit Windows API prologues. Unknown
/// opcodes report as 1 byte.
pub fn x86_instr_len(code: &[u8]) -> usize {
    let Some(&op) = code.first() else {
        return 1;
    };
    match op {
        // Single-byte: PUSH/POP r32, NOP, RET, LEAVE, INT3, segment push/pop
        0x50..=0x5F
        | 0x90
        | 0xC3
        | 0xC9
        | 0xCC
        | 0x06
        | 0x07
        | 0x0E
        | 0x16
        | 0x17
        | 0x1E
        | 0x1F => 1,
        // 2-byte: PUSH imm8, JMP short
        0x6A | 0xEB => 2,
        // 3-byte: RET imm16
        0xC2 => 3,
        // ALU r/m,r and r,r/m: ADD, OR, ADC, SBB, AND, SUB, XOR, CMP
        0x00..=0x03 | 0x08..=0x0B | 0x10..=0x13 | 0x18..=0x1B
        | 0x20..=0x23 | 0x28..=0x2B | 0x30..=0x33 | 0x38..=0x3B
        // TEST r/m,r ; MOV variants ; LEA r,m
        | 0x84 | 0x85 | 0x88 | 0x89 | 0x8A | 0x8B | 0x8C | 0x8D | 0x8E => {
            1 + modrm_ext(code, 0)
        }
        // Group-1 imm8, opcodes 80 / 83: opcode + ModRM [+SIB] [+disp] + imm8
        0x80 | 0x83 => 1 + modrm_ext(code, 1),
        // Group-1 imm32, opcode 81: opcode + ModRM [+SIB] [+disp] + imm32
        0x81 => 1 + modrm_ext(code, 4),
        // 5-byte: near JMP/CALL rel32, PUSH imm32, MOV r32 imm32
        0xE9 | 0xE8 | 0x68 | 0xB8..=0xBF => 5,
        // FF 25 [mem32]: 6 bytes
        0xFF if code.len() > 1 && code[1] == 0x25 => 6,
        // Two-byte opcode escape, 0F xx
        0x0F if code.len() > 1 => match code[1] {
            // Jcc near: 0F 8x rel32
            0x80..=0x8F => 6,
            // Reg/mem forms: MOVZX, CMOVcc, IMUL, NOP r/m, etc. The leading 2
            // covers both the 0x0F escape and the secondary opcode byte.
            0x1F | 0x40..=0x4F | 0xAF | 0xB6 | 0xB7 | 0xBE | 0xBF => 2 + modrm_ext(&code[1..], 0),
            _ => 2,
        },
        _ => 1,
    }
}

/// Computes ModRM byte + optional SIB + optional displacement + `imm_extra`.
fn modrm_ext(code: &[u8], imm_extra: usize) -> usize {
    if code.len() < 2 {
        return 1 + imm_extra;
    }
    let modrm = code[1];
    let md = (modrm >> 6) & 3;
    let rm = modrm & 7;
    let has_sib = md != 3 && rm == 4;
    let sib_base = if has_sib && code.len() > 2 {
        code[2] & 7
    } else {
        0
    };
    let disp = match md {
        0 if rm == 5 => 4,
        0 if has_sib && sib_base == 5 => 4,
        1 => 1,
        2 => 4,
        _ => 0,
    };
    1 + has_sib as usize + disp + imm_extra
}

/// Returns the number of bytes to copy into the trampoline so that the copy
/// ends on an instruction boundary and spans at least 5 bytes.
pub fn trampoline_copy_size(code: &[u8]) -> usize {
    let mut n = 0;
    while n < 5 && n < code.len() {
        n += x86_instr_len(&code[n..]);
    }
    n.min(code.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_e9_jmp() {
        // E9 05 00 00 00 -> at + 5 + 5 = at + 10
        let bytes = [0xE9, 0x05, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(decode_hook_target(0x1000, &bytes), Some(0x100A));
    }

    #[test]
    fn decode_e9_jmp_negative() {
        // rel = -16; at + 5 - 16 = at - 11
        let bytes = [0xE9, 0xF0, 0xFF, 0xFF, 0xFF, 0x00];
        assert_eq!(decode_hook_target(0x1000, &bytes), Some(0x0FF5));
    }

    #[test]
    fn decode_push_ret_stub() {
        // 68 78 56 34 12 C3 -> 0x12345678
        let bytes = [0x68, 0x78, 0x56, 0x34, 0x12, 0xC3];
        assert_eq!(decode_hook_target(0x1000, &bytes), Some(0x12345678));
    }

    #[test]
    fn decode_too_short() {
        assert_eq!(decode_hook_target(0x1000, &[0xE9, 0x00]), None);
    }

    #[test]
    fn decode_unknown_opcode() {
        // 68 ... but trailing byte is not C3.
        let bytes = [0x68, 0x00, 0x00, 0x00, 0x00, 0x90];
        assert_eq!(decode_hook_target(0x1000, &bytes), None);
    }

    #[test]
    fn instr_len_single_byte() {
        // push ebp (55), nop (90), ret (C3)
        assert_eq!(x86_instr_len(&[0x55]), 1);
        assert_eq!(x86_instr_len(&[0x90]), 1);
        assert_eq!(x86_instr_len(&[0xC3]), 1);
    }

    #[test]
    fn instr_len_jmp_short() {
        assert_eq!(x86_instr_len(&[0xEB, 0x10]), 2);
    }

    #[test]
    fn instr_len_jmp_near() {
        assert_eq!(x86_instr_len(&[0xE9, 0x00, 0x00, 0x00, 0x00]), 5);
        assert_eq!(x86_instr_len(&[0xE8, 0x00, 0x00, 0x00, 0x00]), 5);
    }

    #[test]
    fn instr_len_mov_ebp_esp() {
        // 8B EC = mov ebp, esp; ModRM = 11 101 100 (mod=3, no disp)
        assert_eq!(x86_instr_len(&[0x8B, 0xEC]), 2);
    }

    #[test]
    fn instr_len_mov_with_disp8() {
        // 8B 4D 08 = mov ecx, [ebp+8]; mod=01, disp8
        assert_eq!(x86_instr_len(&[0x8B, 0x4D, 0x08]), 3);
    }

    #[test]
    fn instr_len_mov_with_disp32() {
        // 8B 8D 60 FF FF FF = mov ecx, [ebp-0xA0]; mod=10, disp32
        assert_eq!(x86_instr_len(&[0x8B, 0x8D, 0x60, 0xFF, 0xFF, 0xFF]), 6);
    }

    #[test]
    fn instr_len_mov_abs_addr() {
        // 8B 0D 78 56 34 12 = mov ecx, [0x12345678]; mod=00 rm=101 disp32
        assert_eq!(x86_instr_len(&[0x8B, 0x0D, 0x78, 0x56, 0x34, 0x12]), 6);
    }

    #[test]
    fn instr_len_sib_disp32_base5() {
        // 8B 04 25 78 56 34 12 = mov eax, [0x12345678]; mod=00 rm=100 SIB base=5
        assert_eq!(
            x86_instr_len(&[0x8B, 0x04, 0x25, 0x78, 0x56, 0x34, 0x12]),
            7
        );
    }

    #[test]
    fn instr_len_add_imm8() {
        // 83 C4 04 = add esp, 4; group-1 imm8
        assert_eq!(x86_instr_len(&[0x83, 0xC4, 0x04]), 3);
    }

    #[test]
    fn instr_len_cmp_imm32() {
        // 81 7D 08 90 01 00 00 = cmp [ebp+8], 0x190
        assert_eq!(
            x86_instr_len(&[0x81, 0x7D, 0x08, 0x90, 0x01, 0x00, 0x00]),
            7
        );
    }

    #[test]
    fn instr_len_mov_imm32() {
        // B8 78 56 34 12 = mov eax, 0x12345678
        assert_eq!(x86_instr_len(&[0xB8, 0x78, 0x56, 0x34, 0x12]), 5);
    }

    #[test]
    fn instr_len_jmp_far_indirect() {
        // FF 25 78 56 34 12 = jmp [0x12345678]
        assert_eq!(x86_instr_len(&[0xFF, 0x25, 0x78, 0x56, 0x34, 0x12]), 6);
    }

    #[test]
    fn instr_len_jcc_near() {
        // 0F 85 00 00 00 00 = jne rel32
        assert_eq!(x86_instr_len(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]), 6);
    }

    #[test]
    fn instr_len_movzx() {
        // 0F B6 4A 02 = movzx ecx, byte ptr [edx+2]
        assert_eq!(x86_instr_len(&[0x0F, 0xB6, 0x4A, 0x02]), 4);
    }

    #[test]
    fn instr_len_movsx_disp32() {
        // 0F BE 8D 60 FF FF FF = movsx ecx, byte ptr [ebp-0xA0]
        assert_eq!(
            x86_instr_len(&[0x0F, 0xBE, 0x8D, 0x60, 0xFF, 0xFF, 0xFF]),
            7
        );
    }

    #[test]
    fn instr_len_cmovcc() {
        // 0F 44 C3 = cmovz eax, ebx; ModRM mod=11 no disp
        assert_eq!(x86_instr_len(&[0x0F, 0x44, 0xC3]), 3);
    }

    #[test]
    fn instr_len_imul_0f_af() {
        // 0F AF 4D 08 = imul ecx, [ebp+8]; mod=01 disp8
        assert_eq!(x86_instr_len(&[0x0F, 0xAF, 0x4D, 0x08]), 4);
    }

    #[test]
    fn instr_len_nop_rm() {
        // 0F 1F 40 00 = nop dword ptr [eax+0]; mod=01 disp8
        assert_eq!(x86_instr_len(&[0x0F, 0x1F, 0x40, 0x00]), 4);
    }

    #[test]
    fn instr_len_unknown_fallback() {
        // Random unsupported opcode -> 1 byte fallback
        assert_eq!(x86_instr_len(&[0xF1]), 1);
    }

    #[test]
    fn instr_len_empty_input() {
        assert_eq!(x86_instr_len(&[]), 1);
    }

    #[test]
    fn copy_size_short_first() {
        // First insn already covers >= 5 bytes
        let code = [0xE9, 0x00, 0x00, 0x00, 0x00, 0x90];
        assert_eq!(trampoline_copy_size(&code), 5);
    }

    #[test]
    fn copy_size_multi_insn() {
        // mov edi, edi (2) + push ebp (1) + mov ebp, esp (2) = 5
        let code = [0x8B, 0xFF, 0x55, 0x8B, 0xEC];
        assert_eq!(trampoline_copy_size(&code), 5);
    }

    #[test]
    fn copy_size_overshoot() {
        // mov edi, edi (2) + push ebp (1) + sub esp, 0x10 (3) = 6, must stop at boundary >=5
        let code = [0x8B, 0xFF, 0x55, 0x83, 0xEC, 0x10];
        assert_eq!(trampoline_copy_size(&code), 6);
    }

    #[test]
    fn copy_size_truncated_input() {
        // Only 3 bytes of input; should not exceed input length
        let code = [0x55, 0x8B, 0xEC];
        assert_eq!(trampoline_copy_size(&code), 3);
    }
}
