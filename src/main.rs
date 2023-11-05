use messi::{init, parse_commands::get_user_input, server_interaction::clone};

const PORT: &str = "9418";

fn main() {
    // let args = get_user_input();
    // let _second_argument = match args.get(1) {
    //     Some(arg) => arg,
    //     None => {
    //         eprintln!("No se ha ingresado el segundo argumento.");
    //         return;
    //     }
    // };

    let address = "localhost:".to_owned() + PORT;
    let result = clone::git_clone(
        &address,
        "repo_prueba",
        "localhost",
        "/home/fran/Desktop/prueba_para_clonar/repo_prueba",
    );
    println!("{:?}", result);
}
