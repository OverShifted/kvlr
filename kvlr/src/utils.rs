use std::io::{BufRead, BufReader, Cursor};

pub fn array_buf_read(array: &[u8]) -> impl BufRead + '_ {
    BufReader::new(Cursor::new(array))
}

pub trait Unfold {
    type Output;
    fn unfold(self) -> Self::Output;
}

impl<T, E0, E1> Unfold for Result<Result<T, E1>, E0>
where
    E0: Into<anyhow::Error>,
    E1: Into<anyhow::Error>,
{
    type Output = anyhow::Result<T>;
    fn unfold(self) -> Self::Output {
        match self {
            Ok(t) => match t {
                Ok(t) => Ok(t),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }
    }
}
