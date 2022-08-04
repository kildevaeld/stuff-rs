use std::fmt;

pub type MessageBox = Box<dyn Message>;

pub trait Message: Send {
    fn line_count(&self) -> usize;
    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn update(&mut self) {}
}

impl Message for MessageBox {
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

// pub trait MessageArea {
//     fn update_from<M: Message>(&self, idx: usize, msg: M);
//     fn retain(&self, count: usize);
//     fn len(&self) -> usize;
// }

// pub trait MessageAreaExt: MessageArea {
//     fn update<M: Message>(&self, msg: M) {
//         self.update_from(0, msg)
//     }
// }
