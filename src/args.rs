pub struct KrarkArgs {
    pub max_failed_shown: usize,
    pub max_panicked_shown: usize,
}

impl Default for KrarkArgs {
    fn default() -> Self {
        KrarkArgs {
            max_failed_shown: 10,
            max_panicked_shown: 3,
        }
    }
}
