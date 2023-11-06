use crate::branch::{git_branch, get_current_branch_path};
use crate::cat_file::cat_file;
use crate::commit::{new_commit, get_branch_name};
use crate::config::Config;
use crate::hash_object::store_file;
use crate::index::Index;
use crate::init::git_init;
use crate::log::print_logs;
use crate::merge::git_merge;
use crate::remote::git_remote;
use crate::rm::git_rm;
use crate::status::{find_untracked_files, find_unstaged_changes, changes_to_be_committed};
use crate::utils::find_git_directory;
use crate::{add, log, tree_handler};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::checkout::create_or_reset_branch;
use crate::checkout::checkout_commit_detached;
use crate::checkout::force_checkout;
use crate::checkout::create_and_checkout_branch;
use crate::checkout::checkout_branch;

use std::{env, io};


const GIT_DIR: &str = ".mgit";
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
/// use messi::parse_commands::get_user_input;
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
/// use messi::parse_commands::{parse_git_command, GitCommand};
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
/// use messi::parse_commands::{handle_git_command, GitCommand};
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
        GitCommand::Status => handle_status(),
        GitCommand::Add => handle_add(args),
        GitCommand::Rm => handle_rm(args),
        GitCommand::Commit => handle_commit(args),
        GitCommand::Checkout => handle_checkout(args),
        GitCommand::Log => handle_log(),
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
    match env::current_dir() {
        Ok(current_dir) => {
            let Some(git_dir) = (match find_git_directory(&mut current_dir.clone(), GIT_DIR) {
                Some(git_dir) => Some(git_dir),
                None => {
                    eprintln!("Git repository not found");
                    return;
                }
            }) else {
                eprintln!("Error trying to get current directory");
                return;
            };

            let file_to_store = &args[2];

            match store_file(file_to_store, &git_dir) {
                Ok(hash) => {
                    println!("File successfully stored as Git object with hash: {}", hash);
                }
                Err(e) => {
                    eprintln!("Error trying to store file as Git object: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error trying to get current directory: {}", e);
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
    if let Ok(current_dir) = env::current_dir() {
        let current_dir = &current_dir.to_string_lossy().to_string();

        match find_git_directory(&mut PathBuf::from(current_dir), GIT_DIR) {
            Some(git_dir) => match cat_file(hash, &git_dir, &mut std::io::stdout()) {
                Ok(()) => {
                    println!();
                }
                Err(e) => {
                    eprintln!("Error trying to access the content of the file: {}", e);
                }
            },
            None => {
                eprintln!("Git repository not found");
            }
        }
    } else {
        eprintln!("Error trying to get current directory");
    }
}

fn handle_status() {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };

    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            eprintln!("Error al obtener el git dir");
            return;
        }
    };


    let red = "\x1b[31m";
    let yellow = "\x1b[33m";
    let green = "\x1b[32m";
    let reset = "\x1b[0m";
    let branch_name = match get_branch_name(&git_dir) {
        Ok(name) => name,
        Err(_) => {
            eprintln!("No se pudo obtener la rama actual");
            return;
        }
    };
    let index_path = format!("{}/{}",git_dir,"index");
    
    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir,
        None => {
            eprintln!("Fatal error on git rm");
            return;
        }
    };

    let git_ignore_path = format!("{}/{}", working_dir.to_string_lossy(), ".mgitignore");


    let index = match Index::load(&index_path,&git_dir,&git_ignore_path) {
        Ok(index) => index,
        Err(_) => {
            eprintln!("Error loading index.");
            return;
        }
    };

    let current_branch_path = match get_current_branch_path(&git_dir) {
        Ok(path) =>path,
        Err(_e) => {
            eprintln!("Error getting current branch path.");
            return;
        }
    };

    let mut current_commit_file = match File::open(&current_branch_path) {
        Ok(file) => file,
        Err(_e) => {
            eprintln!("Error opening the current commit file.");
            return;
        }
    };
    
    let mut commit_hash = String::new();
    match current_commit_file.read_to_string(&mut commit_hash) {
        Ok(_) => {},
        Err(_e) => {
            eprintln!("Error reading to string.");
            return;
        }
    }
    let commit_tree = match tree_handler::load_tree_from_commit(&commit_hash, &git_dir) {
        Ok(tree) => tree,
        Err(_e) => {
            eprintln!("Couldn't load tree.");
            return;
        }
    };

    let mut untracked_output : Vec<u8> = vec![];
    match find_untracked_files(&current_dir,&working_dir, &index, &mut untracked_output) {
        Ok(_) => {},
        Err(_e) => {
            eprintln!("Error finding untracked files.");
            return;
        }
    }
    
    let mut not_staged_for_commit: Vec<u8> = vec![];
    match find_unstaged_changes(&index,&git_dir,&mut not_staged_for_commit) {
         Ok(_) => {},
         Err(_e) => {
             eprintln!("Error finding changes not staged for commit.");
             return;
         }
    } 

    let mut changes_to_be_committed_output: Vec<u8> = vec![];
    match changes_to_be_committed(&index, &commit_tree, &mut changes_to_be_committed_output) {
         Ok(_) => {},
         Err(_e) => {
             eprintln!("Error finding changes not staged for commit.");
             return;
         }
    } 
    
    print!("{}On branch {}{}\n", branch_name, green, reset);
    //Personalizar la siguiente línea
    print!("Your branch is up to date with 'origin/master'\n\n");
    if !not_staged_for_commit.is_empty() {
        print!("{}Changes not staged for commit:{}\n", red, reset);
        print!("\t(use \"git add <file>...\" to update what will be committed)\n");
        print!("\t(use \"git restore <file>...\" to discard changes in working directory)\n");
    
        for byte in &not_staged_for_commit {
            print!("{}", *byte as char);
        }
    }

    if !changes_to_be_committed_output.is_empty() {
        print!("{}Changes to be commited:{}\n", yellow, reset);
        println!("\t(use \"git add <file>...\" to update what will be committed)");
        println!("\t(use \"git checkout -- <file>...\" to discard changes in working directory)");
        for byte in &changes_to_be_committed_output {
            print!("{}", *byte as char);
        }
  
    }
    if !untracked_output.is_empty() {
        print!("{}Untracked files:{}\n", yellow, reset);
        print!("\t(use \"git add <file>...\" to include in what will be committed)\n");
        
        for byte in &untracked_output {
            print!("{}", *byte as char);
        }
    }
    //Esto no sé de dónde sacarlo ah
    //print!("{}no changes added to commit (use \"git add\" and/or \"git commit -a\"){}\n", green, reset);


}

