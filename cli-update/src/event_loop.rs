use std::time::Duration;

use calloop::{
    channel::{Event, Sender, SyncSender},
    timer::{TimeoutAction, Timer},
    LoopSignal,
};

use crate::{IntoMessages, Manager, Message, MessageBox, MessageList};

pub enum Msg {
    Update(Vec<MessageBox>),
    Retain(usize),
    Clear(bool),
}

struct EventLoopData {
    signal: LoopSignal,
    manager: Manager<std::io::Stdout>,
    message_list: MessageList,
}

pub struct EventLoop<'l> {
    el: calloop::EventLoop<'l, EventLoopData>,
}

impl<'l> EventLoop<'l> {
    pub fn new() -> EventLoop<'l> {
        EventLoop {
            el: calloop::EventLoop::try_new().unwrap(),
        }
    }

    pub fn run<F: FnOnce(LoopSignal, SyncSender<Msg>) + 'static + Send>(mut self, app: F) {
        let signal = self.el.get_signal();

        let mut event_loop_data = EventLoopData {
            signal,
            manager: Manager::<std::io::Stdout>::new(std::io::stdout()),
            message_list: MessageList::default(),
        };

        let (sx, channel) = calloop::channel::sync_channel::<Msg>(1);

        let cloned_signal = self.el.get_signal();
        std::thread::spawn(move || app(cloned_signal, sx));

        self.el
            .handle()
            .insert_source(channel, |event, _meta, shared_data| {
                //
                if let Event::Msg(msg) = event {
                    match msg {
                        Msg::Update(msg) => {
                            shared_data.message_list.update(msg);
                        }
                        Msg::Retain(nr) => {
                            shared_data.message_list.retain(nr);
                        }
                        Msg::Clear(outside) => {
                            shared_data.message_list.clear(outside);
                        }
                        _ => {}
                    }
                    shared_data.signal.wakeup();
                }
            })
            .expect("channel");

        // let mut current_msg = None;

        self.el
            .run(Duration::from_millis(80), &mut event_loop_data, |data| {
                data.message_list.tick();
                data.manager.update(&data.message_list);
            })
            .expect("run");
    }
}
