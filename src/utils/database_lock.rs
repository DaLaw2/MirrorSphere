pub struct DatabaseLock {
    _private: (),
}

impl DatabaseLock {
    pub fn acquire() -> Self {
        Self { _private: () }
    }
}
