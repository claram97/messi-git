use std::io;
use messi::init::git_init;

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
        "status" => GitCommand::Status,
        "add" => GitCommand::Add,
        "rm" => GitCommand::Rm,
        "commit" => GitCommand::Commit,
        "checkout" => GitCommand::Checkout,
        "log" => GitCommand::Log,
        "clone" => GitCommand::Clone,
        "fetch" => GitCommand::Fetch,
        "merge" => GitCommand::Merge,
        "remote" => GitCommand::Remote,
        "pull" => GitCommand::Pull,
        "push" => GitCommand::Push,
        "branch" => GitCommand::Branch,
        _ => {
            eprintln!("No es una opción válida de Git.");
            return; 
        }
    };

    match git_command {
        GitCommand::HashObject => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::CatFile => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Init => {
            println!("Segundo argumento ingresado: {}", second_argument);
            if let Err(err) = git_init("a", "branch", None) {
                eprintln!("Error al inicializar el repositorio Git: {}", err);
            } 
        }
        GitCommand::Status => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Add => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Rm => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Commit => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Checkout => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Log => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Clone => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Fetch => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Merge => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Remote => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Pull => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Push => {
            println!("Segundo argumento ingresado: {}", second_argument);
        },
         _ => todo!()
       
    }
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
