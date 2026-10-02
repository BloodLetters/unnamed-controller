//! The AVR DES instruction, implemented without a C crypto dependency.
//!
//! AVR DES performs one DES Feistel round and applies the initial and inverse
//! initial permutations on every instruction. Sixteen calls with K=0..15 are
//! one complete encryption/decryption operation.

const IP: [u8; 64] = [
    58, 50, 42, 34, 26, 18, 10, 2, 60, 52, 44, 36, 28, 20, 12, 4, 62, 54, 46, 38, 30, 22, 14, 6,
    64, 56, 48, 40, 32, 24, 16, 8, 57, 49, 41, 33, 25, 17, 9, 1, 59, 51, 43, 35, 27, 19, 11, 3, 61,
    53, 45, 37, 29, 21, 13, 5, 63, 55, 47, 39, 31, 23, 15, 7,
];
const FP: [u8; 64] = [
    40, 8, 48, 16, 56, 24, 64, 32, 39, 7, 47, 15, 55, 23, 63, 31, 38, 6, 46, 14, 54, 22, 62, 30,
    37, 5, 45, 13, 53, 21, 61, 29, 36, 4, 44, 12, 52, 20, 60, 28, 35, 3, 43, 11, 51, 19, 59, 27,
    34, 2, 42, 10, 50, 18, 58, 26, 33, 1, 41, 9, 49, 17, 57, 25,
];
const E: [u8; 48] = [
    32, 1, 2, 3, 4, 5, 4, 5, 6, 7, 8, 9, 8, 9, 10, 11, 12, 13, 12, 13, 14, 15, 16, 17, 16, 17, 18,
    19, 20, 21, 20, 21, 22, 23, 24, 25, 24, 25, 26, 27, 28, 29, 28, 29, 30, 31, 32, 1,
];
const P: [u8; 32] = [
    16, 7, 20, 21, 29, 12, 28, 17, 1, 15, 23, 26, 5, 18, 31, 10, 2, 8, 24, 14, 32, 27, 3, 9, 19,
    13, 30, 6, 22, 11, 4, 25,
];
const PC1: [u8; 56] = [
    57, 49, 41, 33, 25, 17, 9, 1, 58, 50, 42, 34, 26, 18, 10, 2, 59, 51, 43, 35, 27, 19, 11, 3, 60,
    52, 44, 36, 63, 55, 47, 39, 31, 23, 15, 7, 62, 54, 46, 38, 30, 22, 14, 6, 61, 53, 45, 37, 29,
    21, 13, 5, 28, 20, 12, 4,
];
const PC2: [u8; 48] = [
    14, 17, 11, 24, 1, 5, 3, 28, 15, 6, 21, 10, 23, 19, 12, 4, 26, 8, 16, 7, 27, 20, 13, 2, 41, 52,
    31, 37, 47, 55, 30, 40, 51, 45, 33, 48, 44, 49, 39, 56, 34, 53, 46, 42, 50, 36, 29, 32,
];
const SHIFTS: [u8; 16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];
const SBOX: [[[u8; 16]; 4]; 8] = [
    [
        [14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7],
        [0, 15, 7, 4, 14, 2, 13, 1, 10, 6, 12, 11, 9, 5, 3, 8],
        [4, 1, 14, 8, 13, 6, 2, 11, 15, 12, 9, 7, 3, 10, 5, 0],
        [15, 12, 8, 2, 4, 9, 1, 7, 5, 11, 3, 14, 10, 0, 6, 13],
    ],
    [
        [15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10],
        [3, 13, 4, 7, 15, 2, 8, 14, 12, 0, 1, 10, 6, 9, 11, 5],
        [0, 14, 7, 11, 10, 4, 13, 1, 5, 8, 12, 6, 9, 3, 2, 15],
        [13, 8, 10, 1, 3, 15, 4, 2, 11, 6, 7, 12, 0, 5, 14, 9],
    ],
    [
        [10, 0, 9, 14, 6, 3, 15, 5, 1, 13, 12, 7, 11, 4, 2, 8],
        [13, 7, 0, 9, 3, 4, 6, 10, 2, 8, 5, 14, 12, 11, 15, 1],
        [13, 6, 4, 9, 8, 15, 3, 0, 11, 1, 2, 12, 5, 10, 14, 7],
        [1, 10, 13, 0, 6, 9, 8, 7, 4, 15, 14, 3, 11, 5, 2, 12],
    ],
    [
        [7, 13, 14, 3, 0, 6, 9, 10, 1, 2, 8, 5, 11, 12, 4, 15],
        [13, 8, 11, 5, 6, 15, 0, 3, 4, 7, 2, 12, 1, 10, 14, 9],
        [10, 6, 9, 0, 12, 11, 7, 13, 15, 1, 3, 14, 5, 2, 8, 4],
        [3, 15, 0, 6, 10, 1, 13, 8, 9, 4, 5, 11, 12, 7, 2, 14],
    ],
    [
        [2, 12, 4, 1, 7, 10, 11, 6, 8, 5, 3, 15, 13, 0, 14, 9],
        [14, 11, 2, 12, 4, 7, 13, 1, 5, 0, 15, 10, 3, 9, 8, 6],
        [4, 2, 1, 11, 10, 13, 7, 8, 15, 9, 12, 5, 6, 3, 0, 14],
        [11, 8, 12, 7, 1, 14, 2, 13, 6, 15, 0, 9, 10, 4, 5, 3],
    ],
    [
        [12, 1, 10, 15, 9, 2, 6, 8, 0, 13, 3, 4, 14, 7, 5, 11],
        [10, 15, 4, 2, 7, 12, 9, 5, 6, 1, 13, 14, 0, 11, 3, 8],
        [9, 14, 15, 5, 2, 8, 12, 3, 7, 0, 4, 10, 1, 13, 11, 6],
        [4, 3, 2, 12, 9, 5, 15, 10, 11, 14, 1, 7, 6, 0, 8, 13],
    ],
    [
        [4, 11, 2, 14, 15, 0, 8, 13, 3, 12, 9, 7, 5, 10, 6, 1],
        [13, 0, 11, 7, 4, 9, 1, 10, 14, 3, 5, 12, 2, 15, 8, 6],
        [1, 4, 11, 13, 12, 3, 7, 14, 10, 15, 6, 8, 0, 5, 9, 2],
        [6, 11, 13, 8, 1, 4, 10, 7, 9, 5, 0, 15, 14, 2, 3, 12],
    ],
    [
        [13, 2, 8, 4, 6, 15, 11, 1, 10, 9, 3, 14, 5, 0, 12, 7],
        [1, 15, 13, 8, 10, 3, 7, 4, 12, 5, 6, 11, 0, 14, 9, 2],
        [7, 11, 4, 1, 9, 12, 14, 2, 0, 6, 10, 13, 15, 3, 5, 8],
        [2, 1, 14, 7, 4, 10, 8, 13, 15, 12, 9, 0, 3, 5, 6, 11],
    ],
];

