#[macro_export]
macro_rules! fail {
    ($ret:expr) => {
        match $ret {
            Ok(ret) => ret,
            Err(err) => return $crate::Outcome::Failure(err),
        }
    };
}

#[macro_export]
macro_rules! success {
    ($expr: expr) => {
        $crate::Outcome::Success($expr)
    };
}

#[macro_export]
macro_rules! next {
    ($expr: expr) => {
        $crate::Outcome::Next($expr)
    };
}
