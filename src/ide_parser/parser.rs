use rowan::{GreenNode, GreenNodeBuilder};
use serde::export::fmt::Debug;
use serde::export::Formatter;

use crate::ide_parser::lexer;
use crate::ide_parser::syntax_kind::SyntaxKind;
use crate::ide_parser::token_source::TextTokenSource;
use crate::ide_parser::tree::SyntaxNode;

pub struct Parse {
    pub green: GreenNode,
    pub errors: Vec<String>,
}

impl Parse {
    fn to_syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
}

pub struct Parser<'i> {
    token_source: TextTokenSource<'i>,
    builder: GreenNodeBuilder<'i>,
}

impl<'i> Parser<'i> {
    fn new(token_source: TextTokenSource) -> Parser {
        Parser {
            token_source,
            builder: GreenNodeBuilder::new(),
        }
    }

    fn parse_token(&mut self, token_kind: SyntaxKind) -> Result<(), String> {
        if self.token_source.current() == token_kind {
            self.builder
                .token(token_kind.into(), self.token_source.current_text().into());
            self.token_source.bump();
            Ok(())
        } else {
            Err(format!("Invalid token {:?}", token_kind))
        }
    }

    fn parse_name(&mut self) {
        self.parse_token(SyntaxKind::Name_Lit);
    }

    fn parse_address(&mut self) {}

    fn parse_module_ident(&mut self) {
        self.builder.start_node(SyntaxKind::ModuleIdent.into());
        self.parse_token(SyntaxKind::Address_Lit);
        self.parse_token(SyntaxKind::ColonColon);
        self.parse_token(SyntaxKind::Name_Lit);
        self.builder.finish_node();
    }

    fn parse_use(&mut self) {
        self.builder.start_node(SyntaxKind::Use.into());
        self.parse_token(SyntaxKind::Use_Kw);
        self.parse_module_ident();
        self.parse_token(SyntaxKind::Semicolon);
        self.builder.finish_node();
    }

    fn parse(mut self) -> Parse {
        self.builder.start_node(SyntaxKind::File.into());
        loop {
            let token = self.token_source.current();
            match token {
                SyntaxKind::EOF => {
                    break;
                }
                SyntaxKind::Use_Kw => {
                    self.parse_use();
                }
                _ => unreachable!("unknown token {:?}", token),
            }
        }
        while self.token_source.current() != SyntaxKind::EOF {
            if self.token_source.current() == SyntaxKind::Use {
                self.parse_use();
            }
        }
        self.builder.finish_node();
        let green = self.builder.finish();
        Parse {
            green,
            errors: vec![],
        }
    }
}

pub fn parse(text: &str) -> Parse {
    let raw_tokens: Vec<_> = lexer::tokenize(text).collect();
    let token_source = TextTokenSource::new(text, &raw_tokens);
    Parser::new(token_source).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let green = parse("use 0x0::Transaction;").to_syntax_node();
        dbg!(green);
    }
}
