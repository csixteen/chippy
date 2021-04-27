use std::collections::VecDeque;

#[derive(Default)]
pub(crate) struct DebugLog(VecDeque<String>);

impl DebugLog {
    pub fn new(size: usize) -> Self {
        DebugLog(VecDeque::with_capacity(size))
    }

    pub fn push(&mut self, entry: String) {
        self.0.push_back(entry);

        if self.0.len() == self.0.capacity() {
            self.0.pop_front();
        }
    }
}
