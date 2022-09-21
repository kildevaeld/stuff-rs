use crate::{message::NEWLINE, Message, MessageBox};
use state::MutexState;

#[derive(Default)]
pub struct MessageList {
    retained: Vec<MessageBox>,
    mesg: Vec<MessageBox>,
}

impl MessageList {
    pub fn retain(&mut self, msg_len: usize) {
        let len = msg_len.min(self.mesg.len());
        self.retained.extend(self.mesg.drain(0..len));
    }

    pub fn update(&mut self, msg: impl IntoMessages) {
        self.mesg = msg.into_messages();
    }

    pub fn clear(&mut self, retained: bool) {
        self.mesg.clear();
        if retained {
            self.retained.clear();
        }
    }
}

impl Message for MessageList {
    fn tick(&mut self) {
        self.retained.tick();
        self.mesg.tick();
    }

    fn line_count(&self) -> usize {
        (self.retained.line_count() + self.mesg.line_count()).max(1)
    }

    fn message(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if !self.retained.is_empty() {
            self.retained.message(f)?;
            // write!(f, "{}", NEWLINE)?;
        }

        if !self.mesg.is_empty() {
            self.mesg.message(f)?;
        }

        // if self.retained.is_empty() && self.mesg.is_empty() {
        //     write!(f, "{}", NEWLINE)?;
        // }

        Ok(())
    }
}

pub trait IntoMessages {
    fn into_messages(self) -> Vec<MessageBox>;
}

macro_rules! into_messages {
    ($item: ident) => {
        impl<$item: Message + 'static> IntoMessages for ($item,) {
            fn into_messages(self) -> Vec<MessageBox> {
                vec![Box::new(self.0)]
            }
        }
    };
    ($first: ident, $($rest: ident),*) => {
        into_messages!($($rest),*);

        impl<$first: Message + 'static, $($rest: Message + 'static),*> IntoMessages for ($first,$($rest),*) {
            fn into_messages(self) -> Vec<MessageBox> {
                #[allow(non_snake_case)]
                let ($first, $($rest),*) = self;

                vec![Box::new($first),$(Box::new($rest)),*]
            }
        }
    };
}

into_messages!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);

impl IntoMessages for Vec<MessageBox> {
    fn into_messages(self) -> Vec<MessageBox> {
        self
    }
}
