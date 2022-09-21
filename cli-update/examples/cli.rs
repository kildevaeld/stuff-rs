use calloop::LoopSignal;
use cli_update::{EventLoop, MessageExt, Msg, Spinner, DOTS};

fn main() {
    let el = EventLoop::new();

    el.run(|signal, sx| {
        //
        // sx.send(Msg::Update(Box::new((
        //     "Hello, World".join_with(Spinner::new(DOTS), " "),
        //     "Hello, World!",
        // ))));
        sx.send(Msg::Update(vec![Box::new(
            "Hello, World".join_with(Spinner::new(DOTS), " "),
        )]));

        std::thread::sleep(std::time::Duration::from_millis(1000));

        sx.send(Msg::Update(vec![Box::new(
            "Hello, World".join_with("✓", " "),
        )]));

        std::thread::sleep(std::time::Duration::from_millis(1000));

        sx.send(Msg::Retain(1));

        std::thread::sleep(std::time::Duration::from_millis(1000));

        sx.send(Msg::Update(vec![Box::new("Test")]));

        std::thread::sleep(std::time::Duration::from_millis(1000));

        // sx.send(Msg::Clear(false));

        signal.stop();
    });
}
