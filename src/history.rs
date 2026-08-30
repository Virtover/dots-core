#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History<T> {
    pub past: Vec<T>,
    pub present: T,
    pub future: Vec<T>,
}

impl<T: Clone> History<T> {
    pub fn new(initial: T) -> Self {
        Self {
            past: Vec::new(),
            present: initial,
            future: Vec::new(),
        }
    }

    pub fn apply(&mut self, next: T) {
        self.past.push(self.present.clone());
        self.present = next;
        self.future.clear();
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.past.pop() else {
            return false;
        };
        self.future.push(self.present.clone());
        self.present = previous;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.future.pop() else {
            return false;
        };
        self.past.push(self.present.clone());
        self.present = next;
        true
    }
}
