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
        sample_size: usize,
        test_func: F,
    ) {
        let jump_size = (mtg_cardbase::ALL_CARDS.len() / sample_size).saturating_sub(1);
        for card in mtg_cardbase::ALL_CARDS.iter().step_by(jump_size) {
            let _result =
                match std::panic::catch_unwind(|| test_func(card, KrarkResult::Ok { passed: 0 })) {
                    Ok(result) => result,
                    Err(payload) => KrarkResult::from_panic_payload(payload),
                };
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

    pub fn assert_ok<T, E: std::fmt::Debug>(&mut self, result: Result<T, E>, name: String) {
        /* Weird trick to avoid borrow checker limitations */
        let mut own: &mut Self = self;
        match (&mut own, result) {
            (KrarkResult::Panicked { .. }, _) => { /*  */ }
            (KrarkResult::Ok { passed }, Ok(_)) => *passed += 1,
            (KrarkResult::Ok { passed }, Err(err)) => {
                /* Here, we fight the borrow checker, hence the "own" */
                *own = KrarkResult::Err {
                    passed: *passed,
                    failed: vec![format!(
                        "Assertion failed: {name}, expected Ok(_), found Err({err:?})"
                    )],
                };
            }
            (KrarkResult::Err { passed, .. }, Ok(_)) => *passed += 1,
            (KrarkResult::Err { failed, .. }, Err(err)) => failed.push(format!(
                "Assertion failed: {name}, expected Ok(_), found Err({err:?})"
            )),
        };
    }
}
