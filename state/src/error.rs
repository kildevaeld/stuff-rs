use core::fmt;

#[derive(Debug)]
pub enum StateError {
    Upgrade,
    Empty,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid state")
    }
}
