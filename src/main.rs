use std::io;

enum GitCommand {
    HashObject,
    CatFile,
    Init,
    Status,
    Add,
    Rm,
    Commit,
    Checkout,
    Log,
    Clone,
    Fetch,
    Merge,
    Remote,
    Pull,
    Push,
    Branch,
}


fn main() {
    let args = get_user_input();
    let second_argument = match args.get(1) {
        Some(arg) => arg,
        None => {
            eprintln!("No se ha ingresado el segundo argumento.");
            return; 
        }
    };

    let git_command = match second_argument.as_str() {
        "hash-object" => GitCommand::HashObject,
        "cat-file" => GitCommand::CatFile,
        "init" => GitCommand::Init,
       
        _ => {
            eprintln!("El segundo argumento no es una opción válida de Git.");
            return; 
        }
    };

    match git_command {
        GitCommand::HashObject => {
            println!("Segundo argumento ingresado: {}", second_argument);
            execute_hash_object(); 
        }
        GitCommand::CatFile => {
            println!("Segundo argumento ingresado: {}", second_argument);
            execute_cat_file(); 
        }
        GitCommand::Init => {
            println!("Segundo argumento ingresado: {}", second_argument);
            execute_init(); 
        },
         _ => todo!()
       
    }
}

fn execute_hash_object() {
    println!("Ejecutando 'git hash-object'...");
    
}

fn execute_cat_file() {
    println!("Ejecutando 'git cat-file'...");
}

fn execute_init() {
    println!("Ejecutando 'git init'...");
}


fn get_user_input() -> Vec<String> {
    let mut input = String::new();
    println!("Ingresa los argumentos (separados por espacios):");
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("Error al leer la entrada del usuario");
    }
    
    let args: Vec<String> = input.trim().split_whitespace().map(|s| s.to_string()).collect();
    args
}
