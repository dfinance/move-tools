use rowan::{GreenNodeBuilder, TextRange, TextSize};

use crate::ide_parser::lexer;
use crate::ide_parser::lexer::Token;
use crate::ide_parser::syntax_kind::SyntaxKind;

pub fn syntax_kind_at(pos: usize, tokens: &[Token]) -> SyntaxKind {
    tokens.get(pos).map(|t| t.kind).unwrap_or(SyntaxKind::EOF)
}

pub struct TextTokenSource<'i> {
    text: &'i str,
    start_offsets: Vec<TextSize>,
    tokens: Vec<Token>,
    // current token kind and current position
    curr: (SyntaxKind, usize),
}

impl<'t> TextTokenSource<'t> {
    pub fn new(text: &'t str, raw_tokens: &'t [Token]) -> TextTokenSource<'t> {
        let mut tokens = vec![];
        let mut start_offsets = vec![];
        let mut last_token_offset = TextSize::zero();
        for &token in raw_tokens.iter() {
            if !token.kind.is_trivia() {
                tokens.push(token);
                start_offsets.push(last_token_offset);
            }
            last_token_offset += token.len;
        }
        let first_kind = syntax_kind_at(0, &tokens);
        TextTokenSource {
            text,
            start_offsets,
            tokens,
            curr: (first_kind, 0),
        }
    }

    pub fn current(&self) -> SyntaxKind {
        self.curr.0
    }

    pub fn current_text(&self) -> &str {
        let pos = self.curr.1;
        let start = self.start_offsets.get(pos).unwrap();
        let end = self.start_offsets.get(pos).unwrap_or(start);
        &self.text[TextRange::new(*start, *end)]
    }

    pub fn bump(&mut self) {
        if self.curr.0 == SyntaxKind::EOF {
            return;
        }
        let pos = self.curr.1 + 1;
        self.curr = (syntax_kind_at(pos, &self.tokens), pos);
    }
}
