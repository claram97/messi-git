use std::io;

use messi::gui::run_main_window;
use messi::parse_commands::get_user_input;
use messi::parse_commands::{handle_git_command, parse_git_command};
fn main() -> io::Result<()> {
    if gtk::init().is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to initialize GTK.",
        ));
    }

    run_main_window().map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    gtk::main();

    let args = get_user_input();
    let second_argument = match args.get(1) {
        Some(arg) => arg,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se ha ingresado el segundo argumento.",
            ));
        }
    };

    if let Some(git_command) = parse_git_command(second_argument) {
        handle_git_command(git_command, args);
    }

    Ok(())
}
