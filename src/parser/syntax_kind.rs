#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum SyntaxKind {
    EOF,
    Whitespace,
    Ident,
    // literals
    Address_Lit,
    Num_Lit,
    U8_Lit,
    U64_Lit,
    U128_Lit,
    ByteString_Lit,
    ByteString_Lit_Unterminated,
    Name_Lit,
    // operators
    Exclaim,
    ExclaimEqual,
    Percent,
    Amp,
    AmpAmp,
    AmpMut,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Star,
    Plus,
    Comma,
    Minus,
    Period,
    PeriodPeriod,
    Slash,
    Colon,
    ColonColon,
    Semicolon,
    Less,
    LessEqual,
    LessLess,
    Equal,
    EqualEqual,
    EqualEqualGreater,
    Greater,
    GreaterEqual,
    GreaterGreater,
    Caret,
    LBrace,
    Pipe,
    PipePipe,
    RBrace,
    // bools
    False,
    True,
    // keywords
    Abort_Kw,
    Acquires_Kw,
    As_Kw,
    Break_Kw,
    Continue_Kw,
    Copy_Kw,
    Copyable_Kw,
    Define_Kw,
    Else_Kw,
    If_Kw,
    Invariant_Kw,
    Let_Kw,
    Loop_Kw,
    Module_Kw,
    Move_Kw,
    Native_Kw,
    Public_Kw,
    Resource_Kw,
    Return_Kw,
    Spec_Kw,
    Struct_Kw,
    Use_Kw,
    While_Kw,
    Fun_Kw,
    // composites
    Name,
    Use,
    ModuleIdent,
    FunctionDef,
    SpecDef,
    // globals
    AddressDef,
    ModuleDef,
    File,
}

impl From<u16> for SyntaxKind {
    fn from(d: u16) -> SyntaxKind {
        unsafe { std::mem::transmute::<u16, SyntaxKind>(d) }
    }
}

impl From<SyntaxKind> for u16 {
    fn from(k: SyntaxKind) -> u16 {
        k as u16
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::Whitespace)
    }
}
