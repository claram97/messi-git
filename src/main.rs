use messi::parse_commands::get_user_input;

fn main() {
    let args = get_user_input();
    let _second_argument = match args.get(1) {
        Some(arg) => arg,
        None => {
            eprintln!("No se ha ingresado el segundo argumento.");
            return;
        }
    };
}