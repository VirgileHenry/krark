pub struct KrarkHarness {
    test_args: libtest_mimic::Arguments,
}

impl KrarkHarness {
    pub fn new() -> KrarkHarness {
        KrarkHarness {
            test_args: libtest_mimic::Arguments::from_args(),
        }
    }
}
