// FIXME: Make me pass! Diff budget: 30 lines.
use std::borrow::Borrow;

struct Builder {
    pub string: Option<String>,
    pub number: Option<usize>,
}

impl Builder {
    pub fn string<T>( & mut self, s: T ) -> & mut Self where T: Into<String> {
        self.string = Some(s.into());
        self
    }
    pub fn number( & mut self, n: usize ) -> & mut Self {
        self.number = Some(n);
        self
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            string: None,
            number: None,
        }
    }
}

impl ToString for Builder {
    fn to_string( & self ) -> String {
        let mut s = String::new();
        let mut space = false;
        match self.string {
            Some(ref x) => {
                s.push_str( x.as_str() );
                space = true;
            },
            _ => {}
        }
        match self.number {
            Some(x) => {
                if space {
                    s.push_str( " " );
                }
                let num = format!("{}", x );
                s.push_str( num.as_str()  );
            },
            _ => {}
        }
        s
    }
}

// Do not modify this function.
fn main() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");

    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default()
        .string("heap!".to_owned())
        .to_string();

    assert_eq!(c, "heap!");
}
