pub struct NextId(usize);

impl NextId {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn next(&mut self) -> usize {
        self.0 += 1;
        self.0 - 1
    }
}
