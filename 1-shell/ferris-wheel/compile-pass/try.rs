// FIXME: Make me compile. Diff budget: 12 line additions and 2 characters.
struct ErrorA;
struct ErrorB;

enum Error {
    A(ErrorA),
    B(ErrorB)
}

fn do_a() -> Result<u16, ErrorA> {
    Err(ErrorA)
}

fn do_b() -> Result<u32, ErrorB> {
    Err(ErrorB)
}

impl From<ErrorA> for Error {
    fn from(e: ErrorA ) -> Error {
        Error::A(e)
    }
}
impl From<ErrorB> for Error {
    fn from(e: ErrorB ) -> Error {
        Error::B(e)
    }
}

fn do_both() -> Result<(u16, u32), Error> {
    Ok((do_a()?, do_b()?))
}

fn main() { }
