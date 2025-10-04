pub struct KrarkResult {
    pub card_name: &'static str,
    pub status: KrarkResultStatus,
}

impl KrarkResult {
    pub(crate) fn new(card_name: &'static str) -> KrarkResult {
        KrarkResult {
            card_name,
            status: KrarkResultStatus::Passed(PassedResult {
                card_name,
                passed: vec![],
            }),
        }
    }

    pub(crate) fn from_panic_payload(
        card_name: &'static str,
        payload: Box<dyn std::any::Any + Send + 'static>,
    ) -> KrarkResult {
        /* Most panic payloads are strings with panic info */
        let trace = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };
        KrarkResult {
            card_name,
            status: KrarkResultStatus::Panicked(PanickedResult { card_name, trace }),
        }
    }

    pub fn assert_eq<T: PartialEq + std::fmt::Debug>(
        &mut self,
        expected: T,
        obtained: T,
        name: String,
    ) {
        match (&mut self.status, expected == obtained) {
            (KrarkResultStatus::Panicked { .. }, _) => { /*  */ }
            (KrarkResultStatus::Passed(passed), true) => passed.passed.push(name),
            (KrarkResultStatus::Passed(passed), false) => {
                self.status = KrarkResultStatus::Failed(FailedResult {
                    card_name: self.card_name,
                    passed: std::mem::take(&mut passed.passed),
                    failed: vec![FailedTc {
                        check_name: name,
                        failure: format!("expected {:?}, obtained {:?}", expected, obtained),
                    }],
                });
            }
            (KrarkResultStatus::Failed(failed), true) => failed.passed.push(name),
            (KrarkResultStatus::Failed(failed), false) => failed.failed.push(FailedTc {
                check_name: name,
                failure: format!("expected {:?}, obtained {:?}", expected, obtained),
            }),
        };
    }

    pub fn assert_ok<T, E: std::fmt::Debug>(&mut self, result: Result<T, E>, name: String) {
        match (&mut self.status, result) {
            (KrarkResultStatus::Panicked { .. }, _) => { /*  */ }
            (KrarkResultStatus::Passed(passed), Ok(_)) => passed.passed.push(name),
            (KrarkResultStatus::Passed(passed), Err(err)) => {
                self.status = KrarkResultStatus::Failed(FailedResult {
                    card_name: self.card_name,
                    passed: std::mem::take(&mut passed.passed),
                    failed: vec![FailedTc {
                        check_name: name,
                        failure: format!("expected Ok(_), obtained Err: {:?}", err),
                    }],
                });
            }
            (KrarkResultStatus::Failed(failed), Ok(_)) => failed.passed.push(name),
            (KrarkResultStatus::Failed(failed), Err(err)) => failed.failed.push(FailedTc {
                check_name: name,
                failure: format!("expected Ok(_), obtained Err: {:?}", err),
            }),
        };
    }
}

pub enum KrarkResultStatus {
    Passed(PassedResult),
    Failed(FailedResult),
    Panicked(PanickedResult),
}

pub struct PassedResult {
    pub card_name: &'static str,
    pub passed: Vec<String>,
}
pub struct FailedResult {
    pub card_name: &'static str,
    pub passed: Vec<String>,
    pub failed: Vec<FailedTc>,
}
pub struct PanickedResult {
    pub card_name: &'static str,
    pub trace: String,
}

pub struct FailedTc {
    pub check_name: String,
    pub failure: String,
}
impl KrarkResultStatus {}
