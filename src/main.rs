use std::io;

fn main() {
    let args = get_user_input();
    let second_argument = match args.get(1) {
        Some(arg) => arg,
        None => {
            eprintln!("No se ha ingresado el tercer argumento.");
            return;
        }
    };

    println!("Tercer argumento ingresado: {}", second_argument);
}

fn get_user_input() -> Vec<String> {
    let mut input = String::new();
    println!("Ingresa los argumentos (separados por espacios):");
    io::stdin().read_line(&mut input).expect("Error al leer la entrada del usuario");
    
    // Divide la entrada en argumentos y la almacena en un vector
    let args: Vec<String> = input.trim().split_whitespace().map(|s| s.to_string()).collect();
    args
}
