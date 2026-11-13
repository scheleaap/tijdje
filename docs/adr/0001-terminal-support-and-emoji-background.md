# Terminal support and painting the background under wide emoji

Supported terminals are VTE-based ones (GNOME Terminal, Ptyxis), Konsole, kitty, foot and ghostty. The UI uses emoji with a variation selector (🏖️, ⚠️) for a pretty look. Ptyxis treats these as one cell wide but draws them two cells wide. ratatui already positions the cursor correctly after them, so columns align. The second cell's background, however, is never painted. After each frame we therefore paint every wide symbol's second cell with a space in the cell's background, then redraw the symbol on top of it.

## Considered Options

- Emoji that are always two cells wide (🌴, ❗): no tricks needed, but not the preferred look.
- Stripping the variation selector (🏖, ⚠): aligns, but renders as plain text symbols.
- Writing every cell with explicit cursor positioning: aligns, but doesn't fix the background and isn't needed, since ratatui already aligns.

## Consequences

Proven with a throwaway prototype in Ptyxis (variant 5). The other supported terminals are checked once available.
