use crate::location;
use crate::reporter;
use crate::token;
use std::iter::Peekable;
use std::str::CharIndices;

pub fn scan_tokens<'a>(
    reporter: &mut dyn reporter::Reporter,
    source: &'a str,
) -> Vec<token::Token<'a>> {
    let mut scanner = Scanner::build();
    scanner.scan(reporter, source)
}

struct Scanner {
    token_start_line_number: u32,
    token_start_line_offset: u32,
    current_line_number: u32,
    current_line_offset: u32,
    start_of_token: usize,
    current_end_of_token: usize,
}

impl Scanner {
    fn build() -> Self {
        Scanner {
            token_start_line_number: 0,
            token_start_line_offset: 0,
            current_line_number: 0,
            current_line_offset: 0,
            start_of_token: 0,
            current_end_of_token: 0,
        }
    }

    fn scan<'a>(
        &mut self,
        reporter: &mut dyn reporter::Reporter,
        source: &'a str,
    ) -> Vec<token::Token<'a>> {
        let mut tokens = Vec::new();
        let mut char_indices = source.char_indices().peekable();
        loop {
            self.token_start_line_offset = self.current_line_offset;
            self.token_start_line_number = self.current_line_number;
            if let Some((i, c)) = self.advance(&mut char_indices) {
                self.start_of_token = i;
                self.current_end_of_token = i;
                if let Some(token) = self.parse_character(reporter, source, &mut char_indices, c) {
                    tokens.push(token);
                }
            } else {
                break;
            }
        }
        tokens
    }

    fn parse_character<'a>(
        &mut self,
        reporter: &mut dyn reporter::Reporter,
        source: &'a str,
        char_indices: &mut Peekable<CharIndices>,
        c: char,
    ) -> Option<token::Token<'a>> {
        match c {
            '(' => Some(self.build_token(token::TokenType::LeftParen, source)),
            ')' => Some(self.build_token(token::TokenType::RightParen, source)),
            '{' => Some(self.build_token(token::TokenType::LeftBrace, source)),
            '}' => Some(self.build_token(token::TokenType::RightBrace, source)),
            ',' => Some(self.build_token(token::TokenType::Comma, source)),
            '.' => Some(self.build_token(token::TokenType::Dot, source)),
            '-' => Some(self.build_token(token::TokenType::Minus, source)),
            '+' => Some(self.build_token(token::TokenType::Plus, source)),
            ';' => Some(self.build_token(token::TokenType::Semicolon, source)),
            '*' => Some(self.build_token(token::TokenType::Star, source)),
            '!' => {
                if self.peek(char_indices, '=') {
                    Some(self.build_token(token::TokenType::BangEqual, source))
                } else {
                    Some(self.build_token(token::TokenType::Bang, source))
                }
            }
            '=' => {
                if self.peek(char_indices, '=') {
                    Some(self.build_token(token::TokenType::EqualEqual, source))
                } else {
                    Some(self.build_token(token::TokenType::Equal, source))
                }
            }
            '<' => {
                if self.peek(char_indices, '=') {
                    Some(self.build_token(token::TokenType::LessEqual, source))
                } else {
                    Some(self.build_token(token::TokenType::Less, source))
                }
            }
            '>' => {
                if self.peek(char_indices, '=') {
                    Some(self.build_token(token::TokenType::GreaterEqual, source))
                } else {
                    Some(self.build_token(token::TokenType::Greater, source))
                }
            }
            '/' => {
                if self.peek(char_indices, '/') {
                    self.consume_line(char_indices);
                    None
                } else {
                    Some(self.build_token(token::TokenType::Slash, source))
                }
            }
            '"' => self.build_string(reporter, source, char_indices),
            ' ' | '\r' | '\t' => None,
            '\n' => {
                self.current_line_number += 1;
                self.current_line_offset = 0;
                None
            }
            _ => {
                if c.is_ascii_digit() {
                    self.build_number(source, char_indices)
                } else {
                    reporter.add_diagnostic(
                        &location::FileLocation::new(
                            self.token_start_line_number,
                            self.token_start_line_offset,
                        ),
                        &location::FileLocation::new(
                            self.current_line_number,
                            self.current_line_offset,
                        ),
                        "Unexpected character",
                    );
                    None
                }
            }
        }
    }

    fn advance(&mut self, char_indices: &mut Peekable<CharIndices>) -> Option<(usize, char)> {
        if let Some((i, c)) = char_indices.next() {
            self.current_line_offset += 1;
            self.current_end_of_token = i;
            Some((i, c))
        } else {
            None
        }
    }

    fn peek(&mut self, char_indices: &mut Peekable<CharIndices>, char_to_peek: char) -> bool {
        if let Some((_i, c)) = char_indices.peek() {
            if *c == char_to_peek {
                self.advance(char_indices);
                return true;
            }
        }
        false
    }

    fn consume_line(&mut self, char_indices: &mut Peekable<CharIndices>) {
        while let Some((_i, c)) = char_indices.peek() {
            if *c == '\n' {
                break;
            }
            self.advance(char_indices);
        }
    }

    fn build_string<'a>(
        &mut self,
        reporter: &mut dyn reporter::Reporter,
        source: &'a str,
        char_indices: &mut Peekable<CharIndices>,
    ) -> Option<token::Token<'a>> {
        loop {
            if let Some((_i, c)) = self.advance(char_indices) {
                if c == '\n' {
                    self.current_line_offset = 0;
                    self.current_line_number += 1;
                }
                if c == '"' {
                    return Some(self.build_string_token(source));
                }
            } else {
                reporter.add_diagnostic(
                    &location::FileLocation::new(
                        self.token_start_line_number,
                        self.token_start_line_offset,
                    ),
                    &location::FileLocation::new(
                        self.current_line_number,
                        self.current_line_offset,
                    ),
                    "Unterminated string",
                );
                return None;
            }
        }
    }

    fn build_number<'a>(
        &mut self,
        source: &'a str,
        char_indices: &mut Peekable<CharIndices>,
    ) -> Option<token::Token<'a>> {
        self.scan_digits(char_indices);
        if let Some((_i, c)) = char_indices.peek() {
            if *c == '.' {
                self.advance(char_indices);
            } else {
                return Some(self.build_number_token(source));
            }
        }
        self.scan_digits(char_indices);
        Some(self.build_number_token(source))
    }

    fn scan_digits(&mut self, char_indices: &mut Peekable<CharIndices>) {
        while let Some((_i, c)) = char_indices.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            self.advance(char_indices);
        }
    }

    fn build_token<'a>(&self, token_type: token::TokenType, source: &'a str) -> token::Token<'a> {
        let lexeme = &source[self.start_of_token..=self.current_end_of_token];
        token::Token::new(
            token_type,
            lexeme,
            location::FileLocation::new(self.token_start_line_number, self.token_start_line_offset),
            location::FileLocation::new(self.current_line_number, self.current_line_offset),
            token::Literal::None,
        )
    }

    fn build_string_token<'a>(&self, source: &'a str) -> token::Token<'a> {
        let lexeme = &source[self.start_of_token..=self.current_end_of_token];
        token::Token::new(
            token::TokenType::String,
            lexeme,
            location::FileLocation::new(self.token_start_line_number, self.token_start_line_offset),
            location::FileLocation::new(self.current_line_number, self.current_line_offset),
            token::Literal::String(
                &source[(self.start_of_token + 1)..=(self.current_end_of_token - 1)],
            ),
        )
    }

    fn build_number_token<'a>(&self, source: &'a str) -> token::Token<'a> {
        let lexeme = &source[self.start_of_token..=self.current_end_of_token];
        token::Token::new(
            token::TokenType::Number,
            lexeme,
            location::FileLocation::new(self.token_start_line_number, self.token_start_line_offset),
            location::FileLocation::new(self.current_line_number, self.current_line_offset),
            token::Literal::Number(lexeme.parse().unwrap()),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::Token;
    use reporter::Reporter;

    #[derive(Debug, PartialEq)]
    struct Diagnostic {
        start: location::FileLocation,
        end: location::FileLocation,
        message: String,
    }

    struct Message {
        message: String,
    }

    struct TestReporter {
        diagnostics: Vec<Diagnostic>,
        messages: Vec<Message>,
    }

    impl TestReporter {
        fn build() -> Self {
            TestReporter {
                diagnostics: Vec::new(),
                messages: Vec::new(),
            }
        }

        fn has_messages(&self) -> bool {
            !self.messages.is_empty()
        }

        fn reset(&mut self) {
            self.messages.clear();
            self.diagnostics.clear();
        }
    }

    impl reporter::Reporter for TestReporter {
        fn add_diagnostic(
            &mut self,
            start: &location::FileLocation,
            end: &location::FileLocation,
            message: &str,
        ) {
            self.diagnostics.push(Diagnostic {
                start: start.clone(),
                end: end.clone(),
                message: message.to_string(),
            });
        }

        fn add_message(&mut self, message: &str) {
            self.messages.push(Message {
                message: message.to_string(),
            });
        }
        fn has_diagnostics(&self) -> bool {
            !self.diagnostics.is_empty()
        }
    }

    fn execute_tests(reporter: &mut TestReporter, tests: &Vec<(&str, Vec<Token>)>) {
        for (source, expected_tokens) in tests {
            reporter.reset();
            let tokens = scan_tokens(reporter, source);
            assert!(!reporter.has_messages(), "Unexpected messages reported");
            assert!(
                !reporter.has_diagnostics(),
                "Unexpected diagnostics reported"
            );
            assert_eq!(
                tokens.len(),
                expected_tokens.len(),
                "Incorrect tokens returned"
            );
            for (i, expected_token) in expected_tokens.iter().enumerate() {
                let token = &tokens[i];
                assert_eq!(*token, *expected_token, "Unexpected token returned");
            }
        }
    }

    #[test]
    fn single_token() {
        let mut reporter = TestReporter::build();

        let tests = vec![
            (
                "(",
                vec![token::Token::new(
                    token::TokenType::LeftParen,
                    "(",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 1),
                    token::Literal::None,
                )],
            ),
            (
                " ( ",
                vec![token::Token::new(
                    token::TokenType::LeftParen,
                    "(",
                    location::FileLocation::new(0, 1),
                    location::FileLocation::new(0, 2),
                    token::Literal::None,
                )],
            ),
            (
                " ! ",
                vec![token::Token::new(
                    token::TokenType::Bang,
                    "!",
                    location::FileLocation::new(0, 1),
                    location::FileLocation::new(0, 2),
                    token::Literal::None,
                )],
            ),
            (
                " != ",
                vec![token::Token::new(
                    token::TokenType::BangEqual,
                    "!=",
                    location::FileLocation::new(0, 1),
                    location::FileLocation::new(0, 3),
                    token::Literal::None,
                )],
            ),
        ];

        execute_tests(&mut reporter, &tests);
    }

    #[test]
    fn multiple_tokens() {
        let mut reporter = TestReporter::build();

        let tests = vec![(
            "( )",
            vec![
                token::Token::new(
                    token::TokenType::LeftParen,
                    "(",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 1),
                    token::Literal::None,
                ),
                token::Token::new(
                    token::TokenType::RightParen,
                    ")",
                    location::FileLocation::new(0, 2),
                    location::FileLocation::new(0, 3),
                    token::Literal::None,
                ),
            ],
        )];

        execute_tests(&mut reporter, &tests);
    }

    #[test]
    fn multiline_tests() {
        let mut reporter = TestReporter::build();

        let tests = vec![(
            "(\n)",
            vec![
                token::Token::new(
                    token::TokenType::LeftParen,
                    "(",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 1),
                    token::Literal::None,
                ),
                token::Token::new(
                    token::TokenType::RightParen,
                    ")",
                    location::FileLocation::new(1, 0),
                    location::FileLocation::new(1, 1),
                    token::Literal::None,
                ),
            ],
        )];

        execute_tests(&mut reporter, &tests);
    }

    #[test]
    fn comment_tests() {
        let mut reporter = TestReporter::build();

        let tests = vec![(
            "(// a comment\n)",
            vec![
                token::Token::new(
                    token::TokenType::LeftParen,
                    "(",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 1),
                    token::Literal::None,
                ),
                token::Token::new(
                    token::TokenType::RightParen,
                    ")",
                    location::FileLocation::new(1, 0),
                    location::FileLocation::new(1, 1),
                    token::Literal::None,
                ),
            ],
        )];

        execute_tests(&mut reporter, &tests);
    }

    #[test]
    fn string_tests() {
        let mut reporter = TestReporter::build();

        let tests = vec![
            (
                "\"a string\"",
                vec![token::Token::new(
                    token::TokenType::String,
                    "\"a string\"",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 10),
                    token::Literal::String("a string"),
                )],
            ),
            (
                "\"a string\nwith a new line\"",
                vec![token::Token::new(
                    token::TokenType::String,
                    "\"a string\nwith a new line\"",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(1, 16),
                    token::Literal::String("a string\nwith a new line"),
                )],
            ),
        ];

        execute_tests(&mut reporter, &tests);
    }

    #[test]
    fn number_tests() {
        let mut reporter = TestReporter::build();

        let tests = vec![
            (
                "10",
                vec![token::Token::new(
                    token::TokenType::Number,
                    "10",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 2),
                    token::Literal::Number(10f64),
                )],
            ),
            (
                "10.",
                vec![token::Token::new(
                    token::TokenType::Number,
                    "10.",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 3),
                    token::Literal::Number(10f64),
                )],
            ),
            (
                "10.1",
                vec![token::Token::new(
                    token::TokenType::Number,
                    "10.1",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 4),
                    token::Literal::Number(10.1f64),
                )],
            ),
        ];

        execute_tests(&mut reporter, &tests);
    }

    #[test]
    fn diagnostic_tests() {
        let mut reporter = TestReporter::build();

        let tests = vec![
            (
                "\"a string",
                vec![Diagnostic {
                    start: location::FileLocation::new(0, 0),
                    end: location::FileLocation::new(0, 9),
                    message: "Unterminated string".to_string(),
                }],
            ),
            (
                "\"a string\nplus",
                vec![Diagnostic {
                    start: location::FileLocation::new(0, 0),
                    end: location::FileLocation::new(1, 4),
                    message: "Unterminated string".to_string(),
                }],
            ),
            (
                " ~ ",
                vec![Diagnostic {
                    start: location::FileLocation::new(0, 1),
                    end: location::FileLocation::new(0, 2),
                    message: "Unexpected character".to_string(),
                }],
            ),
        ];

        for (source, expected_diagnostics) in tests {
            reporter.reset();
            let _tokens = scan_tokens(&mut reporter, source);
            assert!(!reporter.has_messages(), "Unexpected messages reported");
            assert!(
                reporter.has_diagnostics(),
                "Unexpectedly no diagnostics reported"
            );
            assert_eq!(
                reporter.diagnostics.len(),
                expected_diagnostics.len(),
                "Incorrect diagnostics returned"
            );
            for (i, expected_diagnostic) in expected_diagnostics.iter().enumerate() {
                let diagnostic = &reporter.diagnostics[i];
                assert_eq!(
                    *diagnostic, *expected_diagnostic,
                    "Unexpected diagnostic returned"
                );
            }
        }
    }
}
