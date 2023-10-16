use crate::cat_file::cat_file;
use crate::hash_object::store_file;
use crate::init::git_init;
use std::io;

/// Enumeration representing Git commands.
///
/// This enumeration defines Git commands that can be used.
#[derive(Debug)]
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

/// Reads user input from the command line and splits it into a vector of strings.
///
/// This function prompts the user to enter arguments separated by spaces and reads the input
/// from the command line. It then splits the input into individual strings and returns them
/// as a vector.
///
/// ## Example
///
/// ```
/// use your_crate_name::get_user_input;
///
/// let user_input = get_user_input();
/// println!("User input: {:?}", user_input);
/// ```
///
/// The function returns a vector of strings, where each element is a user-supplied argument.
///
/// If there's an error reading user input, an error message is printed to the standard error
/// stream (stderr).
///
/// ## Returns
///
/// Returns a `Vec<String>` containing the user-supplied arguments.
///
pub fn get_user_input() -> Vec<String> {
    let mut input = String::new();
    println!("Enter arguments (separated by spaces):");
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("Error reading user input");
    }

    let args: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
    args
}

/// Parses a user-supplied argument to determine the Git command to execute.
///
/// This function takes a user-supplied argument as a string and attempts to parse it
/// to determine the corresponding Git command.
///
/// ## Parameters
///
/// - `second_argument`: A string representing the user-supplied Git command argument.
///
/// ## Returns
///
/// Returns an `Option<GitCommand>` representing the parsed Git command. If the argument
/// matches a valid Git command, the corresponding `GitCommand` variant is returned within
/// `Some`. If the argument does not match any known Git command, an error message is printed
/// to the standard error stream (stderr), and `None` is returned.
///
/// ## Example
///
/// ```
/// use your_crate_name::{parse_git_command, GitCommand};
///
/// let arg = "init";
/// let result = parse_git_command(arg);
/// match result {
///     Some(cmd) => println!("Parsed Git command: {:?}", cmd),
///     None => println!("Invalid Git command argument"),
/// }
/// ```
///
/// The function can be used to parse user-supplied Git command arguments and determine
/// the corresponding `GitCommand` variant.
///
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
            eprintln!("Not a valid Git option.");
            None
        }
    }
}

/// Handles a Git command with the specified arguments.
///
/// This function takes a `GitCommand` enum representing the Git command to execute and a
/// vector of strings `args` that contains the command's arguments. It then delegates the
/// execution of the specific Git command to a corresponding handler function based on the
/// provided `GitCommand`. The handler function for each Git command is responsible for
/// executing the command with the given arguments.
///
/// ## Parameters
///
/// - `git_command`: The `GitCommand` enum representing the Git command to execute.
/// - `args`: A vector of strings containing the arguments for the Git command.
///
/// ## Example
///
/// ```
/// use parse_commands::{handle_git_command, GitCommand};
///
/// let git_command = GitCommand::Init;
/// let args = vec!["init".to_string()];
/// handle_git_command(git_command, args);
/// ```
///
/// The function allows for the execution of different Git commands based on the provided
/// `GitCommand` enum and its corresponding arguments.
///
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

/// Handles the 'hash-object' Git command.
///
/// This function takes a list of arguments `args`, where the third argument (index 2) should be
/// the path to the file to be stored as a Git object. It then stores the file as a Git object
/// in the object directory and returns the SHA-1 hash of the stored content.
///
/// # Arguments
///
/// - `args`: A vector of strings representing the arguments passed to the command.
///
fn handle_hash_object(args: Vec<String>) {
    let file_to_store = &args[2];
    let current_directory = ".";

    match store_file(file_to_store, current_directory) {
        Ok(hash) => {
            println!(
                "El archivo se ha almacenado como un objeto Git con hash: {}",
                hash
            );
        }
        Err(e) => {
            eprintln!("Error al almacenar el archivo como objeto Git: {}", e);
        }
    }
}

/// Handles the 'cat-file' Git command.
///
/// This function takes a list of arguments `args`, where the third argument (index 2) should be
/// the hash of the Git object to retrieve and display its content. It then calls the `cat_file`
/// function to retrieve and print the content of the Git object with the provided hash.
///
/// # Arguments
///
/// - `args`: A vector of strings representing the arguments passed to the command.
fn handle_cat_file(args: Vec<String>) {
    let hash = &args[2];
    let current_directory = ".";

    match cat_file(hash, current_directory, &mut std::io::stdout()) {
        Ok(()) => {
            println!();
        }
        Err(e) => {
            eprintln!("Error al obtener el contenido del archivo: {}", e);
        }
    }
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

/// Initializes a Git repository with optional configuration.
///
/// This function initializes a Git repository in the specified directory. It supports
/// optional configuration through command-line arguments:
///
/// - `-b` or `--initial-branch`: Specifies the name of the initial branch. If not provided,
///   the default branch name 'main' is used.
/// - `--template`: Specifies a template directory to copy files from.
///
/// If the provided `current_directory` doesn't exist, it will be created.
///
/// ## Parameters
///
/// - `args`: A vector of strings containing command-line arguments. The function parses
///   these arguments to determine the initial branch and template directory.
///
/// ## Example
///
/// ```
/// use init::{handle_init, git_init};
///
/// let args = vec!["init".to_string(), "my_repo".to_string(), "-b".to_string(), "mybranch".to_string()];
/// handle_init(args);
/// ```
///
/// The `handle_init` function initializes a Git repository based on the provided arguments,
/// allowing you to specify the initial branch and a template directory.
///
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
