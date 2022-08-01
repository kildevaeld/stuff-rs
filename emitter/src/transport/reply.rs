pub trait Reply {
    fn carry_on(self) -> bool;
}

impl Reply for bool {
    fn carry_on(self) -> bool {
        self
    }
}

impl Reply for () {
    fn carry_on(self) -> bool {
        true
    }
}

impl<S, E> Reply for Result<S, E> {
    fn carry_on(self) -> bool {
        match self {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
