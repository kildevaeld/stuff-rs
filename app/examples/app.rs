fn main() -> Result<(), app::Error> {
    let builder = app::Builder::<(), _>::new()
        .config_path("app/examples")?
        .init(|core| {
            //
        })
        .build(());

    Ok(())
}
