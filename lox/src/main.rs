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

fn run_prompt() {
    let reporter =
        lox::reporter::Reporter::build(Box::new(report_message), Box::new(report_diagnostic));
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
                lox::run(&reporter, trimmed_line);
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
    let reporter =
        lox::reporter::Reporter::build(Box::new(report_message), Box::new(report_diagnostic));
    lox::run(&reporter, &contents.unwrap());
}

fn report_message(message: &str) {
    println!("Message: {message}");
}

fn report_diagnostic(line: u32, column: u32, message: &str) {
    println!("Diagnostic: [{line}:{column}]{message}");
}
