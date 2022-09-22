use super::message::Message;
use std::{
    fmt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

pub trait Frames {
    fn len(&self) -> usize;
    fn write(&self, frame: usize, f: &mut fmt::Formatter) -> fmt::Result;
}

macro_rules! array {
    ($($count: literal)*) => {
        $(
            impl<M: Message> Frames for [M; $count] {
                fn len(&self) -> usize {
                    $count
                }

                fn write(&self, frame: usize, f: &mut fmt::Formatter) -> fmt::Result {
                    if frame < $count {
                        self[frame].message(f)
                    } else {
                        write!(f, "")
                    }


                }
            }
        )*
    };
}

array!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);

pub const DEFAULT: [char; 2] = ['-', '|'];

pub const DOTS: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

struct State<F> {
    frames: F,
    current: AtomicUsize,
}
pub struct Spinner<F> {
    state: Arc<State<F>>,
}

impl<F> Clone for Spinner<F> {
    fn clone(&self) -> Self {
        Spinner {
            state: self.state.clone(),
        }
    }
}

impl<F> Spinner<F>
where
    F: Frames,
{
    pub fn new(frames: F) -> Spinner<F> {
        Spinner {
            state: Arc::new(State {
                frames: frames,
                current: AtomicUsize::new(0),
            }),
        }
    }

    pub fn next(&self) {
        self.state
            .current
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(x + 1))
            .ok();
    }

    pub fn write(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let current = self.state.current.load(Ordering::Relaxed);
        self.state
            .frames
            .write(current % self.state.frames.len(), f)?;

        Ok(())
    }
}

impl<F> Message for Spinner<F>
where
    F: Frames + Send + Sync,
{
    fn line_count(&self) -> usize {
        1
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f)
    }

    fn tick(&mut self) {
        (&*self).next()
    }
}
