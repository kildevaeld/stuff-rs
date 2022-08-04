mod manager;
pub mod message;

pub use self::{
    manager::Manager,
    message::{Message, MessageBox, MessageExt, SharedMessage},
};
