mod drawing;
mod hamming;
mod state;

use drawing::{c, sc, draw_panel};
use drawing::{
    BG_DEEP, BG_PANEL, COL_ACCENT, COL_FG, COL_GREEN, COL_MUTED, COL_RED, COL_PURPLE,
};
use state::HammingState;
use hamming::hamming_encode;

use fltk::{
    app,
    button::Button,
    enums::{Align, Font, FrameType},
    frame::Frame,
    prelude::*,
    window::Window,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let app = app::App::default();
    app::set_background_color(BG_DEEP.0, BG_DEEP.1, BG_DEEP.2);
    app::set_foreground_color(COL_FG.0, COL_FG.1, COL_FG.2);

    let (win_w, win_h) = (sc(900), sc(600));
    let mut wind = Window::new(100, 80, win_w, win_h, "Hamming(7,4) Code Visualizer");
    wind.set_color(c(BG_DEEP));

    let state = Rc::new(RefCell::new(HammingState::new()));

    // ── Title bar ─────────────────────────────────────────────────────────────
    let title_h = sc(46);
    let mut title = Frame::new(
        0, 0, win_w, title_h,
        "Hamming(7,4)  —  Interactive Error Correction Visualizer",
    );
    title.set_color(c((24, 24, 50)));
    title.set_frame(FrameType::FlatBox);
    title.set_label_color(c(COL_ACCENT));
    title.set_label_font(Font::CourierBold);
    title.set_label_size(sc(20));

    // ── Left sidebar ──────────────────────────────────────────────────────────
    let sb_x = sc(8);
    let sb_y = sc(54);
    let sb_w = sc(190);
    let btn_spacing = sc(46);
    let btn_size    = sc(42);

    let mut lbl_data = Frame::new(sb_x, sb_y, sb_w, sc(17), "Data bits (toggle)");
    lbl_data.set_label_color(c(COL_GREEN));
    lbl_data.set_label_font(Font::CourierBold);
    lbl_data.set_label_size(sc(14));

    let (tx, rx) = app::channel::<(u8, u8)>();

    // ── Data bit buttons ──────────────────────────────────────────────────────
    let mut data_btns: Vec<Button> = (0..4usize).map(|i| {
        let mut btn = Button::new(
            sb_x + i as i32 * btn_spacing,
            sb_y + sc(19),
            btn_size, btn_size, "",
        );
        btn.set_label_font(Font::CourierBold);
        btn.set_label_size(sc(16));
        btn.set_color(c((30, 52, 85)));
        btn.set_label_color(c(COL_FG));
        btn.set_frame(FrameType::RoundedBox);
        btn.emit(tx.clone(), (0u8, i as u8));
        btn
    }).collect();

    // ── Error-injection buttons (7 received bits) ─────────────────────────────
    let mut lbl_err = Frame::new(sb_x, sb_y + sc(70), sb_w, sc(16), "Inject error (flip bit)");
    lbl_err.set_label_color(c(COL_RED));
    lbl_err.set_label_font(Font::CourierBold);
    lbl_err.set_label_size(sc(14));

    let mut err_btns: Vec<Button> = (0..7usize).map(|i| {
        let row = i / 4;
        let col = i % 4;
        let mut btn = Button::new(
            sb_x + col as i32 * btn_spacing,
            sb_y + sc(88) + row as i32 * btn_spacing,
            btn_size, btn_size, "",
        );
        btn.set_label_font(Font::CourierBold);
        btn.set_label_size(sc(14));
        btn.set_color(c((55, 18, 18)));
        btn.set_label_color(c(COL_RED));
        btn.set_frame(FrameType::RoundedBox);
        btn.emit(tx.clone(), (1u8, i as u8));
        btn
    }).collect();

    // ── Reset button ──────────────────────────────────────────────────────────
    let mut reset_btn = Button::new(sb_x, sb_y + sc(188), sb_w - 4, sc(30), "Reset errors");
    reset_btn.set_label_font(Font::CourierBold);
    reset_btn.set_label_size(sc(14));
    reset_btn.set_color(c((18, 52, 22)));
    reset_btn.set_label_color(c(COL_GREEN));
    reset_btn.set_frame(FrameType::RoundedBox);
    reset_btn.emit(tx.clone(), (2u8, 0u8));

    // ── Legend ────────────────────────────────────────────────────────────────
    let legend_text = "Legend:\n\
                       Purple = parity bit\n\
                       Blue   = data bit\n\
                       Red    = error injected\n\
                       Green  = error corrected";
    let mut legend = Frame::new(sb_x, sb_y + sc(228), sb_w, sc(90), legend_text);
    legend.set_label_color(c(COL_MUTED));
    legend.set_label_font(Font::Courier);
    legend.set_label_size(sc(12));
    legend.set_align(Align::Left | Align::Top | Align::Inside);

    // ── Main canvas ───────────────────────────────────────────────────────────
    let cv_x = sb_x + sb_w + sc(8);
    let cv_y = sb_y;
    let cv_w = win_w - cv_x - sc(8);
    let cv_h = win_h - cv_y - sc(8);

    let state_for_draw = state.clone();
    let mut canvas = Frame::new(cv_x, cv_y, cv_w, cv_h, "");
    canvas.set_color(c(BG_PANEL));
    canvas.set_frame(FrameType::FlatBox);
    canvas.draw(move |f| {
        let s = state_for_draw.borrow();
        draw_panel(&s, f.x() + sc(10), f.y() + sc(10), f.width() - sc(20));
    });

    // ── Bottom-right label ────────────────────────────────────────────────────
    let label_w = sc(200);
    let label_h = sc(24);
    let mut bottom_label = Frame::new(
        win_w - label_w - sc(8),
        win_h - label_h - sc(8),
        label_w,
        label_h,
        "Wiktor Murawski, Wiktor Pańczak, Mikołaj Złotek",
    );
    bottom_label.set_label_color(c(COL_MUTED));
    bottom_label.set_label_font(Font::Courier);
    bottom_label.set_label_size(sc(14));
    bottom_label.set_align(Align::Right | Align::Inside);

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

            {
                let s = state.borrow();
                for (i, btn) in data_btns.borrow_mut().iter_mut().enumerate() {
                    btn.set_label(&format!("d{}\n{}", i + 1, s.data[i]));
                }
                for (i, btn) in err_btns.borrow_mut().iter_mut().enumerate() {
                    btn.set_label(&format!("[{}]\n{}", i + 1, s.received[i]));
                    btn.set_color(if s.error_pos == i + 1 {
                        c((100, 18, 18))
                    } else {
                        c((55, 18, 18))
                    });
                }
            }

            canvas.redraw();
        }
    }
}
