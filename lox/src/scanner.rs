use crate::location;
use crate::reporter;
use crate::token;
use std::collections::LinkedList;
use std::iter::Peekable;
use std::str::CharIndices;

pub fn scan_tokens(
    reporter: &mut dyn reporter::Reporter,
    source: &str,
) -> LinkedList<token::Token> {
    let mut scanner = Scanner::build();
    scanner.scan(reporter, source)
}

struct Scanner<'k> {
    token_start_line_number: u32,
    token_start_line_offset: u32,
    current_line_number: u32,
    current_line_offset: u32,
    start_of_token: usize,
    current_end_of_token: usize,
    keywords: token::Keywords<'k>,
}

impl<'k> Scanner<'k> {
    fn build() -> Self {
        Scanner {
            token_start_line_number: 0,
            token_start_line_offset: 0,
            current_line_number: 0,
            current_line_offset: 0,
            start_of_token: 0,
            current_end_of_token: 0,
            keywords: token::Keywords::build(),
        }
    }

    fn scan(
        &mut self,
        reporter: &mut dyn reporter::Reporter,
        source: &str,
    ) -> LinkedList<token::Token> {
        let mut tokens = LinkedList::new();
        let mut char_indices = source.char_indices().peekable();
        loop {
            self.token_start_line_offset = self.current_line_offset;
            self.token_start_line_number = self.current_line_number;
            if let Some((i, c)) = self.advance(&mut char_indices) {
                self.start_of_token = i;
                self.current_end_of_token = i;
                if let Some(token) = self.parse_character(reporter, source, &mut char_indices, c) {
                    tokens.push_back(token);
                }
            } else {
                break;
            }
        }
        self.start_of_token = self.current_end_of_token;
        tokens.push_back(self.build_token(token::TokenType::Eof, source));
        tokens
    }

