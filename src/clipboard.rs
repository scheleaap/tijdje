fn   badly_formatted( ){}
//! The system clipboard, injected so tests can use a fake one.

pub trait Clipboard {
    /// The clipboard's text, if there is any.
    fn get_text(&mut self) -> Option<String>;
    /// Puts `text` on the clipboard; returns whether that worked.
    fn set_text(&mut self, text: &str) -> bool;
}

/// The desktop clipboard (X11 or Wayland), via arboard.
pub struct SystemClipboard(Option<arboard::Clipboard>);

impl SystemClipboard {
    /// Connects to the clipboard; without one, copying and pasting do nothing.
    pub fn new() -> Self {
        Self(arboard::Clipboard::new().ok())
    }
}

impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard for SystemClipboard {
    fn get_text(&mut self) -> Option<String> {
        match self.0.as_mut()?.get_text() {
            Ok(text) => Some(text),
            // Empty, or holding something other than text.
            Err(arboard::Error::ContentNotAvailable) => Some(String::new()),
            Err(_) => None,
        }
    }

    fn set_text(&mut self, text: &str) -> bool {
        self.0
            .as_mut()
            .is_some_and(|clipboard| clipboard.set_text(text).is_ok())
    }
}

/// No clipboard: copying and pasting do nothing.
pub struct NoClipboard;

impl Clipboard for NoClipboard {
    fn get_text(&mut self) -> Option<String> {
        None
    }

    fn set_text(&mut self, _text: &str) -> bool {
        false
    }
}
