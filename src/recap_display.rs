const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_RESET: &str = "\x1b[0m";


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
    let color: bool = match (&harness.test_args.color, &harness.test_args.logfile) {
        (Some(libtest_mimic::ColorSetting::Always), _) => true,
        (Some(libtest_mimic::ColorSetting::Never), _) => false,
        (_, logfile) => logfile.is_none(),
    };

    let (color_green, color_red, color_reset) = if color {
        (COLOR_GREEN, COLOR_RED, COLOR_RESET)
    } else {
        ("", "", "")
    };

    let passed = recap.passed;
    let failed = recap.failed;
    let panicked = recap.panicked;
    let total = passed + failed + panicked;

    let color_passed = if passed == total {
        color_green
    } else {
        color_red
    };
    let color_failed = match failed {
        0 => color_green,
        _ => color_red,
    };
    let color_panicked = match panicked {
        0 => color_green,
        _ => color_red,
    };
    let color_total = color_reset;

    let lines = [
        ("Passed", format!("{}{} ({:.1}%){}",  color_passed, passed, passed as f32 / total as f32 * 100.0, color_reset)),
        ("Failed", format!("{}{} ({:.1}%){}", color_failed, failed, failed as f32 / total as f32 * 100.0, color_reset)),
        ("Panicked", format!("{}{} ({:.1}%){}", color_panicked, panicked, panicked as f32 / total as f32 * 100.0, color_reset)),
        ("Total", format!("{}{}{}", color_total, total, color_reset)),
    ];

    let status_column_width = lines
        .iter()
        .map(|(name, _)| terminal_str_len(name))
        .chain(std::iter::once(terminal_str_len(&harness.name)))
        .max()
        .unwrap_or(0);
    let result_column_width = lines
        .iter()
        .map(|(_, res)| terminal_str_len(res))
        .max()
        .unwrap_or(0);

    let mut table_display =
        TableDisplay::new(&mut output, [status_column_width, result_column_width]);

    table_display.separator_line("┏", &[("━", "━"), ("━", "┓")])?;
    table_display.result_line("┃", &[(&harness.name, " "), ("", "┃")])?;
    table_display.separator_line("┣", &[("━", "┯"), ("━", "┫")])?;
    for (i, (status, result)) in lines.iter().enumerate() {
        table_display.result_line("┃", &[(*status, "│"), (result, "┃")])?;
        if i < lines.len() - 1 {
            table_display.separator_line("┠", &[("─", "┼"), ("─", "┨")])?;
        }
    }
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
        self.output.write("\n".as_bytes())?;
        Ok(())
    }

    fn result_line(&mut self, first: &str, columns: &[(&str, &str); N]) -> std::io::Result<()> {
        self.output.write(first.as_bytes())?;
        for ((column_value, end_sep), size) in columns.iter().zip(self.column_sizes.iter()) {
            self.output.write(column_value.as_bytes())?;
            for _ in terminal_str_len(column_value)..*size {
                self.output.write(" ".as_bytes())?;
            }
            self.output.write(end_sep.as_bytes())?;
        }
        self.output.write("\n".as_bytes())?;
        Ok(())
    }
}

fn terminal_str_len(input: &str) -> usize {
    let ansi_regex = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let stripped = ansi_regex.replace_all(input, "");
    stripped.chars().count()
}