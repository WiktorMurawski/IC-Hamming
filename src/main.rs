//! Hamming(7,4) Interactive Visualizer
//!
//! Controls:
//!   • d1–d4 buttons (top-left): toggle input data bits
//!   • [1]–[7] buttons: flip individual received bits to simulate errors
//!   • "Reset errors" button: restore received word = codeword
//!
//! The canvas shows:
//!   • Codeword row  – the correctly encoded 7-bit word
//!   • Received row  – the word "seen" after the channel (with injected errors)
//!   • Corrected row – result after Hamming syndrome correction
//!   • Syndrome formulas evaluated live
//!   • Parity coverage legend

use fltk::{
    app,
    button::Button,
    draw,
    enums::{Align, Color, Font, FrameType},
    frame::Frame,
    prelude::*,
    window::Window,
};
use std::cell::RefCell;
use std::rc::Rc;

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
fn hamming_encode(data: [u8; 4]) -> [u8; 7] {
    let [d1, d2, d3, d4] = data;
    let p1 = d1 ^ d2 ^ d4;        // positions 1,3,5,7
    let p2 = d1 ^ d3 ^ d4;        // positions 2,3,6,7
    let p4 = d2 ^ d3 ^ d4;        // positions 4,5,6,7
    [p1, p2, d1, p4, d2, d3, d4]
}

/// Compute syndrome from a received 7-bit word.
/// Returns ([s1,s2,s4], error_position).
/// error_position == 0 means no error.
fn hamming_syndrome(word: [u8; 7]) -> ([u8; 3], usize) {
    let [r1, r2, r3, r4, r5, r6, r7] = word;
    let s1 = r1 ^ r3 ^ r5 ^ r7;   // check positions 1,3,5,7
    let s2 = r2 ^ r3 ^ r6 ^ r7;   // check positions 2,3,6,7
    let s4 = r4 ^ r5 ^ r6 ^ r7;   // check positions 4,5,6,7
    let error_pos = (s4 as usize) * 4 + (s2 as usize) * 2 + (s1 as usize);
    ([s1, s2, s4], error_pos)
}

/// Correct a single-bit error given the syndrome error position.
fn hamming_correct(mut word: [u8; 7], error_pos: usize) -> [u8; 7] {
    if error_pos >= 1 && error_pos <= 7 {
        word[error_pos - 1] ^= 1;
    }
    word
}

// ─── Application state ───────────────────────────────────────────────────────

#[derive(Clone)]
struct HammingState {
    /// The 4 input data bits [d1,d2,d3,d4]
    data: [u8; 4],
    /// Encoded 7-bit codeword
    codeword: [u8; 7],
    /// "Received" word (may have injected errors)
    received: [u8; 7],
    /// Syndrome bits [s1, s2, s4]
    syndrome: [u8; 3],
    /// Bit position of detected error (0 = none)
    error_pos: usize,
    /// Word after correction
    corrected: [u8; 7],
}

impl HammingState {
    fn new() -> Self {
        let data = [1, 0, 1, 1];
        let codeword = hamming_encode(data);
        let received = codeword;
        let (syndrome, error_pos) = hamming_syndrome(received);
        let corrected = hamming_correct(received, error_pos);
        HammingState { data, codeword, received, syndrome, error_pos, corrected }
    }

    fn recompute(&mut self) {
        self.codeword = hamming_encode(self.data);
        let (syn, ep) = hamming_syndrome(self.received);
        self.syndrome = syn;
        self.error_pos = ep;
        self.corrected = hamming_correct(self.received, ep);
    }
}

// ─── Drawing ─────────────────────────────────────────────────────────────────

const BG_DEEP:    (u8,u8,u8) = (18, 18, 32);
const BG_PANEL:   (u8,u8,u8) = (24, 24, 44);
const BG_PARITY:  (u8,u8,u8) = (40, 28, 72);
const BG_DATA:    (u8,u8,u8) = (22, 36, 60);
const BG_ERR:     (u8,u8,u8) = (88, 18, 18);
const BG_FIX:     (u8,u8,u8) = (18, 72, 32);