fn permute(value: u64, table: &[u8], input_bits: u8) -> u64 {
    let mut out = 0;
    let mut i = 0;
    while i < table.len() {
        out = (out << 1) | ((value >> (input_bits - table[i])) & 1);
        i += 1;
    }
    out
}

fn rotate28(value: u64, count: u8) -> u64 {
    ((value << count) | (value >> (28 - count))) & 0x0fff_ffff
}

fn subkey(key: u64, round: u8) -> u64 {
    let permuted = permute(key, &PC1, 64);
    let mut c = (permuted >> 28) & 0x0fff_ffff;
    let mut d = permuted & 0x0fff_ffff;
    let mut i = 0;
    while i <= round {
        c = rotate28(c, SHIFTS[i as usize]);
        d = rotate28(d, SHIFTS[i as usize]);
        i += 1;
    }
    permute((c << 28) | d, &PC2, 56)
}

fn feistel(right: u32, key: u64) -> u32 {
    let expanded = permute(right as u64, &E, 32);
    let mixed = expanded ^ key;
    let mut substituted = 0u32;
    let mut box_index = 0;
    while box_index < 8 {
        let shift = 42 - box_index * 6;
        let six = ((mixed >> shift) & 0x3f) as u8;
        let row = ((six & 0x20) >> 4) | (six & 1);
        let column = (six >> 1) & 0x0f;
        substituted = (substituted << 4) | SBOX[box_index][row as usize][column as usize] as u32;
        box_index += 1;
    }
    permute(substituted as u64, &P, 32) as u32
}

/// Execute one AVR DES instruction. `data` and `key` use the AVR register
/// ordering: byte zero is R0/R8 and contains the least significant byte.
pub fn des_round(data: [u8; 8], key: [u8; 8], round: u8, decrypt: bool) -> [u8; 8] {
    let mut block = u64::from_le_bytes(data);
    let key_value = u64::from_le_bytes(key);
    let round = if decrypt {
        15 - (round & 0x0f)
    } else {
        round & 0x0f
    };
    block = permute(block, &IP, 64);
    let mut left = (block >> 32) as u32;
    let mut right = block as u32;
    let next_left = right;
    let next_right = left ^ feistel(right, subkey(key_value, round));
    left = next_left;
    right = next_right;
    let (output_left, output_right) = if round == 15 {
        (right, left)
    } else {
        (left, right)
    };
    permute((output_left as u64) << 32 | output_right as u64, &FP, 64).to_le_bytes()
}

#[cfg(test)]
mod tests {
    use super::{des_round, feistel, permute, rotate28, subkey, IP, PC1};

    #[test]
    fn one_round_is_deterministic() {
        let a = des_round([0; 8], [0; 8], 0, false);
        assert_eq!(a, des_round([0; 8], [0; 8], 0, false));
        assert_ne!(a, [0; 8]);
    }

    #[test]
    fn sixteen_rounds_match_the_standard_des_vector() {
        let mut data = [0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01];
        let key = [0xf1, 0xdf, 0xbc, 0x9b, 0x79, 0x57, 0x34, 0x13];
        let mut round = 0;
        let ip = permute(u64::from_le_bytes(data), &IP, 64);
        let key_permuted = permute(u64::from_le_bytes(key), &PC1, 64);
        assert_eq!(key_permuted, 0xf0ccaaf556678f);
        assert_eq!(rotate28(key_permuted >> 28, 1), 0xe19955f);
        assert_eq!(rotate28(rotate28(key_permuted >> 28, 1), 1), 0xc332abf);
        assert_eq!(subkey(u64::from_le_bytes(key), 0), 0x1b02effc7072);
        assert_eq!(ip, 0xcc00ccfff0aaf0aa);
        assert_eq!(
            feistel(ip as u32, subkey(u64::from_le_bytes(key), 0)),
            0x234aa9bb
        );
        while round < 16 {
            data = des_round(data, key, round, false);
            if round == 0 {
                let state = permute(u64::from_le_bytes(data), &IP, 64);
                assert_eq!(state, 0xf0aaf0aaef4a6544);
            }
            if round == 1 {
                assert_eq!(subkey(u64::from_le_bytes(key), 1), 0x79aed9dbc9e5);
                let state = permute(u64::from_le_bytes(data), &IP, 64);
                assert_eq!(state, 0xef4a6544cc017709);
            }
            round += 1;
        }
        assert_eq!(data, [0x05, 0xb4, 0x0a, 0x0f, 0x54, 0x13, 0xe8, 0x85]);
    }
}
