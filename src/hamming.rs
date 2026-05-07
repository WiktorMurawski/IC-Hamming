// ─── Hamming(7,4) core ───────────────────────────────────────────────────────
//
// Bit layout (1-indexed positions):
//   pos 1 → p1  (parity, covers 1,3,5,7)
//   pos 2 → p2  (parity, covers 2,3,6,7)
//   pos 3 → d1  (data)
//   pos 4 → p4  (parity, covers 4,5,6,7)
//   pos 5 → d2  (data)
//   pos 6 → d3  (data)
//   pos 7 → d4  (data)

/// Encode 4 data bits into a 7-bit Hamming codeword.
pub fn hamming_encode(data: [u8; 4]) -> [u8; 7] {
    let [d1, d2, d3, d4] = data;
    let p1 = d1 ^ d2 ^ d4; // positions 1,3,5,7
    let p2 = d1 ^ d3 ^ d4; // positions 2,3,6,7
    let p4 = d2 ^ d3 ^ d4; // positions 4,5,6,7
    [p1, p2, d1, p4, d2, d3, d4]
}

/// Compute syndrome from a received 7-bit word.
/// Returns ([s1,s2,s4], error_position).
/// error_position == 0 means no error.
pub fn hamming_syndrome(word: [u8; 7]) -> ([u8; 3], usize) {
    let [r1, r2, r3, r4, r5, r6, r7] = word;
    let s1 = r1 ^ r3 ^ r5 ^ r7; // check positions 1,3,5,7
    let s2 = r2 ^ r3 ^ r6 ^ r7; // check positions 2,3,6,7
    let s4 = r4 ^ r5 ^ r6 ^ r7; // check positions 4,5,6,7
    let error_pos = (s4 as usize) * 4 + (s2 as usize) * 2 + (s1 as usize);
    ([s1, s2, s4], error_pos)
}

/// Correct a single-bit error given the syndrome error position.
pub fn hamming_correct(mut word: [u8; 7], error_pos: usize) -> [u8; 7] {
    if error_pos >= 1 && error_pos <= 7 {
        word[error_pos - 1] ^= 1;
    }
    word
}
