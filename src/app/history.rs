//! Undo/redo history: one step per accepted change to a day.

use jiff::civil::Date;

use crate::domain::Day;

/// One accepted change: the day before and after it.
#[derive(Debug, Clone)]
pub struct Change {
    pub date: Date,
    pub before: Day,
    pub after: Day,
}

/// One history for the whole session, across all days.
#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Change>,
    redo: Vec<Change>,
}

impl History {
    /// Records a new change; changes that were undone can no longer be redone.
    pub fn record(&mut self, change: Change) {
        self.undo.push(change);
        self.redo.clear();
    }

    /// The change to undo, if any; it can be redone afterwards.
    pub fn undo(&mut self) -> Option<&Change> {
        move_last(&mut self.undo, &mut self.redo)
    }

    /// The change to redo, if any; it can be undone again afterwards.
    pub fn redo(&mut self) -> Option<&Change> {
        move_last(&mut self.redo, &mut self.undo)
    }
}

/// Moves the last change of `from` to `to` and returns it.
fn move_last<'a>(from: &mut Vec<Change>, to: &'a mut Vec<Change>) -> Option<&'a Change> {
    to.push(from.pop()?);
    to.last()
}
