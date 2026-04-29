# Hamming(7,4) Code Visualizer

An interactive desktop application written in Rust that visualizes how the
Hamming(7,4) error-correcting code works, including live syndrome computation
and single-bit error correction.

## Layout

```
┌────────────────────────────────────────────────────────────┐
│   Hamming(7,4) — Interactive Error Correction Visualizer   │
├───────────────┬────────────────────────────────────────────┤
│  Data bits    │  p1  p2  d1  p4  d2  d3  d4                │
│  [d1][d2]     │  pos1 pos2 pos3 pos4 pos5 pos6 pos7        │
│  [d3][d4]     │                                            │
│               │  Codeword  [ 0  1  1  0  0  1  1 ]         │
│  Inject error │  Received  [ 0  1  0  0  0  1  1 ]         │
│  [1][2][3][4] │  Corrected [ 0  1  1  0  0  1  1 ]         │
│  [5][6][7]    │                                            │
│               │  Syndrome Calculation                      │
│  [Reset]      │  s1 = p1^d1^d2^d4 = 0^0^0^1 = 1            │
│               │  s2 = p2^d1^d3^d4 = 1^0^1^1 = 1            │
│               │  s4 = p4^d2^d3^d4 = 0^0^1^1 = 0            │
│               │  Syndrome (s4 s2 s1) = 0 1 1 = 3           │
│               │  Error at bit 3 → flip received[2]         │
└───────────────┴────────────────────────────────────────────┘
```

## How it works

### Hamming(7,4) Encoding
- 4 data bits (d1–d4) are placed at positions 3, 5, 6, 7
- 3 parity bits (p1, p2, p4) are placed at positions 1, 2, 4
- Each parity bit covers a specific set of positions:
  - **p1** (pos 1): covers positions 1, 3, 5, 7
  - **p2** (pos 2): covers positions 2, 3, 6, 7
  - **p4** (pos 4): covers positions 4, 5, 6, 7

### Syndrome Decoding
The receiver calculates three parity checks:
- `s1 = r1 ⊕ r3 ⊕ r5 ⊕ r7`
- `s2 = r2 ⊕ r3 ⊕ r6 ⊕ r7`
- `s4 = r4 ⊕ r5 ⊕ r6 ⊕ r7`

The syndrome `(s4 s2 s1)` read as a 3-bit binary number gives the **position**
of the erroneous bit (0 = no error). Flipping that bit corrects the word.

## Building

### Prerequisites
- Rust (stable) — install from https://rustup.rs

### Compile & Run
```bash
cargo run --release
```
