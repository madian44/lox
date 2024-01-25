mod common;

use lox::Reporter;

#[test]
fn test_statements() {
    let mut reporter = common::TestReporter::new();
    let tests = vec![
        //("var a = clock(); print a;", "[print] 10"),
        (
            "print \"hello,\" + \" world\";",
            vec!["[print] \"hello, world\""],
        ),
        ("print 10 + 10;", vec!["[print] 20"]),
        ("print 10 - 5;", vec!["[print] 5"]),
        ("print 10 > 5;", vec!["[print] true"]),
        ("print 5 > 5;", vec!["[print] false"]),
        ("print 5 >= 5;", vec!["[print] true"]),
        ("print \"a string\";", vec!["[print] \"a string\""]),
        ("print 10.5 ;", vec!["[print] 10.5"]),
        ("print true ;", vec!["[print] true"]),
        ("print false ;", vec!["[print] false"]),
        ("print nil ;", vec!["[print] nil"]),
        ("print !true ;", vec!["[print] false"]),
        ("print !!true ;", vec!["[print] true"]),
        ("print -3.45 ;", vec!["[print] -3.45"]),
        ("print 5 == 5 ;", vec!["[print] true"]),
        ("print 5 == 4 ;", vec!["[print] false"]),
        ("print \"hello\" == \"hello\";", vec!["[print] true"]),
        ("print \"hello\" == \"world\";", vec!["[print] false"]),
        ("print 4 / 4;", vec!["[print] 1"]),
        ("print 6 / 4;", vec!["[print] 1.5"]),
        ("print 2 * 2;", vec!["[print] 4"]),
        ("print 5 < 5;", vec!["[print] false"]),
        ("print 5 <= 5;", vec!["[print] true"]),
        ("print (5 <= 5);", vec!["[print] true"]),
        ("print !\"string\";", vec!["[print] false"]),
        ("print !!\"string\";", vec!["[print] true"]),
        ("print ((10 - 5) + 1) / (2 * 3);", vec!["[print] 1"]),
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
        (
            "class Thing {
                getCallback() {
                    this.message = \"Hello\";
                    fun localFunction() {
                        print this.message;
                    }
                    return localFunction;
                }
            }
            var callback = Thing().getCallback();
            callback();",
            vec!["[print] \"Hello\""],
        ),
        (
            "class Thing {
                init(message) {
                    this.message = message;
                }
                get() {
                    return \"Hello \" + this.message;
                }
            }
            var val = Thing(\"Bob\");
            var i = val.get();
            print i;",
            vec!["[print] \"Hello Bob\""],
        ),
        (
            "class Thing {
                init(message) {
                    this.message = message;
                    return;
                }
            }
            var val = Thing(\"Bob\");
            var i = val.init(\"John\");
            print i.message;",
            vec!["[print] \"John\""],
        ),
        (
            "class Doughnut {
                cook() {
                    print \"Fry until golden brown\";
                }
            }
            class BostonCream < Doughnut {}
            BostonCream().cook();",
            vec!["[print] \"Fry until golden brown\""],
        ),
        (
            "class Doughnut {
                cook() {
                    print \"Fry until golden brown\";
                }
            }
            class BostonCream < Doughnut {
                cook() {
                    super.cook();
                    print \"Pipe full of custard and coat with chocolate\";
                }
            }
            BostonCream().cook();",
            vec![
                "[print] \"Fry until golden brown\"",
                "[print] \"Pipe full of custard and coat with chocolate\"",
            ],
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
    let mut reporter = common::TestReporter::new();
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
        (
            "class Example { init(param){} } var e = Example();",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 40,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 47,
                },
                message: "Expected 1 arguments but got 0".to_string(),
            },
        ),
        (
            "class Example { init(param){} } var e = Example(1, 2);",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 40,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 47,
                },
                message: "Expected 1 arguments but got 2".to_string(),
            },
        ),
        (
            "class Example { error(param){this.error();} } Example().error(1);",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 29,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 39,
                },
                message: "Expected 1 arguments but got 0".to_string(),
            },
        ),
        (
            "class Example {} Example().error;",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 17,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 26,
                },
                message: "Undefined property 'error'".to_string(),
            },
        ),
        (
            "var value = 1; print value.field;",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 21,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 26,
                },
                message: "Only instances have fields".to_string(),
            },
        ),
        (
            "var value = 1; value.field = 2;",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 15,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 20,
                },
                message: "Only instances have fields".to_string(),
            },
        ),
        (
            "fun error() { print this.fred; } error();",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 20,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 24,
                },
                message: "Cannot use 'this' outside of a class".to_string(),
            },
        ),
        (
            "class Example{ init() { this.dest = this.src; }} var e = Example();",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 36,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 40,
                },
                message: "Undefined property 'src'".to_string(),
            },
        ),
        (
            "var NotAClass = \"I'm not a class\"; class Example < NotAClass {}",
            common::Diagnostic {
                start: lox::FileLocation {
                    line_number: 0,
                    line_offset: 51,
                },
                end: lox::FileLocation {
                    line_number: 0,
                    line_offset: 60,
                },
                message: "Superclass must be a class".to_string(),
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
