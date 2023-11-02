use messi::gui::run_main_window;
use messi::parse_commands::get_user_input;
use messi::parse_commands::{handle_git_command, parse_git_command};
fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    run_main_window();

    gtk::main();

    let args = get_user_input();
    let second_argument = match args.get(1) {
        Some(arg) => arg,
        None => {
            eprintln!("No se ha ingresado el segundo argumento.");
            return;
        }
    };

    if let Some(git_command) = parse_git_command(second_argument) {
        handle_git_command(git_command, args);
    }
}
