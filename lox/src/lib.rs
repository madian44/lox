pub mod reporter;

pub fn run(report: &reporter::Reporter, s: &str) {
    report.add_message(&format!("success; scanning: |{s}|"));
    report.add_diagnostic(1, 1, &format!("success; on a line: |{s}|"));
}
