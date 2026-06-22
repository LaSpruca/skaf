use std::{
    char,
    iter::{Enumerate, Peekable},
    str::Chars,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LexicalToken {
    pub line: usize,
    pub col: usize,
    pub variant: LexicalTokenVariant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexicalTokenVariant {
    Identifier(String),
    String(String),
    LeftCurlyBrace,
    RightCurlyBrace,
    Colon,
    NewLine,
    Dot,
    Query,
    LeftSquareBrace,
    RightSquareBrace,
    LeftPeren,
    RightPeren,
}

impl ToString for LexicalTokenVariant {
    fn to_string(&self) -> String {
        match self {
            Self::Identifier(val) => val.clone(),
            // TODO: String restoration
            Self::String(val) => format!("\"{val}\""),
            Self::LeftCurlyBrace => "{".into(),
            Self::RightCurlyBrace => "}".into(),
            Self::Colon => ":".into(),
            Self::NewLine => "\n".into(),
            Self::Query => "?".into(),
            Self::Dot => ".".into(),
            Self::LeftPeren => "(".into(),
            Self::RightPeren => ")".into(),
            Self::LeftSquareBrace => "[".into(),
            Self::RightSquareBrace => "]".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    col: usize,
    line: usize,
    buffer: &'a str,
    iter: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            col: 0,
            line: 0,
            buffer: source,
            iter: source.chars().enumerate().peekable(),
        }
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let (idx, next) = self.iter.next()?;

        if next == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }

        Some((idx, next))
    }

    /// returns the index of the last character matching this pattern
    fn advance_until<T>(&mut self, cond: T) -> usize
    where
        T: Fn((usize, char)) -> bool,
    {
        loop {
            match self.iter.peek() {
                Some((index, val)) if !cond((*index, *val)) => return index - 1,
                Some(_) => _ = self.advance(),
                None => return self.buffer.len() - 1,
            }
        }
    }

    fn read_string(&mut self) -> Option<LexicalToken> {
        let Self { col, line, .. } = *self;
        let mut buf = String::new();

        while let Some((_, val)) = self.iter.peek() {
            if *val == '\\' {
                _ = self.advance();

                let (_, val) = self.advance()?;

                match val {
                    'n' => buf.push('\n'),
                    '\\' => buf.push('\\'),
                    '\'' => buf.push('\''),
                    _ => {
                        buf.push('\\');
                        buf.push(val)
                    }
                }

                continue;
            }

            if *val == '"' {
                _ = self.advance();
                return Some(LexicalToken {
                    line,
                    col,
                    variant: LexicalTokenVariant::String(buf),
                });
            }

            buf.push(*val);
            _ = self.advance();
        }

        // TODO: Error
        None
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = LexicalToken;

    fn next(&mut self) -> Option<Self::Item> {
        let (idex, char) = self.advance()?;

        match char {
            'a'..='z' | 'A'..='Z' | '_' => Some(LexicalToken {
                col: self.col,
                line: self.line,
                variant: LexicalTokenVariant::Identifier(
                    self.buffer[idex..=self.advance_until(|(_, x)| {
                        !x.is_whitespace()
                            && x != ':'
                            && x != '{'
                            && x != '}'
                            && x != '?'
                            && x != '.'
                            && x != '('
                            && x != ')'
                            && x != '['
                            && x != ']'
                    })]
                        .to_string(),
                ),
            }),
            '{' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::LeftCurlyBrace,
            }),
            '}' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::RightCurlyBrace,
            }),
            '"' => self.read_string(),
            ':' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::Colon,
            }),
            '\\' if self.iter.peek().is_some_and(|(_, x)| *x == '\n') => {
                _ = self.advance();
                self.next()
            }
            '\n' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::NewLine,
            }),
            '?' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::Query,
            }),
            '.' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::Dot,
            }),
            '(' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::LeftPeren,
            }),
            ')' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::RightPeren,
            }),
            '[' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::LeftSquareBrace,
            }),
            ']' => Some(LexicalToken {
                line: self.line,
                col: self.col,
                variant: LexicalTokenVariant::RightSquareBrace,
            }),
            _ => self.next(),
        }
    }
}
