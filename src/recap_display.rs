pub fn output_recap(
    harness: &crate::KrarkHarness,
    recap: crate::KrarkRecap,
) -> std::io::Result<()> {
    let mut output: Box<dyn std::io::Write> = match &harness.test_args.logfile {
        None => Box::new(std::io::stdout()),
        Some(path) => Box::new(
            std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)?,
        ),
    };
    let _color: bool = match (&harness.test_args.color, &harness.test_args.logfile) {
        (Some(libtest_mimic::ColorSetting::Always), _) => true,
        (Some(libtest_mimic::ColorSetting::Never), _) => false,
        (_, logfile) => logfile.is_none(),
    };

    let passed = recap.passed;
    let failed = recap.failed;
    let panicked = recap.panicked;
    let total = passed + failed + panicked;

    let lines = [
        ("Passed", format!("{}", passed)),
        ("Failed", format!("{}", failed)),
        ("Panicked", format!("{}", panicked)),
        ("Total", format!("{}", total)),
    ];

    let title_required_size = harness.name.chars().count();
    let status_column_width = lines
        .iter()
        .map(|(name, _)| name.chars().count())
        .max()
        .unwrap_or(0);
    let result_column_width = lines
        .iter()
        .map(|(_, res)| res.chars().count())
        .max()
        .unwrap_or(0);
    /* Increase status column if the title require it */
    let status_column_width =
        status_column_width.max(title_required_size.saturating_sub(1 + result_column_width));

    let mut table_display =
        TableDisplay::new(&mut output, [status_column_width, result_column_width]);

    table_display.separator_line("┏", &[("━", "━"), ("━", "┓")])?;
    table_display.result_line("┃", &[(&harness.name, " "), ("", "┃")])?;
    table_display.separator_line("┣", &[("━", "┯"), ("━", "┨")])?;
    table_display.result_line("┃", &[("Passed", "│"), (&format!("{}", passed), "┃")])?;
    table_display.separator_line("┣", &[("─", "┼"), ("─", "┨")])?;
    table_display.result_line("┃", &[("Failed", "│"), (&format!("{}", failed), "┃")])?;
    table_display.result_line("┃", &[("Panicked", "│"), (&format!("{}", panicked), "┃")])?;
    table_display.result_line("┃", &[("Total", "│"), (&format!("{}", total), "┃")])?;
    table_display.separator_line("┗", &[("━", "┷"), ("━", "┛")])?;

    Ok(())
}

struct TableDisplay<'out, W: std::io::Write, const N: usize> {
    output: &'out mut W,
    column_sizes: [usize; N],
}

impl<'out, W: std::io::Write, const N: usize> TableDisplay<'out, W, N> {
    fn new(output: &'out mut W, column_sizes: [usize; N]) -> Self {
        TableDisplay {
            output,
            column_sizes,
        }
    }

    fn separator_line(&mut self, first: &str, columns: &[(&str, &str); N]) -> std::io::Result<()> {
        self.output.write(first.as_bytes())?;
        for ((column_line, column_end), size) in columns.iter().zip(self.column_sizes.iter()) {
            for _ in 0..*size {
                self.output.write(column_line.as_bytes())?;
            }
            self.output.write(column_end.as_bytes())?;
        }
        Ok(())
    }

    fn result_line(&mut self, first: &str, columns: &[(&str, &str); N]) -> std::io::Result<()> {
        self.output.write(first.as_bytes())?;
        for ((column_value, end_sep), size) in columns.iter().zip(self.column_sizes.iter()) {
            self.output.write(column_value.as_bytes())?;
            for _ in 0..*size {
                self.output.write(" ".as_bytes())?;
            }
            self.output.write(end_sep.as_bytes())?;
        }
        Ok(())
    }
}
