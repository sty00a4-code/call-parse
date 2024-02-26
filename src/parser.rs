use crate::{
    lexer::Token,
    position::{Located, Position},
};
use std::{iter::Peekable, vec::IntoIter};

pub type Parser = Peekable<IntoIter<Located<Token>>>;
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedToken(Token),
    ExpectedToken {
        expected: Token,
        got: Token,
    },
    ExpectedTokens {
        expected: &'static [Token],
        got: Token,
    },
}
pub trait Parsable
where
    Self: Sized,
{
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<ParseError>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program(Vec<Located<Statement>>);
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assign {
        path: Located<Path>,
        expr: Located<Expression>,
    },
    Call {
        head: Located<Path>,
        args: Vec<Located<Expression>>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Atom(Atom),
    Call {
        head: Box<Located<Self>>,
        args: Vec<Located<Self>>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Path(Path),
    Integer(i64),
    Decimal(f64),
    String(String),
    Expression(Box<Located<Expression>>),
    List(Vec<Located<Expression>>),
    Map(Vec<(Located<String>, Located<Expression>)>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Path {
    Ident(String),
    Field {
        head: Box<Located<Self>>,
        field: Box<Located<Atom>>,
    },
}

impl Parsable for Program {
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<ParseError>> {
        let mut stats = vec![];
        let mut pos = Position::default();
        while parser.peek().is_some() {
            let stat = Statement::parse(parser)?;
            pos.extend(&stat.pos);
            stats.push(stat);
        }
        Ok(Located::new(Self(stats), pos))
    }
}
impl Parsable for Statement {
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<ParseError>> {
        let path = Path::parse(parser)?;
        let mut pos = path.pos.clone();
        let Some(Located {
            value: c_token,
            pos: c_pos,
        }) = parser.next()
        else {
            return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
        };
        let stat = match c_token {
            Token::Equal => {
                let expr = Expression::parse(parser)?;
                pos.extend(&expr.pos);
                Located::new(Self::Assign { path, expr }, pos)
            }
            Token::ParanLeft => {
                let mut args = vec![];
                while let Some(Located {
                    value: c_token,
                    pos: _,
                }) = parser.peek()
                {
                    if c_token == &Token::ParanRight {
                        break;
                    }
                    args.push(Expression::parse(parser)?);
                }
                let Some(Located {
                    value: c_token,
                    pos: c_pos,
                }) = parser.next()
                else {
                    return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
                };
                if c_token != Token::ParanRight {
                    return Err(Located::new(
                        ParseError::ExpectedToken {
                            expected: Token::ParanRight,
                            got: c_token,
                        },
                        c_pos,
                    ));
                }
                pos.extend(&c_pos);
                Located::new(Self::Call { head: path, args }, pos)
            }
            c_token => {
                return Err(Located::new(
                    ParseError::ExpectedTokens {
                        expected: &[Token::Equal, Token::ParanLeft],
                        got: c_token,
                    },
                    c_pos,
                ))
            }
        };
        let Some(Located {
            value: c_token,
            pos: c_pos,
        }) = parser.next()
        else {
            return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
        };
        if c_token != Token::Semicolon {
            return Err(Located::new(
                ParseError::ExpectedToken {
                    expected: Token::Semicolon,
                    got: c_token,
                },
                c_pos,
            ));
        }
        Ok(stat)
    }
}
impl Parsable for Expression {
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<ParseError>> {
        let mut head = Atom::parse(parser)?.map(Self::Atom);
        while let Some(Located {
            value: c_token,
            pos: _,
        }) = parser.peek()
        {
            head = match c_token {
                Token::ParanLeft => {
                    parser.next();
                    let mut pos = head.pos.clone();
                    let mut args = vec![];
                    while let Some(Located {
                        value: c_token,
                        pos: _,
                    }) = parser.peek()
                    {
                        if c_token == &Token::ParanRight {
                            break;
                        }
                        args.push(Expression::parse(parser)?);
                    }
                    let Some(Located {
                        value: c_token,
                        pos: c_pos,
                    }) = parser.next()
                    else {
                        return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
                    };
                    if c_token != Token::ParanRight {
                        return Err(Located::new(
                            ParseError::ExpectedToken {
                                expected: Token::ParanRight,
                                got: c_token,
                            },
                            c_pos,
                        ));
                    }
                    pos.extend(&c_pos);
                    Located::new(
                        Self::Call {
                            head: Box::new(head),
                            args,
                        },
                        pos,
                    )
                }
                _ => break,
            };
        }
        Ok(head)
    }
}
impl Parsable for Atom {
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<ParseError>> {
        if matches!(
            parser.peek(),
            Some(Located {
                value: Token::Ident(_),
                pos: _
            })
        ) {
            return Ok(Path::parse(parser)?.map(Self::Path));
        }
        let Some(Located {
            value: token,
            mut pos,
        }) = parser.next()
        else {
            return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
        };
        match token {
            Token::Integer(value) => Ok(Located::new(Self::Integer(value), pos)),
            Token::Decimal(value) => Ok(Located::new(Self::Decimal(value), pos)),
            Token::String(value) => Ok(Located::new(Self::String(value), pos)),
            Token::ParanLeft => {
                let expr = Expression::parse(parser)?;
                let Some(Located {
                    value: c_token,
                    pos: c_pos,
                }) = parser.next()
                else {
                    return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
                };
                if c_token != Token::ParanRight {
                    return Err(Located::new(
                        ParseError::ExpectedToken {
                            expected: Token::ParanRight,
                            got: c_token,
                        },
                        c_pos,
                    ));
                }
                pos.extend(&c_pos);
                Ok(Located::new(Self::Expression(Box::new(expr)), pos))
            }
            Token::BracketLeft => {
                let mut exprs = vec![];
                while let Some(Located {
                    value: c_token,
                    pos: _,
                }) = parser.peek()
                {
                    if c_token == &Token::BracketRight {
                        break;
                    }
                    exprs.push(Expression::parse(parser)?);
                }
                let Some(Located {
                    value: c_token,
                    pos: c_pos,
                }) = parser.next()
                else {
                    return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
                };
                if c_token != Token::BracketRight {
                    return Err(Located::new(
                        ParseError::ExpectedToken {
                            expected: Token::BracketRight,
                            got: c_token,
                        },
                        c_pos,
                    ));
                }
                pos.extend(&c_pos);
                Ok(Located::new(Self::List(exprs), pos))
            }
            token => Err(Located::new(ParseError::UnexpectedToken(token), pos)),
        }
    }
}
impl Parsable for Path {
    fn parse(parser: &mut Parser) -> Result<Located<Self>, Located<ParseError>> {
        let mut head = Self::ident(parser)?;
        while let Some(Located {
            value: c_token,
            pos: _,
        }) = parser.peek()
        {
            head = match c_token {
                Token::Dot => {
                    parser.next();
                    let mut pos = head.pos.clone();
                    let field = if matches!(parser.peek(), Some(Located { value: Token::Ident(_), pos: _ })) {
                        Self::ident(parser)?.map(Atom::Path)
                    } else {
                        Atom::parse(parser)?
                    };
                    pos.extend(&field.pos);
                    Located::new(
                        Self::Field {
                            head: Box::new(head),
                            field: Box::new(field),
                        },
                        pos,
                    )
                }
                _ => break,
            };
        }
        Ok(head)
    }
}
impl Path {
    fn ident(parser: &mut Parser) -> Result<Located<Self>, Located<ParseError>> {
        let Some(Located {
            value: c_token,
            pos: c_pos,
        }) = parser.next()
        else {
            return Err(Located::new(ParseError::UnexpectedEOF, Position::default()));
        };
        if let Token::Ident(ident) = c_token {
            Ok(Located::new(Self::Ident(ident), c_pos))
        } else {
            Err(Located::new(
                ParseError::ExpectedToken {
                    expected: Token::BracketRight,
                    got: c_token,
                },
                c_pos,
            ))
        }
    }
}
