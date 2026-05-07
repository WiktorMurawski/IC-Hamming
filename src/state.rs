use crate::hamming::{hamming_correct, hamming_encode, hamming_syndrome};

// ─── Application state ───────────────────────────────────────────────────────

#[derive(Clone)]
pub struct HammingState {
    /// The 4 input data bits [d1,d2,d3,d4]
    pub data: [u8; 4],
    /// Encoded 7-bit codeword
    pub codeword: [u8; 7],
    /// "Received" word (may have injected errors)
    pub received: [u8; 7],
    /// Syndrome bits [s1, s2, s4]
    pub syndrome: [u8; 3],
    /// Bit position of detected error (0 = none)
    pub error_pos: usize,
    /// Word after correction
    pub corrected: [u8; 7],
}

impl HammingState {
    pub fn new() -> Self {
        let data = [1, 0, 1, 1];
        let codeword = hamming_encode(data);
        let received = codeword;
        let (syndrome, error_pos) = hamming_syndrome(received);
        let corrected = hamming_correct(received, error_pos);
        HammingState { data, codeword, received, syndrome, error_pos, corrected }
    }

    pub fn recompute(&mut self) {
        self.codeword = hamming_encode(self.data);
        let (syn, ep) = hamming_syndrome(self.received);
        self.syndrome = syn;
        self.error_pos = ep;
        self.corrected = hamming_correct(self.received, ep);
    }
}
