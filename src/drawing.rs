use fltk::{
    draw,
    enums::{Align, Color, Font},
};

use crate::state::HammingState;

// ─── Global scale factor ─────────────────────────────────────────────────────
//
// Change SCALE to resize everything proportionally.
// 1.0 = original size, 1.3 = 30% larger, 1.5 = 50% larger, etc.

pub const SCALE: f32 = 1.4;

/// Scale a pixel/font value by the global SCALE factor.
pub fn sc(n: i32) -> i32 {
    (n as f32 * SCALE) as i32
}

// ─── Color palette ───────────────────────────────────────────────────────────

pub const BG_DEEP:    (u8, u8, u8) = (18, 18, 32);
pub const BG_PANEL:   (u8, u8, u8) = (24, 24, 44);
pub const BG_PARITY:  (u8, u8, u8) = (40, 28, 72);
pub const BG_DATA:    (u8, u8, u8) = (22, 36, 60);
pub const BG_ERR:     (u8, u8, u8) = (88, 18, 18);
pub const BG_FIX:     (u8, u8, u8) = (18, 72, 32);

pub const COL_ACCENT: (u8, u8, u8) = (99, 179, 237);
pub const COL_GREEN:  (u8, u8, u8) = (72, 199, 142);
pub const COL_RED:    (u8, u8, u8) = (252, 110, 110);
pub const COL_YELLOW: (u8, u8, u8) = (246, 211, 101);
pub const COL_PURPLE: (u8, u8, u8) = (183, 148, 246);
pub const COL_FG:     (u8, u8, u8) = (226, 232, 240);
pub const COL_MUTED:  (u8, u8, u8) = (100, 116, 139);

/// Convert an RGB tuple into an FLTK `Color`.
pub fn c(rgb: (u8, u8, u8)) -> Color {
    Color::from_rgb(rgb.0, rgb.1, rgb.2)
}

// ─── Canvas drawing ──────────────────────────────────────────────────────────