    fn parse_character(
        &mut self,
        reporter: &mut dyn reporter::Reporter,
        source: &str,
        char_indices: &mut Peekable<CharIndices>,
        c: char,
    ) -> Option<token::Token> {
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
                } else if c.is_alphabetic() || c == '_' {
                    self.build_identifier(source, char_indices)
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

    fn peek_next(&self, source: &str, offset: usize) -> Option<char> {
        if source.len() < offset + 1 {
            return None;
        }
        let next = &source[(offset + 1)..];
        next.chars().next()
    }

    fn build_token(&self, token_type: token::TokenType, source: &str) -> token::Token {
        let lexeme = &source[self.start_of_token..=self.current_end_of_token];
        token::Token::new(
            token_type,
            lexeme,
            location::FileLocation::new(self.token_start_line_number, self.token_start_line_offset),
            location::FileLocation::new(self.current_line_number, self.current_line_offset),
            token::Literal::None,
        )
    }

    fn build_string_token(&self, source: &str) -> token::Token {
        let lexeme = &source[self.start_of_token..=self.current_end_of_token];
        token::Token::new(
            token::TokenType::String,
            lexeme,
            location::FileLocation::new(self.token_start_line_number, self.token_start_line_offset),
            location::FileLocation::new(self.current_line_number, self.current_line_offset),
            token::Literal::String(
                source[(self.start_of_token + 1)..=(self.current_end_of_token - 1)].to_string(),
            ),
        )
    }

    fn build_number_token(&self, source: &str) -> token::Token {
        let lexeme = &source[self.start_of_token..=self.current_end_of_token];
        token::Token::new(
            token::TokenType::Number,
            lexeme,
            location::FileLocation::new(self.token_start_line_number, self.token_start_line_offset),
            location::FileLocation::new(self.current_line_number, self.current_line_offset),
            token::Literal::Number(lexeme.parse().unwrap()),
        )
    }

    fn consume_line(&mut self, char_indices: &mut Peekable<CharIndices>) {
        while let Some((_i, c)) = char_indices.peek() {
            if *c == '\n' {
                break;
            }
            self.advance(char_indices);
        }
    }

    fn build_string(
        &mut self,
        reporter: &mut dyn reporter::Reporter,
        source: &str,
        char_indices: &mut Peekable<CharIndices>,
    ) -> Option<token::Token> {
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

    fn build_number(
        &mut self,
        source: &str,
        char_indices: &mut Peekable<CharIndices>,
    ) -> Option<token::Token> {
        self.scan_digits(char_indices);
        if let Some((i, c)) = char_indices.peek() {
            if *c == '.' {
                let peeked_next_char = self.peek_next(source, *i);
                if peeked_next_char.is_none() {
                    return Some(self.build_number_token(source));
                }
                if !peeked_next_char.unwrap().is_ascii_digit() {
                    return Some(self.build_number_token(source));
                }
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

    fn build_identifier(
        &mut self,
        source: &str,
        char_indices: &mut Peekable<CharIndices>,
    ) -> Option<token::Token> {
        while let Some((_i, c)) = char_indices.peek() {
            if c.is_alphabetic() || c.is_ascii_digit() || *c == '_' {
                self.advance(char_indices);
            } else {
                break;
            }
        }
        let token = self.build_token(token::TokenType::Identifier, source);
        if let Some(identifier_token) = self.keywords.get_keyword(&token.lexeme) {
            let literal = token::get_keyword_literal(&identifier_token);
            return Some(token::Token {
                token_type: identifier_token,
                literal,
                ..token
            });
        }
        Some(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::reporter::test::Diagnostic;
    use crate::reporter::test::TestReporter;
    use crate::reporter::Reporter;
    use crate::token::Token;

    fn execute_tests(reporter: &mut TestReporter, tests: &Vec<(&str, Vec<Token>)>) {
        for (source, expected_tokens) in tests {
            reporter.reset();
            let mut tokens = scan_tokens(reporter, source);
            assert!(!reporter.has_messages(), "Unexpected messages reported");
            assert!(
                !reporter.has_diagnostics(),
                "Unexpected diagnostics reported"
            );
            assert_eq!(
                tokens.len(),
                expected_tokens.len() + 1, // always expect end of file
                "Incorrect tokens returned"
            );

            let last_token = tokens.back().unwrap();
            assert_eq!(last_token.token_type, token::TokenType::Eof);

            for expected_token in expected_tokens.iter() {
                let token = tokens.pop_front();
                assert_eq!(token.unwrap(), *expected_token, "Unexpected token returned");
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
                    token::Literal::String("a string".to_string()),
                )],
            ),
            (
                "\"a string\nwith a new line\"",
                vec![token::Token::new(
                    token::TokenType::String,
                    "\"a string\nwith a new line\"",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(1, 16),
                    token::Literal::String("a string\nwith a new line".to_string()),
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
                vec![
                    token::Token::new(
                        token::TokenType::Number,
                        "10",
                        location::FileLocation::new(0, 0),
                        location::FileLocation::new(0, 2),
                        token::Literal::Number(10f64),
                    ),
                    token::Token::new(
                        token::TokenType::Dot,
                        ".",
                        location::FileLocation::new(0, 2),
                        location::FileLocation::new(0, 3),
                        token::Literal::None,
                    ),
                ],
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
    fn identifier_tests() {
        let mut reporter = TestReporter::build();

        let tests = vec![
            (
                "andy",
                vec![token::Token::new(
                    token::TokenType::Identifier,
                    "andy",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 4),
                    token::Literal::None,
                )],
            ),
            (
                "and",
                vec![token::Token::new(
                    token::TokenType::And,
                    "and",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 3),
                    token::Literal::None,
                )],
            ),
            (
                "with_underscore",
                vec![token::Token::new(
                    token::TokenType::Identifier,
                    "with_underscore",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 15),
                    token::Literal::None,
                )],
            ),
            (
                "with123digits",
                vec![token::Token::new(
                    token::TokenType::Identifier,
                    "with123digits",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 13),
                    token::Literal::None,
                )],
            ),
            (
                "_SHOUT",
                vec![token::Token::new(
                    token::TokenType::Identifier,
                    "_SHOUT",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 6),
                    token::Literal::None,
                )],
            ),
            (
                "false",
                vec![token::Token::new(
                    token::TokenType::False,
                    "false",
                    location::FileLocation::new(0, 0),
                    location::FileLocation::new(0, 5),
                    token::Literal::False,
                )],
            ),
        ];

        execute_tests(&mut reporter, &tests);
    }

    #[test]
    fn adjacent_token_tests() {
        let mut reporter = TestReporter::build();

        let tests = vec![
            (
                "()",
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
                        location::FileLocation::new(0, 1),
                        location::FileLocation::new(0, 2),
                        token::Literal::None,
                    ),
                ],
            ),
            (
                "10.!",
                vec![
                    token::Token::new(
                        token::TokenType::Number,
                        "10",
                        location::FileLocation::new(0, 0),
                        location::FileLocation::new(0, 2),
                        token::Literal::Number(10f64),
                    ),
                    token::Token::new(
                        token::TokenType::Dot,
                        ".",
                        location::FileLocation::new(0, 2),
                        location::FileLocation::new(0, 3),
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Bang,
                        "!",
                        location::FileLocation::new(0, 3),
                        location::FileLocation::new(0, 4),
                        token::Literal::None,
                    ),
                ],
            ),
            (
                ".\"10.1\"10.1",
                vec![
                    token::Token::new(
                        token::TokenType::Dot,
                        ".",
                        location::FileLocation::new(0, 0),
                        location::FileLocation::new(0, 1),
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::String,
                        "\"10.1\"",
                        location::FileLocation::new(0, 1),
                        location::FileLocation::new(0, 7),
                        token::Literal::String("10.1".to_string()),
                    ),
                    token::Token::new(
                        token::TokenType::Number,
                        "10.1",
                        location::FileLocation::new(0, 7),
                        location::FileLocation::new(0, 11),
                        token::Literal::Number(10.1f64),
                    ),
                ],
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
