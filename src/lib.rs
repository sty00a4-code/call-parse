#[cfg(test)]
mod tests;
pub mod position;
pub mod lexer;
pub mod parser;
pub mod ir;
pub mod compiler;

pub trait Switch {
    type Item;
    fn switch(self) -> Self::Item;
}
impl<T, E> Switch for Option<Result<T, E>> {
    type Item = Result<Option<T>, E>;
    fn switch(self) -> Self::Item {
        match self {
            Some(value) => match value {
                Ok(value) => Ok(Some(value)),
                Err(err) => Err(err)
            }
            None => Ok(None)
        }
    }
}