fn handle_add(args: Vec<String>) {
    let (git_dir, git_ignore_path) =
        match crate::gui::repository_window::find_git_directory_and_ignore() {
            Ok((dir, ignore_path)) => (dir, ignore_path),
            Err(err) => {
                eprintln!("Error: {:?}", err); // Imprime el error en la salida de error estándar
                                               // Aquí puedes tomar acciones adicionales en caso de error si es necesario
                                               // Por ejemplo, puedes retornar valores por defecto o finalizar el programa.
                return;
            }
        };
    let index_path = (&git_dir).to_string() + "/index";
    match add::add(&args[2], &index_path, &git_dir, &git_ignore_path, None) {
        Ok(_) => {}
        Err(_err) => {}
    };
}

fn handle_rm(args: Vec<String>) {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };

    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            eprintln!("Error al obtener el git dir");
            return;
        }
    };

    let index_path = format!("{}/{}", git_dir, "index");
    let git_dir_parent = match Path::new(&git_dir).parent() {
        Some(dir) => dir,
        None => {
            eprintln!("Fatal error on git rm");
            return;
        }
    };

    let git_ignore_path = format!("{}/{}", git_dir_parent.to_string_lossy(), ".mgitignore");

    match git_rm(&args[2], &index_path, &git_dir, &git_ignore_path) {
        Ok(_) => {}
        Err(_err) => {}
    }
}

fn handle_commit(args: Vec<String>) {
    let (git_dir, git_ignore_path) =
        match crate::gui::repository_window::find_git_directory_and_ignore() {
            Ok((dir, ignore_path)) => (dir, ignore_path),
            Err(err) => {
                eprintln!("Error: {:?}", err); // Imprime el error en la salida de error estándar
                                               // Aquí puedes tomar acciones adicionales en caso de error si es necesario
                                               // Por ejemplo, puedes retornar valores por defecto o finalizar el programa.
                return;
            }
    };
    match new_commit(&git_dir,&args[3],&git_ignore_path) {
        Ok(_) => {},
        Err(_err) => {}
    };
}

