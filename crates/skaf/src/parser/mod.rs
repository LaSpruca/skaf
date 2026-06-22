use lex::{Lexer, LexicalToken, LexicalTokenVariant};
use std::{collections::HashMap, hash::Hash, iter::Peekable};

pub mod lex;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub line: usize,
    pub col: usize,
    pub value: String,
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Identifier {}

impl Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct Value {
    pub line: usize,
    pub col: usize,
    pub value: ValueVariant,
}

#[derive(Clone, Debug)]
pub enum ValueVariant {
    String(String),
    Invoke(String, Vec<Value>),
    Query(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Object {
    pub line: usize,
    pub col: usize,
    pub tag: Identifier,
    pub name: Identifier,
    pub data: HashMap<Identifier, Value>,
}

impl Object {
    pub fn get_tag(&self) -> &str {
        &self.tag.value
    }

    pub fn get_name(&self) -> &str {
        &self.name.value
    }

    pub fn get_field(&self, key: impl ToString) -> Option<&Value> {
        self.data.get(&Identifier {
            line: 0,
            col: 0,
            value: key.to_string(),
        })
    }
}

fn parse_value(tokens: &mut Peekable<Lexer>) -> Option<Value> {
    match tokens.peek()?.variant {
        LexicalTokenVariant::String(_) => {
            let Some(LexicalToken {
                line,
                col,
                variant: LexicalTokenVariant::String(value),
            }) = tokens.next()
            else {
                unreachable!()
            };

            Some(Value {
                line,
                col,
                value: ValueVariant::String(value),
            })
        }
        LexicalTokenVariant::Query => {
            let Some(LexicalToken { line, col, .. }) = tokens.next() else {
                unreachable!()
            };
            let mut query_path = Vec::new();

            while let Some(identifier) = parse_identifier(tokens) {
                query_path.push(identifier.value);
                if matches!(
                    tokens.peek(),
                    Some(LexicalToken {
                        variant: LexicalTokenVariant::Dot,
                        ..
                    })
                ) {
                    _ = tokens.next();
                }
            }

            Some(Value {
                line,
                col,
                value: ValueVariant::Query(query_path),
            })
        }
        LexicalTokenVariant::Dot => {
            let Some(LexicalToken { line, col, .. }) = tokens.next() else {
                unreachable!()
            };

            let name = parse_identifier(tokens)?;
            let mut values = Vec::new();
            while tokens
                .peek()
                .is_some_and(|x| x.variant != LexicalTokenVariant::NewLine)
            {
                values.push(parse_value(tokens)?);
            }

            Some(Value {
                line,
                col,
                value: ValueVariant::Invoke(name.value, values),
            })
        }
        _ => None,
    }
}

fn parse_identifier(tokens: &mut Peekable<Lexer>) -> Option<Identifier> {
    let LexicalToken {
        variant: peeked, ..
    } = tokens.peek()?;

    match peeked {
        LexicalTokenVariant::Identifier(_) => {
            let Some(LexicalToken {
                line,
                col,
                variant: LexicalTokenVariant::Identifier(value),
            }) = tokens.next()
            else {
                unreachable!();
            };

            Some(Identifier { line, col, value })
        }
        LexicalTokenVariant::LeftCurlyBrace => {
            let Some(LexicalToken { line, col, .. }) = tokens.next() else {
                unreachable!()
            };

            let mut ident = String::new();

            loop {
                let LexicalToken { variant, .. } = tokens.next()?;

                if variant == LexicalTokenVariant::RightCurlyBrace {
                    let variant = tokens.peek().map(|x| &x.variant);
                    if variant == Some(&LexicalTokenVariant::RightCurlyBrace) {
                        _ = tokens.next();
                        ident.push('}');
                    } else {
                        return Some(Identifier {
                            line,
                            col,
                            value: ident,
                        });
                    }
                } else {
                    ident.push_str(&(variant.to_string()));
                }
            }
        }
        _ => None,
    }
}

fn parse_object(tokens: &mut Peekable<Lexer>) -> Option<Object> {
    let LexicalToken { line, col, .. } = tokens.peek()?;
    let (line, col) = (*line, *col);

    let tag = parse_identifier(tokens)?;
    let name = parse_identifier(tokens)?;

    if tokens
        .peek()
        .is_none_or(|x| x.variant != LexicalTokenVariant::LeftCurlyBrace)
    {
        return None;
    }
    _ = tokens.next();

    if tokens
        .peek()
        .is_some_and(|x| x.variant == LexicalTokenVariant::NewLine)
    {
        _ = tokens.next();
    }

    let mut data = HashMap::new();

    while tokens
        .peek()
        .is_some_and(|x| x.variant != LexicalTokenVariant::RightCurlyBrace)
    {
        let name = parse_identifier(tokens)?;

        if !matches!(
            tokens.next(),
            Some(LexicalToken {
                variant: LexicalTokenVariant::Colon,
                ..
            })
        ) {
            return None;
        }

        let value = parse_value(tokens)?;

        if matches!(
            tokens.peek(),
            Some(LexicalToken {
                variant: LexicalTokenVariant::NewLine,
                ..
            })
        ) {
            _ = tokens.next();
        }

        data.insert(name, value);
    }

    _ = tokens.next();

    Some(Object {
        line,
        col,
        name,
        tag,
        data,
    })
}

pub fn parse(tokens: Lexer) -> Vec<Object> {
    let mut tokens = tokens.peekable();
    let mut objects = vec![];

    loop {
        if let Some(object) = parse_object(&mut tokens) {
            objects.push(object);
            continue;
        }

        if let Some(LexicalToken {
            variant: LexicalTokenVariant::NewLine,
            ..
        }) = tokens.next()
        {
            continue;
        }

        break;
    }

    objects
}
