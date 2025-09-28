pub struct KrarkHarness {
    test_args: libtest_mimic::Arguments,
}

impl KrarkHarness {
    pub fn new() -> KrarkHarness {
        KrarkHarness {
            test_args: libtest_mimic::Arguments::from_args(),
        }
    }

    pub fn run<
        F: Fn(&mtg_cardbase::Card, KrarkResult) -> KrarkResult + std::panic::RefUnwindSafe,
    >(
        &mut self,
        test_func: F,
    ) {
        for card in mtg_cardbase::ALL_CARDS.iter() {
            let _result =
                match std::panic::catch_unwind(|| test_func(card, KrarkResult::Ok { passed: 0 })) {
                    Ok(result) => result,
                    Err(payload) => KrarkResult::from_panic_payload(payload),
                };
        }
    }

    pub fn run_on_sample<
        F: Fn(&mtg_cardbase::Card, KrarkResult) -> KrarkResult + std::panic::RefUnwindSafe,
    >(
        &mut self,
        test_func: F,
        sample_size: usize,
    ) {
        let cards_count = mtg_cardbase::ALL_CARDS.len();
        let sample_size = sample_size.min(cards_count);
        let jump_size = sample_size / cards_count;

        let mut cards = mtg_cardbase::ALL_CARDS.iter();
        for _ in 0..sample_size {
            let card = match cards.next() {
                Some(next) => next,
                None => break, /* unreachable */
            };

            let result =
                match std::panic::catch_unwind(|| test_func(card, KrarkResult::Ok { passed: 0 })) {
                    Ok(result) => result,
                    Err(payload) => KrarkResult::from_panic_payload(payload),
                };

            for _ in 0..jump_size - 1 {
                let _ = cards.next();
            }
        }
    }
}

pub enum KrarkResult {
    Ok { passed: usize },
    Err { passed: usize, failed: Vec<String> },
    Panicked { trace: String },
}

impl KrarkResult {
    fn from_panic_payload(payload: Box<dyn std::any::Any + Send + 'static>) -> KrarkResult {
        let trace = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };
        KrarkResult::Panicked { trace }
    }

    pub fn assert_eq<T: PartialEq + std::fmt::Debug>(
        &mut self,
        expected: T,
        obtained: T,
        name: String,
    ) {
        /* Weird trick to avoid borrow checker limitations */
        let mut own: &mut Self = self;
        match (&mut own, expected == obtained) {
            (KrarkResult::Panicked { .. }, _) => { /*  */ }
            (KrarkResult::Ok { passed }, true) => *passed += 1,
            (KrarkResult::Ok { passed }, false) => {
                /* Here, we fight the borrow checker, hence the "own" */
                *own = KrarkResult::Err {
                    passed: *passed,
                    failed: vec![format!(
                        "Assertion failed: {name}, expected {expected:?}, found {obtained:?}"
                    )],
                };
            }
            (KrarkResult::Err { passed, .. }, true) => *passed += 1,
            (KrarkResult::Err { failed, .. }, false) => failed.push(format!(
                "Assertion failed: {name}, expected {expected:?}, found {obtained:?}"
            )),
        };
    }
}
