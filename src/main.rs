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

fn get_user_input() -> Vec<String> {
    let mut input = String::new();
    println!("Ingresa los argumentos (separados por espacios):");
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("Error al leer la entrada del usuario");
    }
    
    let args: Vec<String> = input.trim().split_whitespace().map(|s| s.to_string()).collect();
    args
}

fn parse_git_command(second_argument: &str) -> Option<GitCommand> {
    match second_argument {
        "hash-object" => Some(GitCommand::HashObject),
        "cat-file" => Some(GitCommand::CatFile),
        "init" => Some(GitCommand::Init),
        "status" => Some(GitCommand::Status),
        "add" => Some(GitCommand::Add),
        "rm" => Some(GitCommand::Rm),
        "commit" => Some(GitCommand::Commit),
        "checkout" => Some(GitCommand::Checkout),
        "log" => Some(GitCommand::Log),
        "clone" => Some(GitCommand::Clone),
        "fetch" => Some(GitCommand::Fetch),
        "merge" => Some(GitCommand::Merge),
        "remote" => Some(GitCommand::Remote),
        "pull" => Some(GitCommand::Pull),
        "push" => Some(GitCommand::Push),
        "branch" => Some(GitCommand::Branch),
        _ => {
            eprintln!("No es una opción válida de Git.");
            None
        }
    }
}

fn handle_git_command(git_command: GitCommand, second_argument: &str) {
    match git_command {
        GitCommand::HashObject
        | GitCommand::CatFile
        | GitCommand::Status
        | GitCommand::Add
        | GitCommand::Rm
        | GitCommand::Commit
        | GitCommand::Checkout
        | GitCommand::Log
        | GitCommand::Clone
        | GitCommand::Fetch
        | GitCommand::Merge
        | GitCommand::Remote
        | GitCommand::Pull
        | GitCommand::Push
        | GitCommand::Branch => {
            println!("Segundo argumento ingresado: {}", second_argument);
        }
        GitCommand::Init => {
            println!("Segundo argumento ingresado: {}", second_argument);
            if let Err(err) = git_init("a", "branch", None) {
                eprintln!("Error al inicializar el repositorio Git: {}", err);
            }
        }
    }
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

    if let Some(git_command) = parse_git_command(second_argument) {
        handle_git_command(git_command, second_argument);
    }
}

