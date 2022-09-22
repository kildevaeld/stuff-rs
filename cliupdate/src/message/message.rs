use core::fmt;

pub type MessageBox = Box<dyn Message>;

pub trait Message {
    fn line_count(&self) -> usize;
    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn tick(&mut self) {}
}

impl Message for MessageBox {
    fn line_count(&self) -> usize {
        (&**self).line_count()
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&**self).message(f)
    }

    fn tick(&mut self) {
        (&mut **self).tick()
    }
}
