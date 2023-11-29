use lox::Reporter;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process;

fn main() {
    println!("Hello, Lox!");

    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => {
            eprintln!("Usage: {} [script]", args[0]);
            process::exit(64);
        }
    }
}

struct MainReporter {
    has_errors: bool,
}
impl MainReporter {
    fn reset(&mut self) {
        self.has_errors = false;
    }
}
impl lox::Reporter for MainReporter {
    fn add_diagnostic(
        &mut self,
        start: &lox::FileLocation,
        end: &lox::FileLocation,
        message: &str,
    ) {
        self.has_errors = true;
        println!(
            "Diagnostic: [{0}:{1} {2}:{3}] {4}",
            start.line_number, start.line_offset, end.line_number, end.line_offset, message
        );
    }

    fn add_message(&mut self, message: &str) {
        println!("Message: {message}");
    }

    fn has_diagnostics(&self) -> bool {
        self.has_errors
    }
}

fn run_prompt() {
    let mut reporter = MainReporter { has_errors: false };
    loop {
        print!("> ");
        if io::stdout().flush().is_err() {
            return;
        }
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Err(_) => break,
            Ok(_) => {
                let trimmed_line = line.trim();
                if trimmed_line.is_empty() {
                    break;
                }
                reporter.reset();
                lox::run(&mut reporter, trimmed_line);
            }
        }
    }
    println!("done");
}

fn run_file(filepath: &str) {
    let contents = fs::read_to_string(filepath);
    if let Err(e) = contents {
        eprintln!("{e}");
        return;
    }
    let mut reporter = MainReporter { has_errors: false };
    lox::run(&mut reporter, &contents.unwrap());
    if reporter.has_diagnostics() {
        process::exit(70);
    }
}
