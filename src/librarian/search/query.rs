pub struct QuerySearch<'a> {
    pub pattern: &'a str,
    pub repeats: usize,
}

impl<'a> QuerySearch<'a> {
    pub fn new(pattern: &'a str) -> Self {
        Self {
            pattern,
            repeats: 0,
        }
    }

    pub fn repeating(mut self, times: usize) -> Self {
        self.repeats = times;
        self
    }
}
