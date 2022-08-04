use crossterm::{
    cursor, execute, queue, style,
    terminal::{self, ClearType},
};
use locking::Lockable;
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crate::message::{Message, MessageExt};

struct State<W> {
    last_len: AtomicUsize,
    outside: AtomicUsize,
    writer: W,
}

pub struct Manager<W> {
    state: Arc<State<W>>,
}

impl<W> Clone for Manager<W> {
    fn clone(&self) -> Self {
        Manager {
            state: self.state.clone(),
        }
    }
}

impl<W> Manager<W>
where
    for<'a> W: Lockable<'a>,
    for<'a> <W as Lockable<'a>>::Guard: io::Write,
{
    pub fn new(writer: W) -> Manager<W> {
        Manager {
            state: Arc::new(State {
                last_len: AtomicUsize::new(0),
                outside: AtomicUsize::new(0),
                writer,
            }),
        }
    }

    pub fn retain(&self, mut count: usize) -> usize {
        let last_len = self.state.last_len.load(Ordering::Relaxed);
        if count > last_len {
            count = last_len;
        }

        self.state.outside.fetch_add(count, Ordering::Relaxed);
        self.state
            .last_len
            .store(last_len - count, Ordering::Relaxed);

        last_len - count
    }

    pub fn len(&self) -> usize {
        self.state.last_len.load(Ordering::Relaxed) + self.state.outside.load(Ordering::Relaxed)
    }

    pub fn hide_cursor(&self) -> io::Result<()> {
        execute!(self.state.writer.lock(), cursor::Hide)
    }

    pub fn show_cursor(&self) -> io::Result<()> {
        execute!(self.state.writer.lock(), cursor::Show)
    }

    pub fn update(&self, msg: impl Message) -> io::Result<()> {
        self.update_from(msg, 0)
    }

    pub fn update_from(&self, msg: impl Message, mut idx: usize) -> io::Result<()> {
        let mut writer = self.state.writer.lock();

        let last_len = self.state.last_len.load(Ordering::Relaxed);

        if idx > last_len {
            idx = last_len;
        }

        let mut count = self.retain(idx);

        queue!(writer, style::ResetColor)?;

        while count > 0 {
            queue!(
                writer,
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
            )?;

            if count > 1 {
                queue!(writer, cursor::MoveToPreviousLine(1))?;
            }

            count -= 1;
        }

        if idx > 0 {
            queue!(writer, style::Print("\n"))?;
        }

        let lines = msg.line_count();

        queue!(writer, style::Print(msg.display()))?;

        self.state.last_len.swap(lines, Ordering::Relaxed);

        writer.flush()?;

        Ok(())
    }

    pub fn clear(&self, outside: bool) -> io::Result<()> {
        let mut count = self.state.last_len.swap(0, Ordering::Relaxed);

        if outside {
            count += self.state.outside.swap(0, Ordering::Relaxed);
        }

        let mut writer = self.state.writer.lock();

        while count > 0 {
            queue!(
                writer,
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
            )?;

            if count > 1 {
                queue!(writer, cursor::MoveToPreviousLine(1))?;
            }

            count -= 1;
        }

        writer.flush()?;

        Ok(())
    }
}
