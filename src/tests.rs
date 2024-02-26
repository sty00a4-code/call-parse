use crate::{lexer::{LexError, Lexer, Token}, parser::{Parsable, Program}, position::Located};

#[test]
fn lexing_hello_world() -> Result<(), Located<LexError>> {
    let text = r#"print("hello");"#;
    let tokens = Lexer::new(text).lex()?.into_iter();
    dbg!(&tokens);
    let mut tokens = tokens.into_iter();
    assert_eq!(tokens.next().map(|token| token.unwrap()), Some(Token::Ident("print".to_string())));
    assert_eq!(tokens.next().map(|token| token.unwrap()), Some(Token::ParanLeft));
    assert_eq!(tokens.next().map(|token| token.unwrap()), Some(Token::String("hello".to_string())));
    assert_eq!(tokens.next().map(|token| token.unwrap()), Some(Token::ParanRight));
    assert_eq!(tokens.next().map(|token| token.unwrap()), Some(Token::Semicolon));
    Ok(())
}

#[test]
fn main() {
    let text = r#"a.1 = 2;"#;
    let tokens = Lexer::new(text).lex().unwrap();
    dbg!(&tokens);
    let ast = Program::parse(&mut tokens.into_iter().peekable()).unwrap();
    dbg!(&ast);
    // let ir = .unwrap();
    // dbg!(&ir);
}