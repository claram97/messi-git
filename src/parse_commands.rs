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

pub fn handle_git_command(git_command: GitCommand, second_argument: &str) {
    match git_command {
        GitCommand::HashObject => handle_hash_object(second_argument),
        GitCommand::CatFile => handle_cat_file(second_argument),
        GitCommand::Status => handle_status(second_argument),
        GitCommand::Add => handle_add(second_argument),
        GitCommand::Rm => handle_rm(second_argument),
        GitCommand::Commit => handle_commit(second_argument),
        GitCommand::Checkout => handle_checkout(second_argument),
        GitCommand::Log => handle_log(second_argument),
        GitCommand::Clone => handle_clone(second_argument),
        GitCommand::Fetch => handle_fetch(second_argument),
        GitCommand::Merge => handle_merge(second_argument),
        GitCommand::Remote => handle_remote(second_argument),
        GitCommand::Pull => handle_pull(second_argument),
        GitCommand::Push => handle_push(second_argument),
        GitCommand::Branch => handle_branch(second_argument),
        GitCommand::Init => handle_init(second_argument),
    }
}

fn handle_hash_object(second_argument: &str) {
    println!("Handling HashObject command with argument: {}", second_argument);
}

fn handle_cat_file(second_argument: &str) {
    println!("Handling CatFile command with argument: {}", second_argument);
}
fn handle_status(second_argument: &str) {
    println!("Handling Status command with argument: {}", second_argument);
}

fn handle_add(second_argument: &str) {
    println!("Handling Add command with argument: {}", second_argument);

}

fn handle_rm(second_argument: &str) {
    println!("Handling Rm command with argument: {}", second_argument);
}

fn handle_commit(second_argument: &str) {
    println!("Handling Commit command with argument: {}", second_argument);
}

fn handle_checkout(second_argument: &str) {
    println!("Handling Checkout command with argument: {}", second_argument);
}

fn handle_log(second_argument: &str) {
    println!("Handling Log command with argument: {}", second_argument);

}

fn handle_clone(second_argument: &str) {
    println!("Handling Clone command with argument: {}", second_argument);
    
}

fn handle_fetch(second_argument: &str) {
    println!("Handling Fetch command with argument: {}", second_argument);
    
}

fn handle_merge(second_argument: &str) {
    println!("Handling Merge command with argument: {}", second_argument);
    
}

fn handle_remote(second_argument: &str) {
    println!("Handling Remote command with argument: {}", second_argument);
    
}

fn handle_pull(second_argument: &str) {
    println!("Handling Pull command with argument: {}", second_argument);
    
}

fn handle_push(second_argument: &str) {
    println!("Handling Push command with argument: {}", second_argument);
    
}

fn handle_branch(second_argument: &str) {
    println!("Handling Branch command with argument: {}", second_argument);
}

fn handle_init(second_argument: &str) {
    println!("Handling Init command with argument: {}", second_argument);
    
}

