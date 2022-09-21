mod emitter;
mod transport;
pub use self::{emitter::*, transport::*};

#[cfg(test)]
mod test {

    use super::*;

    struct Event(i32);

    #[test]
    fn test() {
        let mut global = Event(1);
        let emitter = Emitter::<i32, Direct<i32>>::new();

        let sub = emitter.listen(|event: &i32| {
            println!("GOT EVENT: {}", event);
            global.0 += 1;
        });

        let (sx, rx) = std::sync::mpsc::channel();

        let sub2 = emitter.listen(sx);

        emitter.emit(200);
        println!("EVENT {:?}", rx.recv());
        drop(sub2);
        emitter.emit(201);

        println!("EVENT {:?}", rx.recv());
        sub.close();

        emitter.emit(200);

        drop(emitter);

        println!("GLOBAL {}", global.0);
    }

    #[cfg(feature = "sync")]
    #[test]
    fn test_sync() {
        let emitter = Emitter::<i32, SyncDirect<i32>>::new();

        let _ = emitter.listen(|event: &i32| {
            println!("GOT EVENT: {}", event);
        });

        let m = emitter.clone();
        std::thread::spawn(move || m.emit(400));

        emitter.emit(200);

        emitter.emit(201);

        std::thread::sleep(std::time::Duration::from_millis(200));
        drop(emitter);
    }

    #[cfg(feature = "thread")]
    #[test]
    fn test_thread() {
        let emitter = Emitter::<i32, Thread<i32>>::new();

        let sub = emitter.listen(|event: &i32| {
            println!("THREADED GOT EVENT: {}", event);
        });

        /*let m = emitter.clone();
        std::thread::spawn(move || {
            m.emit(400)
        });*/

        emitter.emit(200);

        emitter.emit(201);

        std::thread::sleep(std::time::Duration::from_secs(1));
        drop(emitter);
    }

    #[cfg(feature = "pool")]
    #[test]
    fn test_pool() {
        let emitter = Emitter::<i32, Pool<i32>>::new();

        let sub = emitter.listen(|event: &i32| {
            println!("PPOL GOT EVENT: {}", event);
        });

        let m = emitter.clone();
        std::thread::spawn(move || m.emit(400));

        emitter.emit(200);

        emitter.emit(201);

        std::thread::sleep(std::time::Duration::from_millis(200));
        drop(emitter);
    }
}
