# Terminal support and painting the background under wide emoji

Supported terminals are VTE-based ones (GNOME Terminal, Ptyxis), Konsole, kitty, foot and ghostty. The UI uses emoji with a variation selector (🏖️, ⚠️) for a pretty look. Ptyxis treats these as one cell wide but draws them two cells wide. ratatui positions the cursor explicitly after them, so columns align. The second cell's background, however, is never painted. After each frame we therefore paint every wide symbol's second cell with a space in the cell's background, then redraw the symbol on top of it.

There is one exception to ratatui's positioning: when the cell covered by such an emoji changes, ratatui writes that cell right after the emoji without moving the cursor. Terminals that advance two cells (Ghostty, kitty, foot) then shift the rest of the row one cell to the right, which can overwrite a panel border. We therefore tell ratatui to skip the covered cells; the repaint above clears them instead.

## Considered Options

- Emoji that are always two cells wide (🌴, ❗): no tricks needed, but not the preferred look.
- Stripping the variation selector (🏖, ⚠): aligns, but renders as plain text symbols.
- Writing every cell with explicit cursor positioning: aligns, but doesn't fix the background, and skipping the covered cells is enough.

## Consequences

Proven with a throwaway prototype in Ptyxis (variant 5). The skipped covered cells were found with a test that replays ratatui's output on a simulated screen, and checked by hand in Ghostty. The other supported terminals are checked once available.
