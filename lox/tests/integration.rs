mod common;

use lox::Reporter;

#[test]
fn test_statements() {
    let mut reporter = common::TestReporter::build();
    let tests = vec![
        // ("var a = clock(); print a;", "[print] 10"),
        (
            "\"hello,\" + \" world\";",
            vec!["[interpreter] \"hello, world\""],
        ),
        ("10 + 10;", vec!["[interpreter] 20"]),
        ("10 - 5;", vec!["[interpreter] 5"]),
        ("10 > 5;", vec!["[interpreter] true"]),
        ("5 > 5;", vec!["[interpreter] false"]),
        ("5 >= 5;", vec!["[interpreter] true"]),
        ("\"a string\";", vec!["[interpreter] \"a string\""]),
        ("10.5 ;", vec!["[interpreter] 10.5"]),
        ("true ;", vec!["[interpreter] true"]),
        ("false ;", vec!["[interpreter] false"]),
        ("nil ;", vec!["[interpreter] nil"]),
        ("!true ;", vec!["[interpreter] false"]),
        ("!!true ;", vec!["[interpreter] true"]),
        ("-3.45 ;", vec!["[interpreter] -3.45"]),
        ("5 == 5 ;", vec!["[interpreter] true"]),
        ("5 == 4 ;", vec!["[interpreter] false"]),
        ("\"hello\" == \"hello\";", vec!["[interpreter] true"]),
        ("\"hello\" == \"world\";", vec!["[interpreter] false"]),
        ("4 / 4;", vec!["[interpreter] 1"]),
        ("6 / 4;", vec!["[interpreter] 1.5"]),
        ("2 * 2;", vec!["[interpreter] 4"]),
        ("5 < 5;", vec!["[interpreter] false"]),
        ("5 <= 5;", vec!["[interpreter] true"]),
        ("(5 <= 5);", vec!["[interpreter] true"]),
        ("!\"string\";", vec!["[interpreter] false"]),
        ("!!\"string\";", vec!["[interpreter] true"]),
        ("((10 - 5) + 1) / (2 * 3);", vec!["[interpreter] 1"]),
        ("print ((10 - 5) + 1) / (2 * 3);", vec!["[print] 1"]),
        ("var a = 1 ; { a = 2; print a;}", vec!["[print] 2"]),
        (
            "fun fib(n) {
                if( n<= 1) return n;
                return fib(n-2) + fib(n-1);
            }
            print fib(1);",
            vec!["[print] 1"],
        ),
        (
            "fun fib(n) {
                if( n<= 1) return n;
                return fib(n-2) + fib(n-1);
            }
            print fib(2);",
            vec!["[print] 1"],
        ),
        (
            "fun fib(n) {
                if( n<= 1) return n;
                return fib(n-2) + fib(n-1);
            }
            print fib(0);
            print fib(3);
            print fib(6);",
            vec!["[print] 0", "[print] 2", "[print] 8"],
        ),
        (
            "fun makeCounter() {
                var i = 0;
                fun count() {
                    i = i + 1;
                    print i;
                }
                return count;
            }
            var counter = makeCounter();
            counter();
            counter();",
            vec!["[print] 1", "[print] 2"],
        ),
        (
            "fun makeCounter() {
                var i = 0;
                fun count() {
                    i = i + 1;
                    print i;
                }
                return count;
            }
            var counter = makeCounter();
            var outer = \"outer str\";
            counter();
            print outer;",
            vec!["[print] 1", "[print] \"outer str\""],
        ),
    ];

    for (source, expected_messages) in tests {
        reporter.reset();
        lox::interpret(&reporter, source);
        if reporter.has_diagnostics() {
            println!("Unexpected errors for : {}", source,);
            reporter.print_contents();
            panic!("Unexpected errors");
        }
        for expected_message in expected_messages {
            if !reporter.has_message(expected_message) {
                println!("Missing message: {} != {}", source, expected_message);
                reporter.print_contents();
                panic!("Missing message");
            }
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
        (
            "{b = 1;}",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 1,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 6,
                },
                message: "Undefined variable 'b'".to_string(),
            },
        ),
        (
            "print a;",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 6,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 7,
                },
                message: "Undefined variable 'a'".to_string(),
            },
        ),
        (
            "\"totally not a function\"();",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 0,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 24,
                },
                message: "Can only call functions and classes".to_string(),
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
