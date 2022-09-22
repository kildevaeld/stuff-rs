use cliupdate::Manager;
use crossterm::style::{style, Stylize};
use std::{thread::spawn, time::Duration};

// #[tokio::main]
fn main() -> std::io::Result<()> {
    let manager = Manager::<std::io::Stdout>::new(std::io::stdout());
    // manager.hide_cursor()?;

    std::thread::sleep(Duration::from_secs(1));

    manager.update("Tisk")?;

    std::thread::sleep(Duration::from_secs(1));

    manager.update_from("Tisk 2", 1)?;

    std::thread::sleep(Duration::from_secs(1));

    manager.update_from(style("Tisk 3").dark_cyan().bold(), 1)?;

    std::thread::sleep(Duration::from_secs(1));

    manager.clear(true)?;

    manager.update("Wait")?;

    let m = manager.clone();
    spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        m.update("From thread")?;
        std::thread::sleep(Duration::from_secs(1));
        m.update(["Lines", "from", "thread"])?;

        Result::<_, std::io::Error>::Ok(())
    });

    std::thread::sleep(Duration::from_secs(2));

    manager.clear(false)?;

    manager.show_cursor()?;

    Ok(())
}
