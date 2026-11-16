use ratatui::{buffer::Buffer, layout::Rect};
use tijdje::wide_symbols;

#[test]
fn repainting_wide_symbols_leaves_the_cursor_where_it_was() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 1));
    buffer.set_string(0, 0, "⚠️ x", ratatui::style::Style::default());
    let mut out = Vec::new();
    wide_symbols::paint_backgrounds(&buffer, &mut out).unwrap();
    let out = String::from_utf8(out).unwrap();
    // DECSC (save cursor) first, DECRC (restore cursor) last.
    assert!(out.starts_with("\x1b7"), "{out:?}");
    assert!(out.ends_with("\x1b8"), "{out:?}");
    assert!(out.contains("⚠️"), "the emoji is repainted: {out:?}");
}
