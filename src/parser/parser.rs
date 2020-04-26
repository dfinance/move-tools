use rowan::{GreenNode, GreenNodeBuilder};
use serde::export::fmt::Debug;
use serde::export::Formatter;

use crate::parser::lexer;
use crate::parser::syntax_kind::SyntaxKind;
use crate::parser::tokens::Tokens;
use crate::parser::tree::SyntaxNode;

pub struct Parse {
    pub green: GreenNode,
    pub errors: Vec<String>,
}

impl Parse {
    fn to_syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
}

pub fn parse(text: &str) -> Parse {
    let raw_tokens: Vec<_> = lexer::tokenize(text).collect();
    let token_source = Tokens::new(text, &raw_tokens);
    Parser::new(token_source).parse()
}

pub struct Parser<'i> {
    tokens: Tokens<'i>,
    builder: GreenNodeBuilder<'i>,
}

impl<'i> Parser<'i> {
    fn new(tokens: Tokens) -> Parser {
        Parser {
            tokens,
            builder: GreenNodeBuilder::new(),
        }
    }

    fn parse_token(&mut self, token_kind: SyntaxKind) -> Result<(), String> {
        if self.tokens.current() == token_kind {
            self.builder
                .token(token_kind.into(), self.tokens.current_text().into());
            self.tokens.bump();
            Ok(())
        } else {
            Err(format!("Invalid token {:?}", token_kind))
        }
    }

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
            let token = self.tokens.current();
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
        while self.tokens.current() != SyntaxKind::EOF {
            if self.tokens.current() == SyntaxKind::Use {
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
