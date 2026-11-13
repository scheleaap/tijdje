# Terminal support and emoji choice

Supported terminals are VTE-based ones (GNOME Terminal, Ptyxis), Konsole, kitty, foot and ghostty. The UI only uses emoji that are two cells wide by default (🌴, ❗, 🤒, 🎄, ☕), not emoji that need a variation selector to become wide (🏖️, ⚠️). A prototype in Ptyxis showed that ratatui keeps columns aligned for both kinds, but Ptyxis renders the background behind 🏖️ and ⚠️ only partially.

## Considered Options

- 🏖️ and ⚠️ with explicit cursor positioning: aligned, but with a broken background in Ptyxis.
- Stripping the variation selector (🏖, ⚠): aligned, but renders as plain text symbols.

## Consequences

No custom cursor positioning is needed; ratatui's own rendering suffices. The other supported terminals are checked once available.
