use std::{
    iter::Peekable,
    num::{ParseFloatError, ParseIntError},
    str::Chars,
};

use crate::{
    position::{Located, Position},
    Switch,
};

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    pub text: Peekable<Chars<'a>>,
    pub ln: usize,
    pub col: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Integer(i64),
    Decimal(f64),
    String(String),
    ParanLeft,
    ParanRight,
    BracketLeft,
    BracketRight,
    BraceLeft,
    BraceRight,
    Equal,
    Semicolon,
    Dot,
}
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    BadCharacter(char),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    ExpectedEscapeCharacter,
    UnclosedString,
}
impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text: text.chars().peekable(),
            ln: 0,
            col: 0,
        }
    }
    pub fn lex(&mut self) -> Result<Vec<Located<Token>>, Located<LexError>> {
        let mut tokens = vec![];
        while let Some(token) = self.next().switch()? {
            tokens.push(token);
        }
        Ok(tokens)
    }
    pub fn advance(&mut self) -> Option<char> {
        let c = self.text.next();
        if c == Some('\n') {
            self.ln += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        c
    }
    pub fn skip_whitespace(&mut self) -> Option<()> {
        while let Some(c) = self.text.peek().copied() {
            if !c.is_ascii_whitespace() {
                break;
            }
            self.advance()?;
        }
        Some(())
    }
    pub fn pos(&self) -> Position {
        Position::new(self.ln..self.ln, self.col..self.col + 1)
    }
}
impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Located<Token>, Located<LexError>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace()?;
        while self.text.peek().copied() == Some('#') {
            while let Some(c) = self.text.peek().copied() {
                if c == '\n' {
                    break;
                }
                self.advance()?;
            }
            self.advance()?;
            self.skip_whitespace()?;
        }
        let mut pos = self.pos();
        let c = self.advance()?;
        match c {
            '(' => Some(Ok(Located::new(Token::ParanLeft, pos))),
            ')' => Some(Ok(Located::new(Token::ParanRight, pos))),
            '[' => Some(Ok(Located::new(Token::BracketLeft, pos))),
            ']' => Some(Ok(Located::new(Token::BracketRight, pos))),
            '{' => Some(Ok(Located::new(Token::BraceLeft, pos))),
            '}' => Some(Ok(Located::new(Token::BraceRight, pos))),
            '=' => Some(Ok(Located::new(Token::Equal, pos))),
            ';' => Some(Ok(Located::new(Token::Semicolon, pos))),
            '.' => Some(Ok(Located::new(Token::Dot, pos))),
            end_c if end_c == '"' || end_c == '\'' => {
                let mut string = String::new();
                while let Some(c) = self.text.peek().copied() {
                    if c == end_c {
                        break;
                    }
                    string.push(match c {
                        '\\' => {
                            self.advance()?;
                            let Some(c) = self.advance() else {
                                return Some(Err(Located::new(
                                    LexError::ExpectedEscapeCharacter,
                                    self.pos(),
                                )));
                            };
                            match c {
                                'n' => '\n',
                                't' => '\t',
                                'r' => '\r',
                                c if c.is_ascii_digit() => {
                                    let mut pos = self.pos();
                                    let mut number = String::from(c);
                                    while let Some(c) = self.text.peek().copied() {
                                        if !c.is_ascii_digit() {
                                            break;
                                        }
                                        number.push(c);
                                        pos.extend(&self.pos());
                                        self.advance();
                                    }
                                    match number
                                        .parse::<u8>()
                                        .map_err(LexError::ParseIntError)
                                        .map_err(|err| Located::new(err, pos))
                                    {
                                        Ok(value) => value as char,
                                        Err(err) => return Some(Err(err)),
                                    }
                                }
                                c => c,
                            }
                        }
                        c => c,
                    });
                    self.advance();
                }
                pos.extend(&self.pos());
                if self.text.next() != Some(end_c) {
                    return Some(Err(Located::new(LexError::UnclosedString, pos)));
                }
                Some(Ok(Located::new(Token::String(string), pos)))
            }
            c if c.is_ascii_digit() => {
                let mut number = String::from(c);
                while let Some(c) = self.text.peek().copied() {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    number.push(c);
                    pos.extend(&self.pos());
                    self.advance();
                }
                if self.text.peek().copied() == Some('.') {
                    number.push('.');
                    pos.extend(&self.pos());
                    self.advance();
                    while let Some(c) = self.text.peek().copied() {
                        if !c.is_ascii_digit() {
                            break;
                        }
                        number.push(c);
                        pos.extend(&self.pos());
                        self.advance();
                    }
                    Some(Ok(Located::new(
                        Token::Decimal(
                            match number
                                .parse()
                                .map_err(LexError::ParseFloatError)
                                .map_err(|err| Located::new(err, pos.clone()))
                            {
                                Ok(value) => value,
                                Err(err) => return Some(Err(err)),
                            },
                        ),
                        pos,
                    )))
                } else {
                    Some(Ok(Located::new(
                        Token::Integer(
                            match number
                                .parse()
                                .map_err(LexError::ParseIntError)
                                .map_err(|err| Located::new(err, pos.clone()))
                            {
                                Ok(value) => value,
                                Err(err) => return Some(Err(err)),
                            },
                        ),
                        pos,
                    )))
                }
            }
            c if c.is_ascii_alphanumeric() => {
                let mut ident = String::from(c);
                while let Some(c) = self.text.peek().copied() {
                    if !c.is_ascii_alphanumeric() {
                        break;
                    }
                    ident.push(c);
                    pos.extend(&self.pos());
                    self.advance();
                }
                Some(Ok(Located::new(Token::Ident(ident), pos)))
            }
            c => Some(Err(Located::new(LexError::BadCharacter(c), pos))),
        }
    }
}
