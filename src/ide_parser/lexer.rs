use std::str::Chars;

use crate::completion::get_keywords;
use crate::ide_parser::cursor::Cursor;
use crate::ide_parser::syntax_kind::SyntaxKind;
use rowan::TextSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    pub kind: SyntaxKind,
    pub len: TextSize,
}

impl Token {
    pub fn new(kind: SyntaxKind, len: TextSize) -> Token {
        Token { kind, len }
    }
}

/// True if `c` is considered a whitespace according to Rust language definition.
/// See [Rust language reference](https://doc.rust-lang.org/reference/whitespace.html)
/// for definitions of these classes.
pub fn is_whitespace(c: char) -> bool {
    // This is Pattern_White_Space.
    //
    // Note that this set is stable (ie, it doesn't change with different
    // Unicode versions), so it's ok to just hard-code the values.

    match c {
        // Usual ASCII suspects
        | '\u{0009}' // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space
            => true,
        _ => false,
    }
}

impl Cursor<'_> {
    fn advance(&mut self) -> (SyntaxKind, usize) {
        let first_char = self.bump().unwrap();
        let syntax_kind = match first_char {
            c if is_whitespace(c) => self.consume_whitespace(),
            '0'..='9' => {
                if self.consume_if_next("x") && self.initial_len > 2 {
                    self.consume_hex_digits();
                    if self.len_consumed() == 2 {
                        SyntaxKind::Num_Lit
                    } else {
                        SyntaxKind::Address_Lit
                    }
                } else {
                    self.consume_decimal_number()
                }
            }
            'x' => {
                if self.consume_if_next("\"") {
                    // Search the current source line for a closing quote.
                    self.consume_while(|c| c != '"');
                    if self.is_eof() {
                        SyntaxKind::ByteString_Lit_Unterminated
                    } else {
                        SyntaxKind::ByteString_Lit
                    }
                } else {
                    self.consume_ident()
                }
            }
            'A'..='Z' | 'a'..='v' | 'y' | 'z' | '_' => self.consume_ident(),
            '&' => {
                if self.consume_if_next("mut ") {
                    SyntaxKind::AmpMut
                } else if self.consume_if_next("&") {
                    SyntaxKind::AmpAmp
                } else {
                    SyntaxKind::Amp
                }
            }
            '|' => {
                if self.consume_if_next("|") {
                    SyntaxKind::PipePipe
                } else {
                    SyntaxKind::Pipe
                }
            }
            '=' => {
                if self.consume_if_next("=>") {
                    SyntaxKind::EqualEqualGreater
                } else if self.consume_if_next("=") {
                    SyntaxKind::EqualEqual
                } else {
                    SyntaxKind::Equal
                }
            }
            '!' => {
                if self.consume_if_next("=") {
                    SyntaxKind::ExclaimEqual
                } else {
                    SyntaxKind::Exclaim
                }
            }
            '<' => {
                if self.consume_if_next("=") {
                    SyntaxKind::LessEqual
                } else if self.consume_if_next("<") {
                    SyntaxKind::LessLess
                } else {
                    SyntaxKind::Less
                }
            }
            '>' => {
                if self.consume_if_next("=") {
                    SyntaxKind::GreaterEqual
                } else if self.consume_if_next(">") {
                    SyntaxKind::GreaterGreater
                } else {
                    SyntaxKind::Greater
                }
            }
            ':' => {
                if self.consume_if_next(":") {
                    SyntaxKind::ColonColon
                } else {
                    SyntaxKind::Colon
                }
            }
            '%' => SyntaxKind::Percent,
            '(' => SyntaxKind::LParen,
            ')' => SyntaxKind::RParen,
            '[' => SyntaxKind::LBracket,
            ']' => SyntaxKind::RBracket,
            '*' => SyntaxKind::Star,
            '+' => SyntaxKind::Plus,
            ',' => SyntaxKind::Comma,
            '-' => SyntaxKind::Minus,
            '.' => {
                if self.consume_if_next(".") {
                    SyntaxKind::PeriodPeriod
                } else {
                    SyntaxKind::Period
                }
            }
            '/' => SyntaxKind::Slash,
            ';' => SyntaxKind::Semicolon,
            '^' => SyntaxKind::Caret,
            '{' => SyntaxKind::LBrace,
            '}' => SyntaxKind::RBrace,
            _ => unreachable!(),
        };
        (syntax_kind, self.len_consumed())
    }

    fn consume_whitespace(&mut self) -> SyntaxKind {
        self.consume_while(is_whitespace);
        SyntaxKind::Whitespace
    }

    // Return the length of the substring containing characters in [0-9a-fA-F].
    fn consume_hex_digits(&mut self) {
        let is_hex_digit = |c| matches!(c, 'a'..='f' | 'A'..='F' | '0'..='9');
        self.consume_while(is_hex_digit);
    }

    fn consume_decimal_number(&mut self) -> SyntaxKind {
        self.consume_while(|c| matches!(c, '0'..='9'));
        if self.is_next("u8") {
            self.bump_n_times(2);
            SyntaxKind::U8_Lit
        } else if self.is_next("u64") {
            self.bump_n_times(3);
            SyntaxKind::U64_Lit
        } else if self.is_next("u128") {
            self.bump_n_times(4);
            SyntaxKind::U128_Lit
        } else {
            SyntaxKind::Num_Lit
        }
    }

    fn consume_ident(&mut self) -> SyntaxKind {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'));
        SyntaxKind::Ident
    }
}

fn get_name_token_kind(name: &str) -> SyntaxKind {
    match name {
        "abort" => SyntaxKind::Abort_Kw,
        "acquires" => SyntaxKind::Acquires_Kw,
        "as" => SyntaxKind::As_Kw,
        "break" => SyntaxKind::Break_Kw,
        "continue" => SyntaxKind::Continue_Kw,
        "copy" => SyntaxKind::Copy_Kw,
        "copyable" => SyntaxKind::Copyable_Kw,
        "define" => SyntaxKind::Define_Kw,
        "else" => SyntaxKind::Else_Kw,
        "false" => SyntaxKind::False,
        "fun" => SyntaxKind::Fun_Kw,
        "if" => SyntaxKind::If_Kw,
        "invariant" => SyntaxKind::Invariant_Kw,
        "let" => SyntaxKind::Let_Kw,
        "loop" => SyntaxKind::Loop_Kw,
        "module" => SyntaxKind::Module_Kw,
        "move" => SyntaxKind::Move_Kw,
        "native" => SyntaxKind::Native_Kw,
        "public" => SyntaxKind::Public_Kw,
        "resource" => SyntaxKind::Resource_Kw,
        "return" => SyntaxKind::Return_Kw,
        "spec" => SyntaxKind::Spec_Kw,
        "struct" => SyntaxKind::Struct_Kw,
        "true" => SyntaxKind::True,
        "use" => SyntaxKind::Use_Kw,
        "while" => SyntaxKind::While_Kw,
        _ => SyntaxKind::Name_Lit,
    }
}

/// Parses the first token from the provided input string.
fn first_token(input: &str) -> (SyntaxKind, usize) {
    let (mut kind, len) = Cursor::new(input).advance();
    if kind == SyntaxKind::Ident {
        kind = get_name_token_kind(&input[..len]);
    }
    (kind, len)
}

/// Creates an iterator that produces tokens from the input string.
pub fn tokenize(mut input: &str) -> impl Iterator<Item = Token> + '_ {
    std::iter::from_fn(move || {
        if input.is_empty() {
            return None;
        }
        let (kind, len) = first_token(input);
        input = &input[len..];
        Some(Token::new(kind, (len as u32).into()))
    })
}
