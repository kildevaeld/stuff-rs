use crate::message::Message;
use core::fmt;
use crossterm::style;
use std::{borrow::Cow, fmt::Write};

const NEWLINE: char = '\n';

impl<'a> Message for () {
    fn line_count(&self) -> usize {
        1
    }

    fn message(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<'a> Message for &'a str {
    fn line_count(&self) -> usize {
        (*self).lines().count()
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <&'a str as fmt::Display>::fmt(&self, f)
    }
}

impl<'a> Message for Cow<'a, str> {
    fn line_count(&self) -> usize {
        (*self).lines().count()
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Message for char {
    fn line_count(&self) -> usize {
        1
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(*self)
    }
}

impl<'a, T> Message for &'a T
where
    T: Message,
    &'a T: Send,
{
    fn line_count(&self) -> usize {
        (&**self).line_count()
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).message(f)
    }
}

impl<'a, T> Message for &'a mut T
where
    T: Message,
{
    fn line_count(&self) -> usize {
        (&**self).line_count()
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&**self).message(f)
    }

    fn update(&mut self) {
        (&mut **self).update()
    }
}

impl Message for String {
    fn line_count(&self) -> usize {
        self.lines().count()
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <String as fmt::Display>::fmt(self, f)
    }
}

impl<M: Message> Message for Vec<M> {
    fn line_count(&self) -> usize {
        self.iter().fold(0, |prev, cur| prev + cur.line_count())
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.line_count();
        for (count, next) in self.iter().enumerate() {
            next.message(f)?;
            if count + 1 != len {
                f.write_char(NEWLINE)?;
            }
        }

        Ok(())
    }

    fn update(&mut self) {
        for next in self.iter_mut() {
            next.update()
        }
    }
}

impl<M: Message> Message for [M] {
    fn line_count(&self) -> usize {
        self.iter().fold(0, |prev, cur| prev + cur.line_count())
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.line_count();
        for (count, next) in self.iter().enumerate() {
            next.message(f)?;
            if count + 1 != len {
                f.write_char(NEWLINE)?;
            }
        }

        Ok(())
    }

    fn update(&mut self) {
        for next in self.iter_mut() {
            next.update()
        }
    }
}

impl<S: Message + fmt::Display> Message for style::StyledContent<S> {
    fn line_count(&self) -> usize {
        self.content().line_count()
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <style::StyledContent<S> as fmt::Display>::fmt(self, f)
    }
}

macro_rules! array {
    ($($count: literal)*) => {
        $(
            impl<M: Message> Message for [M; $count] {
                fn line_count(&self) -> usize {
                    self.iter().fold(0, |prev, cur| prev + cur.line_count())
                }

                fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    let len = self.line_count();
                    for (count, next) in self.iter().enumerate() {
                        next.message(f)?;
                        if count + 1 != len {
                            f.write_char(NEWLINE)?;
                        }
                    }

                    Ok(())
                }

                fn update(&mut self) {
                    for next in self.iter_mut() {
                        next.update()
                    }
                }
            }
        )*
    };
}

array!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28);

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! tuples {
    ($first: ident) => {

    };
    ($first: ident $($item: ident)*) => {
        tuples!($($item)*);
        #[allow(non_snake_case)]
        impl<$first: Message, $($item: Message),*> Message for ($first, $($item),*)  {
            fn line_count(&self) -> usize {
                let ($first, $($item),*) = self;
                $first.line_count() +
                $(
                    $item.line_count() +
                )+ 0
            }

            fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let len = count!($($item)*) + 1;
                let mut i = 0;
                let ($first, $($item),*) = self;
                $first.message(f)?;
                i += 1;
                if i != len {
                    f.write_char(NEWLINE)?;
                }

                $(
                    $item.message(f)?;
                    i += 1;
                    if i != len {
                        f.write_char(NEWLINE)?;
                    }

                )+
                Ok(())
            }

            fn update(&mut self) {
                let ($first,$($item),*) = self;
                $first.update();
                $(
                    $item.update();
                )+
            }
        }
    };
}

tuples!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16);