const COL_ACCENT: (u8,u8,u8) = (99, 179, 237);
const COL_GREEN:  (u8,u8,u8) = (72, 199, 142);
const COL_RED:    (u8,u8,u8) = (252, 110, 110);
const COL_YELLOW: (u8,u8,u8) = (246, 211, 101);
const COL_PURPLE: (u8,u8,u8) = (183, 148, 246);
const COL_FG:     (u8,u8,u8) = (226, 232, 240);
const COL_MUTED:  (u8,u8,u8) = (100, 116, 139);

fn c(rgb: (u8,u8,u8)) -> Color { Color::from_rgb(rgb.0, rgb.1, rgb.2) }

/// Draw the visualization panel inside the canvas frame.
fn draw_panel(s: &HammingState, x: i32, y: i32, w: i32) {
    // ── Bit grid ─────────────────────────────────────────────────────────────
    let cell_w = 54;
    let cell_h = 46;
    let total_w = 7 * cell_w;
    let bx = x + (w - total_w) / 2;  // center the grid

    let col_labels  = ["p1","p2","d1","p4","d2","d3","d4"];
    let is_parity   = [true, true, false, true, false, false, false];

    // Column headers
    draw::set_font(Font::CourierBold, 13);
    for i in 0..7usize {
        let cx = bx + i as i32 * cell_w;
        draw::set_draw_color(if is_parity[i] { c(COL_PURPLE) } else { c(COL_ACCENT) });
        draw::draw_text2(col_labels[i], cx, y + 4, cell_w, 16, Align::Center);
    }
    draw::set_font(Font::Courier, 10);
    draw::set_draw_color(c(COL_MUTED));
    for i in 0..7usize {
        let cx = bx + i as i32 * cell_w;
        draw::draw_text2(&format!("pos {}", i+1), cx, y + 19, cell_w, 13, Align::Center);
    }

    // Three rows: codeword, received, corrected
    let row_defs: [(&str, [u8;7], (u8,u8,u8)); 3] = [
        ("Codeword",  s.codeword,  COL_GREEN),
        ("Received",  s.received,  COL_YELLOW),
        ("Corrected", s.corrected, COL_ACCENT),
    ];

    for (ri, (row_label, bits, row_col)) in row_defs.iter().enumerate() {
        let ry = y + 38 + ri as i32 * (cell_h + 10);

        // Row label
        draw::set_font(Font::CourierBold, 12);
        draw::set_draw_color(c(*row_col));
        draw::draw_text2(row_label, x, ry, bx - x - 6, cell_h, Align::Right | Align::Center);

        for i in 0..7usize {
            let cx = bx + i as i32 * cell_w;
            let bit = bits[i];
            let is_err = ri == 1 && s.error_pos == i + 1;
            let is_fix = ri == 2 && s.error_pos != 0 && s.error_pos == i + 1;

            // Cell background
            let bg = if is_err { c(BG_ERR) }
                     else if is_fix  { c(BG_FIX) }
                     else if is_parity[i] { c(BG_PARITY) }
                     else { c(BG_DATA) };
            draw::set_draw_color(bg);
            draw::draw_rectf(cx + 3, ry + 3, cell_w - 6, cell_h - 6);

            // Border
            let border = if is_err { c(COL_RED) }
                         else if is_fix { c(COL_GREEN) }
                         else if is_parity[i] { c(COL_PURPLE) }
                         else { c(COL_MUTED) };
            draw::set_draw_color(border);
            draw::draw_rect(cx + 3, ry + 3, cell_w - 6, cell_h - 6);

            // Bit digit
            draw::set_font(Font::CourierBold, 22);
            draw::set_draw_color(c(COL_FG));
            draw::draw_text2(&bit.to_string(), cx, ry + 2, cell_w, cell_h - 10, Align::Center);

            // ERR / FIX tag
            if is_err || is_fix {
                draw::set_font(Font::CourierBold, 9);
                let tag_col = if is_err { c(COL_RED) } else { c(COL_GREEN) };
                let tag_str = if is_err { "ERR" } else { "FIX" };
                draw::set_draw_color(tag_col);
                draw::draw_text2(tag_str, cx, ry + cell_h - 16, cell_w, 12, Align::Center);
            }
        }
    }

    // ── Separator ────────────────────────────────────────────────────────────
    let sep_y = y + 38 + 3 * (cell_h + 10) + 6;
    draw::set_draw_color(c(COL_MUTED));
    draw::draw_line(x + 4, sep_y, x + w - 4, sep_y);

    // ── Syndrome formulas ────────────────────────────────────────────────────
    draw::set_font(Font::CourierBold, 14);
    draw::set_draw_color(c(COL_ACCENT));
    draw::draw_text2("Syndrome Calculation", x, sep_y + 8, w, 20, Align::Center);

    let [r1,r2,r3,r4,r5,r6,r7] = s.received;
    let [s1,s2,s4] = s.syndrome;

    let formulas = [
        format!("s1  =  p1 ^ d1 ^ d2 ^ d4   =   {} ^ {} ^ {} ^ {}   =   {}", r1, r3, r5, r7, s1),
        format!("s2  =  p2 ^ d1 ^ d3 ^ d4   =   {} ^ {} ^ {} ^ {}   =   {}", r2, r3, r6, r7, s2),
        format!("s4  =  p4 ^ d2 ^ d3 ^ d4   =   {} ^ {} ^ {} ^ {}   =   {}", r4, r5, r6, r7, s4),
    ];

    draw::set_font(Font::Courier, 13);
    draw::set_draw_color(c(COL_PURPLE));
    for (fi, formula) in formulas.iter().enumerate() {
        draw::draw_text2(formula, x + 16, sep_y + 32 + fi as i32 * 22, w - 32, 20, Align::Left);
    }

    // Syndrome result
    let syndrome_val = (s4 as usize) * 4 + (s2 as usize) * 2 + (s1 as usize);
    let res_y = sep_y + 32 + 3 * 22 + 6;

    draw::set_draw_color(c(COL_MUTED));
    draw::draw_line(x + 16, res_y, x + w - 16, res_y);

    draw::set_font(Font::CourierBold, 13);
    draw::set_draw_color(c(COL_ACCENT));
    draw::draw_text2(
        &format!("Syndrome  (s4 s2 s1)  =  {} {} {}  =  {} in decimal",
            s4, s2, s1, syndrome_val),
        x + 16, res_y + 6, w - 32, 20, Align::Left);

    let (msg, msg_col) = if s.error_pos == 0 {
        ("✓  Syndrome = 0 — no error detected".to_string(), c(COL_GREEN))
    } else {
        (format!("✗  Syndrome = {} → error at bit position {}  →  flip received[{}]",
            syndrome_val, s.error_pos, s.error_pos), c(COL_RED))
    };
    draw::set_font(Font::CourierBold, 14);
    draw::set_draw_color(msg_col);
    draw::draw_text2(&msg, x + 16, res_y + 28, w - 32, 22, Align::Left);

    // ── Coverage legend ──────────────────────────────────────────────────────
    let leg_y = res_y + 58;
    draw::set_draw_color(c(COL_MUTED));
    draw::draw_line(x + 4, leg_y, x + w - 4, leg_y);

    draw::set_font(Font::CourierBold, 13);
    draw::set_draw_color(c(COL_ACCENT));
    draw::draw_text2("Parity Bit Coverage", x, leg_y + 6, w, 18, Align::Center);

    let coverages = [
        ("p1", "covers positions 1, 3, 5, 7  →  checks d1, d2, d4"),
        ("p2", "covers positions 2, 3, 6, 7  →  checks d1, d3, d4"),
        ("p4", "covers positions 4, 5, 6, 7  →  checks d2, d3, d4"),
    ];
    draw::set_font(Font::Courier, 12);
    for (ci, (name, desc)) in coverages.iter().enumerate() {
        let ly = leg_y + 26 + ci as i32 * 19;
        draw::set_draw_color(c(COL_PURPLE));
        draw::draw_text2(name, x + 16, ly, 28, 17, Align::Left);
        draw::set_draw_color(c(COL_FG));
        draw::draw_text2(desc, x + 46, ly, w - 62, 17, Align::Left);
    }

    // How error position is determined
    let exp_y = leg_y + 26 + 3 * 19 + 6;
    draw::set_draw_color(c(COL_MUTED));
    draw::draw_line(x + 4, exp_y, x + w - 4, exp_y);
    draw::set_font(Font::Courier, 12);
    draw::set_draw_color(c(COL_MUTED));
    draw::draw_text2(
        "Error position = 4·s4 + 2·s2 + 1·s1   (syndrome bits form a binary address)",
        x + 16, exp_y + 6, w - 32, 17, Align::Left);
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    let app = app::App::default();
    app::set_background_color(BG_DEEP.0, BG_DEEP.1, BG_DEEP.2);
    app::set_foreground_color(COL_FG.0, COL_FG.1, COL_FG.2);

    let (win_w, win_h) = (920, 740);
    let mut wind = Window::new(100, 80, win_w, win_h, "Hamming(7,4) Code Visualizer");
    wind.set_color(c(BG_DEEP));

    let state = Rc::new(RefCell::new(HammingState::new()));

    // ── Title bar ─────────────────────────────────────────────────────────────
    let mut title = Frame::new(0, 0, win_w, 46, "Hamming(7,4)  —  Interactive Error Correction Visualizer");
    title.set_color(c((24, 24, 50)));
    title.set_frame(FrameType::FlatBox);
    title.set_label_color(c(COL_ACCENT));
    title.set_label_font(Font::CourierBold);
    title.set_label_size(17);

    // ── Left sidebar ──────────────────────────────────────────────────────────
    let (sb_x, sb_y, sb_w) = (8, 54, 190);

    let mut lbl_data = Frame::new(sb_x, sb_y, sb_w, 17, "Data bits (toggle)");
    lbl_data.set_label_color(c(COL_GREEN));
    lbl_data.set_label_font(Font::CourierBold);
    lbl_data.set_label_size(11);

    let (tx, rx) = app::channel::<(u8, u8)>();

    // ── Data bit buttons ──────────────────────────────────────────────────────
    let mut data_btns: Vec<Button> = (0..4usize).map(|i| {
        let mut btn = Button::new(sb_x + i as i32 * 46, sb_y + 19, 42, 42, "");
        btn.set_label_font(Font::CourierBold);
        btn.set_label_size(13);
        btn.set_color(c((30, 52, 85)));
        btn.set_label_color(c(COL_FG));
        btn.set_frame(FrameType::RoundedBox);
        btn.emit(tx.clone(), (0u8, i as u8));
        btn
    }).collect();

    // ── Error-injection buttons (7 received bits) ─────────────────────────────
    let mut lbl_err = Frame::new(sb_x, sb_y + 70, sb_w, 16, "Inject error (flip bit)");
    lbl_err.set_label_color(c(COL_RED));
    lbl_err.set_label_font(Font::CourierBold);
    lbl_err.set_label_size(11);

    let mut err_btns: Vec<Button> = (0..7usize).map(|i| {
        let row = i / 4;
        let col = i % 4;
        let mut btn = Button::new(
            sb_x + col as i32 * 46,
            sb_y + 88 + row as i32 * 46,
            42, 42, "");
        btn.set_label_font(Font::CourierBold);
        btn.set_label_size(12);
        btn.set_color(c((55, 18, 18)));
        btn.set_label_color(c(COL_RED));
        btn.set_frame(FrameType::RoundedBox);
        btn.emit(tx.clone(), (1u8, i as u8));
        btn
    }).collect();

    // ── Reset button ──────────────────────────────────────────────────────────
    let mut reset_btn = Button::new(sb_x, sb_y + 188, sb_w - 4, 30, "Reset errors");
    reset_btn.set_label_font(Font::CourierBold);
    reset_btn.set_label_size(12);
    reset_btn.set_color(c((18, 52, 22)));
    reset_btn.set_label_color(c(COL_GREEN));
    reset_btn.set_frame(FrameType::RoundedBox);
    reset_btn.emit(tx.clone(), (2u8, 0u8));

    // ── Legend labels ─────────────────────────────────────────────────────────
    let legend_text = "Legend:\n\
                       Purple = parity bit\n\
                       Blue   = data bit\n\
                       Red    = error injected\n\
                       Green  = error corrected";
    let mut legend = Frame::new(sb_x, sb_y + 228, sb_w, 90, legend_text);
    legend.set_label_color(c(COL_MUTED));
    legend.set_label_font(Font::Courier);
    legend.set_label_size(10);
    legend.set_align(Align::Left | Align::Top | Align::Inside);

    // ── Main canvas ───────────────────────────────────────────────────────────
    let cv_x = sb_x + sb_w + 8;
    let cv_y = sb_y;
    let cv_w = win_w - cv_x - 8;
    let cv_h = win_h - cv_y - 8;

    let state_for_draw = state.clone();
    let mut canvas = Frame::new(cv_x, cv_y, cv_w, cv_h, "");
    canvas.set_color(c(BG_PANEL));
    canvas.set_frame(FrameType::FlatBox);
    canvas.draw(move |f| {
        let s = state_for_draw.borrow();
        draw_panel(&s, f.x() + 10, f.y() + 10, f.width() - 20);
    });

    wind.end();
    wind.show();

    // ── Sync button labels from initial state ─────────────────────────────────
    {
        let s = state.borrow();
        for (i, btn) in data_btns.iter_mut().enumerate() {
            btn.set_label(&format!("d{}\n{}", i + 1, s.data[i]));
        }
        for (i, btn) in err_btns.iter_mut().enumerate() {
            btn.set_label(&format!("[{}]\n{}", i + 1, s.received[i]));
        }
    }

    let data_btns = Rc::new(RefCell::new(data_btns));
    let err_btns  = Rc::new(RefCell::new(err_btns));

    // ── Event loop ────────────────────────────────────────────────────────────
    while app.wait() {
        if let Some((kind, idx)) = rx.recv() {
            // Mutate state
            {
                let mut s = state.borrow_mut();
                match kind {
                    0 => { // toggle a data bit → re-encode, reset received
                        s.data[idx as usize] ^= 1;
                        s.codeword = hamming_encode(s.data);
                        s.received = s.codeword;
                        s.recompute();
                    }
                    1 => { // flip a received bit → re-run syndrome
                        s.received[idx as usize] ^= 1;
                        s.recompute();
                    }
                    2 => { // reset: restore received = codeword
                        s.received = s.codeword;
                        s.recompute();
                    }
                    _ => {}
                }
            }

            // Update sidebar button labels
            {
                let s = state.borrow();
                for (i, btn) in data_btns.borrow_mut().iter_mut().enumerate() {
                    btn.set_label(&format!("d{}\n{}", i + 1, s.data[i]));
                }
                for (i, btn) in err_btns.borrow_mut().iter_mut().enumerate() {
                    btn.set_label(&format!("[{}]\n{}", i + 1, s.received[i]));
                    btn.set_color(if s.error_pos == i + 1 {
                        c((100, 18, 18))  // highlight if this is the error bit
                    } else {
                        c((55, 18, 18))
                    });
                }
            }

            canvas.redraw();
        }
    }
}
