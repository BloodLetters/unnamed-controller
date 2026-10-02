use crate::types::ProjectData;

/// Manages undo and redo action history using project snapshots.
pub struct HistoryStack {
    undo_stack: Vec<ProjectData>,
    redo_stack: Vec<ProjectData>,
    max_depth: usize,
}

impl Default for HistoryStack {
    fn default() -> Self {
        Self::new(50)
    }
}

impl HistoryStack {
    /// Creates a new history stack with the given maximum undo depth.
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    /// Records the current state before a mutation occurs.
    pub fn record_state(&mut self, state: ProjectData) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(state);
        self.redo_stack.clear();
    }

    /// Restores the previous state from the undo stack.
    pub fn undo(&mut self, current: ProjectData) -> Option<ProjectData> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current);
        Some(prev)
    }

    /// Restores the next state from the redo stack.
    pub fn redo(&mut self, current: ProjectData) -> Option<ProjectData> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current);
        Some(next)
    }

    /// Queries whether undo is currently available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Queries whether redo is currently available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_project() -> ProjectData {
        ProjectData::default()
    }

    #[test]
    fn test_undo_redo_lifecycle() {
        let mut history = HistoryStack::new(10);
        let s0 = dummy_project();
        let s1 = dummy_project();
        let s2 = dummy_project();

        history.record_state(s0);
        history.record_state(s1);

        assert!(history.can_undo());
        assert!(!history.can_redo());

        let restored_s1 = history.undo(s2);
        assert!(restored_s1.is_some());
        assert!(history.can_undo());
        assert!(history.can_redo());

        let redone = history.redo(restored_s1.unwrap());
        assert!(redone.is_some());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_record_clears_redo() {
        let mut history = HistoryStack::new(10);
        let s0 = dummy_project();
        let s1 = dummy_project();
        let s2 = dummy_project();

        history.record_state(s0);
        let _ = history.undo(s1);
        assert!(history.can_redo());

        history.record_state(s2);
        assert!(!history.can_redo());
    }
}
