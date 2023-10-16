use std::io;
use crate::init::git_init;

pub enum GitCommand {
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

pub fn get_user_input() -> Vec<String> {
    let mut input = String::new();
    println!("Ingresa los argumentos (separados por espacios):");
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("Error al leer la entrada del usuario");
    }
    
    let args: Vec<String> = input.trim().split_whitespace().map(|s| s.to_string()).collect();
    args
}

pub fn parse_git_command(second_argument: &str) -> Option<GitCommand> {
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

pub fn handle_git_command(git_command: GitCommand, args: Vec<String>) {
    match git_command {
        GitCommand::HashObject => handle_hash_object(args),
        GitCommand::CatFile => handle_cat_file(args),
        GitCommand::Status => handle_status(args),
        GitCommand::Add => handle_add(args),
        GitCommand::Rm => handle_rm(args),
        GitCommand::Commit => handle_commit(args),
        GitCommand::Checkout => handle_checkout(args),
        GitCommand::Log => handle_log(args),
        GitCommand::Clone => handle_clone(args),
        GitCommand::Fetch => handle_fetch(args),
        GitCommand::Merge => handle_merge(args),
        GitCommand::Remote => handle_remote(args),
        GitCommand::Pull => handle_pull(args),
        GitCommand::Push => handle_push(args),
        GitCommand::Branch => handle_branch(args),
        GitCommand::Init => handle_init(args),
    }
}

fn handle_hash_object(args: Vec<String>) {
    println!("Handling HashObject command with argument: ");
}

fn handle_cat_file(args: Vec<String>) {
    println!("Handling CatFile command with argument: ");
}

fn handle_status(args: Vec<String>) {
    println!("Handling Status command with argument: ");
}

fn handle_add(args: Vec<String>) {
    println!("Handling Add command with argument: ");
}

fn handle_rm(args: Vec<String>) {
    println!("Handling Rm command with argument: ");
}

fn handle_commit(args: Vec<String>) {
    println!("Handling Commit command with argument: ");
}

fn handle_checkout(args: Vec<String>) {
    println!("Handling Checkout command with argument: ");
}

fn handle_log(args: Vec<String>) {
    println!("Handling Log command with argument: ");

}

fn handle_clone(args: Vec<String>) {
    println!("Handling Clone command with argument: ");
    
}

fn handle_fetch(args: Vec<String>) {
    println!("Handling Fetch command with argument: ");
    
}

fn handle_merge(args: Vec<String>) {
    println!("Handling Merge command with argument: ");
    
}

fn handle_remote(args: Vec<String>) {
    println!("Handling Remote command with argument: ");
    
}

fn handle_pull(args: Vec<String>) {
    println!("Handling Pull command with argument: ");
    
}

fn handle_push(args: Vec<String>) {
    println!("Handling Push command with argument: ");
    
}

fn handle_branch(args: Vec<String>) {
    println!("Handling Branch command with argument: ");
}

pub fn handle_init(args: Vec<String>) {
   
    let mut current_directory = ".";
    let mut initial_branch = "main";
    let mut template_directory: Option<&str> = None;

    let mut index = 2; 
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "-b" | "--initial-branch" => {
                if index + 1 < args.len() {
                    initial_branch = &args[index + 1];
                    index += 1;
                }
            }
            "--template" => {
                if index + 1 < args.len() {
                    template_directory = Some(&args[index + 1]);
                    index += 1;
                }
            }
            _ => {
                current_directory = arg;
            }
        }
        index += 1;
    }
    if let Err(err) = git_init(current_directory, initial_branch, template_directory) {
        eprintln!("Error al inicializar el repositorio Git: {}", err);
    }
}

