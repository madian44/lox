mod common;

use lox::Reporter;

#[test]
fn test_expression() {
    let mut reporter = common::TestReporter::build();
    let tests = vec![
        ("\"hello,\" + \" world\";", "[interpreter] hello, world"),
        ("10 + 10;", "[interpreter] 20"),
        ("10 - 5;", "[interpreter] 5"),
        ("10 > 5;", "[interpreter] true"),
        ("5 > 5;", "[interpreter] false"),
        ("5 >= 5;", "[interpreter] true"),
        ("\"a string\";", "[interpreter] a string"),
        ("10.5 ;", "[interpreter] 10.5"),
        ("true ;", "[interpreter] true"),
        ("false ;", "[interpreter] false"),
        ("nil ;", "[interpreter] nil"),
        ("!true ;", "[interpreter] false"),
        ("!!true ;", "[interpreter] true"),
        ("-3.45 ;", "[interpreter] -3.45"),
        ("5 == 5 ;", "[interpreter] true"),
        ("5 == 4 ;", "[interpreter] false"),
        ("\"hello\" == \"hello\";", "[interpreter] true"),
        ("\"hello\" == \"world\";", "[interpreter] false"),
        ("4 / 4;", "[interpreter] 1"),
        ("6 / 4;", "[interpreter] 1.5"),
        ("2 * 2;", "[interpreter] 4"),
        ("5 < 5;", "[interpreter] false"),
        ("5 <= 5;", "[interpreter] true"),
        ("(5 <= 5);", "[interpreter] true"),
        ("!\"string\";", "[interpreter] false"),
        ("!!\"string\";", "[interpreter] true"),
        ("((10 - 5) + 1) / (2 * 3);", "[interpreter] 1"),
        ("print ((10 - 5) + 1) / (2 * 3);", "[print] 1"),
    ];

    for (expression, expected_message) in tests {
        reporter.reset();
        lox::interpret(&reporter, expression);
        if !reporter.has_message(expected_message) || reporter.has_diagnostics() {
            println!("Unexpected errors: {} != {}", expression, expected_message);
            reporter.print_contents();
            panic!("Unexpected errors");
        }
    }
}

#[test]
fn test_failures() {
    let mut reporter = common::TestReporter::build();
    let tests = vec![
        (
            "\"hello,\" + 10 ;",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 0,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 13,
                },
                message: "Operands must be two numbers or two strings".to_string(),
            },
        ),
        (
            "\"hello\" - true ; ",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 0,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 14,
                },
                message: "Operand should be a number".to_string(),
            },
        ),
        (
            "((10 - 5) + 1) / (2 * \"fred\") ;",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 18,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 28,
                },
                message: "Operand should be a number".to_string(),
            },
        ),
    ];

    for (expression, expected_diagnostic) in &tests {
        reporter.reset();
        lox::interpret(&reporter, expression);
        if !reporter.has_diagnostic(expected_diagnostic) {
            println!(
                "Missing diagnostic: {} != {:?}",
                expression, expected_diagnostic
            );
            reporter.print_contents();
            panic!("Missing diagnostic");
        }
    }
}
