use std::fs;
use std::io::Write;

use messi::{add, commit};
use messi::gui::run_main_window;
use messi::parse_commands::{get_user_input, handle_git_command, parse_git_command};
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

    // let mut file = fs::File::create("test.txt").unwrap();
    // file.write_all(b"Hello, world!").unwrap();

    // let result = add::add("test.txt", ".mgit/index", ".mgit", "", None);
    // println!("{:?}", result);

    // let result = commit::new_commit(".mgit", "Hola", "");
    // println!("{:?}", result);

    // let mut file = fs::File::create("test2.txt").unwrap();
    // file.write_all(b"Holis, world!").unwrap();

    // let result = add::add("test2.txt", ".mgit/index", ".mgit", "", None);
    // println!("{:?}", result);
        
    // let result = commit::new_commit(".mgit", "Holis", "");
    // println!("{:?}", result);

    // //Otro mas

    // let mut file = fs::File::create("test3.txt").unwrap();
    // file.write_all(b"Buenas, world!").unwrap();

    // let result = add::add("test3.txt", ".mgit/index", ".mgit", "", None);
    // println!("{:?}", result);

    // let result = commit::new_commit(".mgit", "Buenas", "");
    // println!("{:?}", result);
}