fn handle_checkout(args: Vec<String>) {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };

    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            eprintln!("Error al obtener el git dir");
            return;
        }
    };

    let working_dir = match Path::new(&git_dir).parent() {
        Some(parent) => parent.to_string_lossy().to_string(),
        None => {
            eprintln!("Error al obtener el working dir");
            return;
        }
    };

    if args.len() < 2 {
        eprintln!("Usage: my_git_checkout <option> <branch_or_commit>");
        return;
    }

    let option = &args[2];
    let git_dir1 = Path::new(&git_dir);
    println!("aber {}", &args[1]);
    match option.as_str() {
        // Change to the specified branch
        
        // Create and change to a new branch
        "-b" => {
            let destination = &args[3];

            if let Err(err) = create_and_checkout_branch(git_dir1, &working_dir, destination) {
                eprintln!("Error al crear y cambiar a una nueva rama: {:?}", err);
            }
        }
        // Create or reset a branch if it exists
        "-B" => {
            let destination = &args[3];

            if let Err(err) = create_or_reset_branch(git_dir1, &working_dir, destination) {
                eprintln!("Error al crear o restablecer una rama si existe: {:?}", err);
            }
        }
        // Change to a specific commit (detached mode)
        "--detach" => {
            let destination = &args[3];

            if let Err(err) = checkout_commit_detached(git_dir1, &working_dir, destination) {
                eprintln!("Error al cambiar a un commit específico (modo desconectado): {:?}", err);
            }
        }
        // Force the change of branch or commit (discarding uncommitted changes)
        "-f" => {
            let destination = &args[3];

            if let Err(err) = force_checkout(git_dir1, destination) {
                eprintln!("Error al forzar el cambio de rama o commit (descartando cambios sin confirmar): {:?}", err);
            }
        }
        _ => {
            if let Err(err) = checkout_branch(git_dir1, &working_dir, option) {
                eprintln!("Error al cambiar a la rama especificada: {:?}", err);
            }
        }
    }
}


fn handle_log() {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };
    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir,
        None => {
            eprintln!("Error al obtener el git dir");
            return;
        }
    };

    let log_iter = match log::log(None, &git_dir, 10, 0, false) {
        Ok(iter) => iter,
        Err(_e) => {
            eprintln!("Error en git log.");
            return;
        }
    };
    print_logs(log_iter);
}

fn handle_clone(args: Vec<String>) {
    println!("Handling Clone command with argument: ");
}

fn handle_fetch(args: Vec<String>) {
    println!("Handling Fetch command with argument: ");
}

fn handle_merge(args: Vec<String>) {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };

    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            eprintln!("Error al obtener el git dir");
            return;
        }
    };

    let working_dir = match Path::new(&git_dir).parent() {
        Some(parent) => parent.to_string_lossy().to_string(),
        None => {
            eprintln!("Error al obtener el working dir");
            return;
        }
    }; 

    let branch_name = match get_branch_name(&git_dir) {
        Ok(name) => name,
        Err(_) => {
            eprintln!("No se pudo obtener la rama actual");
            return;
        }
    };

    match git_merge(&branch_name,&args[2],&git_dir,&working_dir) {
        Ok(_) => {

        },
        Err(_e) => {
            eprintln!("Error en git merge.");
        }
    };
    
}

fn handle_remote(args: Vec<String>) {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };

    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            eprintln!("Error al obtener el git dir");
            return;
        }
    };

    let working_dir = match Path::new(&git_dir).parent() {
        Some(parent) => parent.to_string_lossy().to_string(),
        None => {
            eprintln!("Error al obtener el working dir");
            return;
        }
    };

    let config_path = format!("{}/{}", working_dir, ".mgitignore");

    let mut config = match Config::load(&config_path) {
        Ok(config) => config,
        Err(_e) => {
            eprintln!("Error al cargar el config file.");
            return;
        }
    };

    let line: Vec<&str> = args.iter().skip(2).map(|s| s.as_str()).collect();

    match git_remote(&mut config, line, &mut io::stdout()) {
        Ok(_config) => {}
        Err(_e) => {
            eprintln!("Error en git remote.");
            return;
        }
    }
}

fn handle_pull(args: Vec<String>) {
    println!("Handling Pull command with argument: ");
}

fn handle_push(args: Vec<String>) {
    println!("Handling Push command with argument: ");
}

fn handle_branch(args: Vec<String>) {
    let result;
    if args.len() == 2 {
        result = git_branch(None);
    }
    else {
        let name = (&args[2]).to_string();
        result = git_branch(Some(name));
    }

    match result {
        Ok(_) => {},
        Err(_e) => {

        }
    }
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
/// use messi::parse_commands::handle_init;
/// use messi::init::git_init;
///
/// let args = vec!["init".to_string(), "my_repo".to_string(), "-b".to_string(), "mybranch".to_string()];
/// handle_init(args);
/// ```
///
/// The `handle_init` function initializes a Git repository based on the provided arguments,
/// allowing you to specify the initial branch and a template directory.
///
pub fn handle_init(args: Vec<String>) {
    let mut current_directory = match std::env::current_dir() {
        Ok(dir) => dir.to_string_lossy().to_string(),
        Err(_) => {
            eprintln!("Current dir not found");
            return;
        }
    };
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
                current_directory = arg.to_string();
            }
        }
        index += 1;
    }
    if let Err(err) = git_init(&current_directory, initial_branch, template_directory) {
        eprintln!("Error al inicializar el repositorio Git: {}", err);
    }
}
