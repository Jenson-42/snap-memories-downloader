use std::io;
#[cfg(windows)]
use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        WindowsResource::new()
            //Path to icon file.
            .set_icon("./resources/icon.ico")
            .compile()?;
    }
    Ok(())
}