/// Draw the visualization panel inside the canvas frame.
pub fn draw_panel(s: &HammingState, x: i32, y: i32, w: i32) {
    // ── Bit grid ─────────────────────────────────────────────────────────────
    let cell_w = sc(54);
    let cell_h = sc(46);
    let total_w = 7 * cell_w;
    let bx = x + (w - total_w) / 2; // center the grid

    let col_labels = ["p1", "p2", "d1", "p4", "d2", "d3", "d4"];
    let is_parity  = [true, true, false, true, false, false, false];

    // Column headers
    draw::set_font(Font::CourierBold, sc(13));
    for i in 0..7usize {
        let cx = bx + i as i32 * cell_w;
        draw::set_draw_color(if is_parity[i] { c(COL_PURPLE) } else { c(COL_ACCENT) });
        draw::draw_text2(col_labels[i], cx, y + sc(4), cell_w, sc(16), Align::Center);
    }
    draw::set_font(Font::Courier, sc(10));
    draw::set_draw_color(c(COL_MUTED));
    for i in 0..7usize {
        let cx = bx + i as i32 * cell_w;
        draw::draw_text2(&format!("pos {}", i + 1), cx, y + sc(19), cell_w, sc(13), Align::Center);
    }

    // Three rows: codeword, received, corrected
    let row_defs: [(&str, [u8; 7], (u8, u8, u8)); 3] = [
        ("Codeword",  s.codeword,  COL_GREEN),
        ("Received",  s.received,  COL_YELLOW),
        ("Corrected", s.corrected, COL_ACCENT),
    ];

    for (ri, (row_label, bits, row_col)) in row_defs.iter().enumerate() {
        let ry = y + sc(38) + ri as i32 * (cell_h + sc(10));

        // Row label
        draw::set_font(Font::CourierBold, sc(12));
        draw::set_draw_color(c(*row_col));
        draw::draw_text2(row_label, x, ry, bx - x - 6, cell_h, Align::Right | Align::Center);

        for i in 0..7usize {
            let cx = bx + i as i32 * cell_w;
            let bit = bits[i];
            let is_err = ri == 1 && s.error_pos == i + 1;
            let is_fix = ri == 2 && s.error_pos != 0 && s.error_pos == i + 1;

            // Cell background
            let bg = if is_err       { c(BG_ERR) }
                     else if is_fix  { c(BG_FIX) }
                     else if is_parity[i] { c(BG_PARITY) }
                     else            { c(BG_DATA) };
            draw::set_draw_color(bg);
            draw::draw_rectf(cx + 3, ry + 3, cell_w - 6, cell_h - 6);

            // Border
            let border = if is_err          { c(COL_RED) }
                         else if is_fix     { c(COL_GREEN) }
                         else if is_parity[i] { c(COL_PURPLE) }
                         else               { c(COL_MUTED) };
            draw::set_draw_color(border);
            draw::draw_rect(cx + 3, ry + 3, cell_w - 6, cell_h - 6);

            // Bit digit
            draw::set_font(Font::CourierBold, sc(22));
            draw::set_draw_color(c(COL_FG));
            draw::draw_text2(&bit.to_string(), cx, ry + 2, cell_w, cell_h - sc(10), Align::Center);

            // ERR / FIX tag
            if is_err || is_fix {
                draw::set_font(Font::CourierBold, sc(9));
                let tag_col = if is_err { c(COL_RED) } else { c(COL_GREEN) };
                let tag_str = if is_err { "ERR" } else { "FIX" };
                draw::set_draw_color(tag_col);
                draw::draw_text2(tag_str, cx, ry + cell_h - sc(16), cell_w, sc(12), Align::Center);
            }
        }
    }

    // ── Separator ────────────────────────────────────────────────────────────
    let sep_y = y + sc(38) + 3 * (cell_h + sc(10)) + sc(6);
    draw::set_draw_color(c(COL_MUTED));
    draw::draw_line(x + 4, sep_y, x + w - 4, sep_y);

    // ── Syndrome formulas ────────────────────────────────────────────────────
    draw::set_font(Font::CourierBold, sc(14));
    draw::set_draw_color(c(COL_ACCENT));
    draw::draw_text2("Syndrome Calculation", x, sep_y + sc(8), w, sc(20), Align::Center);

    let [r1, r2, r3, r4, r5, r6, r7] = s.received;
    let [s1, s2, s4] = s.syndrome;

    let formulas = [
        format!("s1  =  p1 ^ d1 ^ d2 ^ d4   =   {} ^ {} ^ {} ^ {}   =   {}", r1, r3, r5, r7, s1),
        format!("s2  =  p2 ^ d1 ^ d3 ^ d4   =   {} ^ {} ^ {} ^ {}   =   {}", r2, r3, r6, r7, s2),
        format!("s4  =  p4 ^ d2 ^ d3 ^ d4   =   {} ^ {} ^ {} ^ {}   =   {}", r4, r5, r6, r7, s4),
    ];

    draw::set_font(Font::Courier, sc(13));
    draw::set_draw_color(c(COL_PURPLE));
    for (fi, formula) in formulas.iter().enumerate() {
        draw::draw_text2(
            formula,
            x + sc(16),
            sep_y + sc(32) + fi as i32 * sc(22),
            w - sc(32),
            sc(20),
            Align::Left,
        );
    }

    // Syndrome result
    let syndrome_val = (s4 as usize) * 4 + (s2 as usize) * 2 + (s1 as usize);
    let res_y = sep_y + sc(32) + 3 * sc(22) + sc(6);

    draw::set_draw_color(c(COL_MUTED));
    draw::draw_line(x + sc(16), res_y, x + w - sc(16), res_y);

    draw::set_font(Font::CourierBold, sc(13));
    draw::set_draw_color(c(COL_ACCENT));
    draw::draw_text2(
        &format!(
            "Syndrome  (s4 s2 s1)  =  {} {} {}  =  {} in decimal",
            s4, s2, s1, syndrome_val
        ),
        x + sc(16),
        res_y + sc(6),
        w - sc(32),
        sc(20),
        Align::Left,
    );

    let (msg, msg_col) = if s.error_pos == 0 {
        ("✓  Syndrome = 0 — no error detected".to_string(), c(COL_GREEN))
    } else {
        (
            format!(
                "✗  Syndrome = {} → error at bit position {}  →  flip received[{}]",
                syndrome_val, s.error_pos, s.error_pos
            ),
            c(COL_RED),
        )
    };
    draw::set_font(Font::CourierBold, sc(14));
    draw::set_draw_color(msg_col);
    draw::draw_text2(&msg, x + sc(16), res_y + sc(28), w - sc(32), sc(22), Align::Left);

    // ── Coverage legend ──────────────────────────────────────────────────────
    let leg_y = res_y + sc(58);
    draw::set_draw_color(c(COL_MUTED));
    draw::draw_line(x + 4, leg_y, x + w - 4, leg_y);

    draw::set_font(Font::CourierBold, sc(13));
    draw::set_draw_color(c(COL_ACCENT));
    draw::draw_text2("Parity Bit Coverage", x, leg_y + sc(6), w, sc(18), Align::Center);

    let coverages = [
        ("p1", "covers positions 1, 3, 5, 7  →  checks d1, d2, d4"),
        ("p2", "covers positions 2, 3, 6, 7  →  checks d1, d3, d4"),
        ("p4", "covers positions 4, 5, 6, 7  →  checks d2, d3, d4"),
    ];
    draw::set_font(Font::Courier, sc(12));
    for (ci, (name, desc)) in coverages.iter().enumerate() {
        let ly = leg_y + sc(26) + ci as i32 * sc(19);
        draw::set_draw_color(c(COL_PURPLE));
        draw::draw_text2(name, x + sc(16), ly, sc(28), sc(17), Align::Left);
        draw::set_draw_color(c(COL_FG));
        draw::draw_text2(desc, x + sc(46), ly, w - sc(62), sc(17), Align::Left);
    }

    // How error position is determined
    let exp_y = leg_y + sc(26) + 3 * sc(19) + sc(6);
    draw::set_draw_color(c(COL_MUTED));
    draw::draw_line(x + 4, exp_y, x + w - 4, exp_y);
    draw::set_font(Font::Courier, sc(12));
    draw::set_draw_color(c(COL_MUTED));
    draw::draw_text2(
        "Error position = 4·s4 + 2·s2 + 1·s1   (syndrome bits form a binary address)",
        x + sc(16),
        exp_y + sc(6),
        w - sc(32),
        sc(17),
        Align::Left,
    );
}
