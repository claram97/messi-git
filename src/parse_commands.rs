use crate::branch::{self, get_current_branch_path, git_branch};
use crate::cat_file::cat_file;
use crate::check_ignore::git_check_ignore;
use crate::checkout::checkout_branch;
use crate::checkout::checkout_commit_detached;
use crate::checkout::create_and_checkout_branch;
use crate::checkout::create_or_reset_branch;
use crate::checkout::force_checkout;
use crate::clone::git_clone;
use crate::commit::{get_branch_name, new_commit};
use crate::config::Config;
use crate::configuration::{GIT_DIR, GIT_IGNORE, HOST, INDEX};
use crate::fetch::git_fetch;
use crate::hash_object::store_file;
use crate::index::Index;
use crate::init::git_init;
use crate::log::print_logs;
use crate::ls_files::git_ls_files;
use crate::merge::git_merge;
use crate::pull::git_pull;
use crate::remote::git_remote;
use crate::rm::git_rm;
use crate::show_ref::git_show_ref;
use crate::status::{changes_to_be_committed, find_unstaged_changes, find_untracked_files};
use crate::tree_handler::Tree;
use crate::utils::{find_git_directory, obtain_git_dir};
use crate::{add, git_config, log, ls_tree, push, rebase, tag, tree_handler};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use std::{env, io};

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
    ListFiles,
    ListTrees,
    CheckIgnore,
    ShowRef,
    Rebase,
    Tag,
    Config,
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
        "ls-files" => Some(GitCommand::ListFiles),
        "ls-tree" => Some(GitCommand::ListTrees),
        "check-ignore" => Some(GitCommand::CheckIgnore),
        "show-ref" => Some(GitCommand::ShowRef),
        "rebase" => Some(GitCommand::Rebase),
        "tag" => Some(GitCommand::Tag),
        "config" => Some(GitCommand::Config),
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
        GitCommand::Pull => handle_pull(),
        GitCommand::Push => handle_push(),
        GitCommand::Branch => handle_branch(args),
        GitCommand::Init => handle_init(args),
        GitCommand::ListFiles => handle_ls_files(args),
        GitCommand::ListTrees => handle_ls_trees(args),
        GitCommand::CheckIgnore => handle_check_ignore(args),
        GitCommand::ShowRef => handle_show_ref(args),
        GitCommand::Rebase => handle_rebase(args),
        GitCommand::Tag => handle_tag(args),
        GitCommand::Config => handle_config(args),
    }
}

/// Handles the configuration based on the provided arguments.
///
/// # Arguments
///
/// * `args` - A vector of strings containing the command-line arguments.
///
fn handle_config(args: Vec<String>) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el git dir.\n {:?}", err);
            return;
        }
    };
    let change = args.len() == 5;
    match git_config::git_config(&git_dir, args) {
        Ok(_) => {
            if change {
                println!("Successfully changed");
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
        }
    }
}

/// Handles the "ls-files" command in a Git-like system.
///
/// # Arguments
///
/// * `args` - The arguments passed to the "ls-files" command.
fn handle_ls_files(args: Vec<String>) {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };

    let git_dir = match find_git_directory(&mut current_dir, GIT_DIR) {
        Some(dir) => dir,
        None => {
            eprintln!("Error al obtener el git dir");
            return;
        }
    };

    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            eprintln!("Fatal error on ls files.");
            return;
        }
    };

    let index_path = format!("{}/{}", git_dir, "index");
    let gitignore_path = format!("{}/{}", working_dir, GIT_IGNORE);
    let index = match Index::load(&index_path, &git_dir, &gitignore_path) {
        Ok(ind) => ind,
        Err(_e) => {
            eprintln!("Couldn't load index.");
            return;
        }
    };

    let current_dir = current_dir.to_string_lossy().to_string();
    match git_ls_files(
        &working_dir,
        &git_dir,
        &current_dir,
        args,
        &index,
        &mut io::stdout(),
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e)
        }
    }
}

fn handle_ls_trees(args: Vec<String>) {
    if args.len() < 3 {
        eprintln!("Usage: git ls-tree <tree-ish>");
        return;
    }

    let git_dir = match find_git_directory(&mut PathBuf::from("."), GIT_DIR) {
        Some(dir) => dir,
        None => {
            eprintln!("Error obtaining the git directory");
            return;
        }
    };

    match args.len() {
        3 => handle_ls_tree_default(&args[2], PathBuf::from(&git_dir)),
        4 => handle_ls_tree_with_options(&args[2..], PathBuf::from(&git_dir)),
        5 => handle_ls_tree_with_option(&args[2], PathBuf::from(&git_dir), &args[3]),
        _ => {
            eprintln!("Usage: git ls-tree <tree-ish>");
        }
    }
}

fn handle_ls_tree_default(tree_ish: &str, git_dir: PathBuf) {
    let result = ls_tree::ls_tree(
        tree_ish,
        git_dir.to_str().expect("Invalid git_dir path"),
        "",
        &mut io::stdout(),
    );
    if let Err(err) = result {
        eprintln!("{:?}", err);
    }
}

fn handle_ls_tree_with_options(args: &[String], git_dir: PathBuf) {
    let git_dir_str = git_dir.to_str().expect("Invalid git_dir path");
    match args {
        [arg1, tree_ish] if arg1 == "-r" => {
            handle_ls_tree_with_option(tree_ish, git_dir_str.into(), "-r")
        }
        [arg1, tree_ish] if arg1 == "-d" => {
            handle_ls_tree_with_option(tree_ish, git_dir_str.into(), "-d")
        }
        [arg1, arg2, tree_ish] if arg1 == "-r" && arg2 == "-t" => {
            handle_ls_tree_with_option(tree_ish, git_dir_str.into(), "-r-t")
        }
        _ => eprintln!("Usage: git ls-tree <tree-ish>"),
    }
}

fn handle_ls_tree_with_option(tree_ish: &str, git_dir: PathBuf, option: &str) {
    let result = ls_tree::ls_tree(
        tree_ish,
        git_dir.to_str().expect("Invalid git_dir path"),
        option,
        &mut io::stdout(),
    );
    if let Err(err) = result {
        eprintln!("{:?}", err);
    }
}

/// Handles the "check-ignore" command in a Git-like system.
///
/// # Arguments
///
/// * `args` - The arguments passed to the "check-ignore" command.
fn handle_check_ignore(args: Vec<String>) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el git dir.\n {:?}", err);
            return;
        }
    };

    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            eprintln!("Fatal error on ls files.");
            return;
        }
    };

    let gitignore_path = format!("{}/{}", working_dir, GIT_IGNORE);

    match git_check_ignore(GIT_IGNORE, &gitignore_path, args, &mut io::stdout()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e)
        }
    }
}

/// Handles the "show-ref" command in a Git-like system.
///
/// # Arguments
///
/// * `args` - The arguments passed to the "show-ref" command.
fn handle_show_ref(args: Vec<String>) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el git dir.\n {:?}", err);
            return;
        }
    };

    match git_show_ref(&git_dir, args, &mut io::stdout()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e)
        }
    }
}

fn handle_rebase(args: Vec<String>) {
    if args.len() < 3 {
        eprintln!("Usage: git rebase <branch>");
        return;
    }

    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("{:?}", error);
            return;
        }
    };

    let our_branch = match get_branch_name(&git_dir) {
        Ok(name) => name,
        Err(_) => {
            eprintln!("No se pudo obtener la rama actual");
            return;
        }
    };

    if args.len() == 3 {
        // Check if the branch given exists
        let branch_name = &args[2];
        match branch::get_branch_commit_hash(branch_name, &git_dir) {
            Ok(_) => {
                // Do the rebase
                match rebase::rebase(&our_branch, branch_name, &git_dir) {
                    Ok(_) => {
                        println!("Rebase successful");
                    }
                    Err(error) => {
                        eprintln!("{}", error);
                    }
                }
            }
            Err(_) => {
                eprintln!("Branch {} does not exist", branch_name);
            }
        };
    } else {
        eprintln!("Usage: git rebase <branch>");
    }
}

fn handle_tag(args: Vec<String>) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("{:?}", error.to_string());
            return;
        }
    };
    match tag::git_tag(&git_dir, args, &mut io::stdout()) {
        Ok(_) => {}
        Err(error) => {
            eprintln!("{:?}", error.to_string());
        }
    };
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

/// Retrieves the working directory of a Git repository based on the provided Git directory.
///
/// This function takes a Git directory path (`git_dir`) and returns the corresponding
/// working directory. It is assumed that the Git directory is a valid path, and the function
/// attempts to obtain the parent directory as the working directory. If successful, it returns
/// a `PathBuf` containing the working directory.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git directory.
///
/// # Returns
///
/// Returns a `Result` containing a `PathBuf` with the working directory if successful,
/// otherwise returns an `io::Error` with a description of the encountered issue.
///
fn get_working_directory_status(git_dir: &str) -> io::Result<PathBuf> {
    let parent = Path::new(git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Error al obtener el working dir"))?;
    Ok(parent.to_path_buf())
}

/// Loads the index and commit tree of a Git repository.
///
/// This function reads and returns the index and commit tree of a Git repository
/// based on the provided Git directory.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git directory.
///
/// # Returns
///
/// Returns a `Result` containing a tuple with the loaded `Index` and `Tree` if successful,
/// otherwise returns an `io::Error` with a description of the encountered issue.
///
/// # Errors
///
/// Returns an `io::Error` if there are issues loading the index or commit tree.
///
fn load_index_and_commit_tree(git_dir: &str) -> io::Result<(Index, Tree)> {
    let index_path = format!("{}/{}", git_dir, "index");
    let git_ignore_path = format!(
        "{}/{}",
        get_working_directory_status(git_dir)?.to_string_lossy(),
        GIT_IGNORE
    );

    let index = Index::load(&index_path, git_dir, &git_ignore_path)?;

    let branch_path = get_current_branch_path(git_dir)?;
    let current_branch_path = format!("{}/{}", git_dir, branch_path);

    if let Ok(mut current_commit_file) = File::open(current_branch_path) {
        let mut commit_hash = String::new();
        current_commit_file.read_to_string(&mut commit_hash)?;

        let commit_tree = tree_handler::load_tree_from_commit(&commit_hash, git_dir)?;

        Ok((index, commit_tree))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Error al abrir el archivo de commit",
        ))
    }
}

/// Prints the changes to be committed based on the Git index and commit tree.
///
/// This function prints the changes that are currently staged for commit according to
/// the provided Git index and commit tree. It provides information about files that
/// are scheduled to be committed.
///
/// # Arguments
///
/// * `index` - A reference to the Git index.
/// * `commit_tree` - A reference to the commit tree.
///
/// # Returns
///
/// Returns an `io::Result` indicating whether the operation was successful or encountered an error.
///
/// # Errors
///
/// Returns an `io::Error` if there are issues while determining the changes to be committed.
///
fn print_changes_to_be_committed(index: &Index, commit_tree: &Tree) -> io::Result<()> {
    let mut changes_to_be_committed_output: Vec<u8> = vec![];
    changes_to_be_committed(index, commit_tree, &mut changes_to_be_committed_output)?;

    if !changes_to_be_committed_output.is_empty() {
        println!();
        println!("\x1b[33mChanges to be commited:\x1b[0m\n");
        println!("\t(use \"git add <file>...\" to update what will be committed)");
        println!("\t(use \"git checkout -- <file>...\" to discard changes in working directory)");

        for byte in &changes_to_be_committed_output {
            print!("{}", *byte as char);
            println!();
        }
    }

    Ok(())
}

/// Prints information about untracked files based on the current state of the Git repository.
///
/// This function identifies and prints information about files in the working directory
/// that are not currently tracked by Git. It provides guidance on how to include these
/// files in the next commit.
///
/// # Arguments
///
/// * `current_dir` - A reference to the current directory.
/// * `working_dir` - A reference to the working directory.
/// * `index` - A reference to the Git index.
///
/// # Returns
///
/// Returns an `io::Result` indicating whether the operation was successful or encountered an error.
///
/// # Errors
///
/// Returns an `io::Error` if there are issues while finding untracked files.
///
fn print_untracked_files(current_dir: &Path, working_dir: &Path, index: &Index) -> io::Result<()> {
    let mut untracked_output: Vec<u8> = vec![];
    find_untracked_files(current_dir, working_dir, index, &mut untracked_output)?;

    if !untracked_output.is_empty() {
        println!();
        println!("\x1b[33mUntracked files:\x1b[0m\n");
        println!("\t(use \"git add <file>...\" to include in what will be committed)");

        for byte in &untracked_output {
            print!("{}", *byte as char);
        }
    }

    Ok(())
}

/// Prints information about changes not staged for commit in the Git repository.
///
/// This function identifies and prints information about changes in the working directory
/// that have not been staged for the next commit. It provides guidance on how to update
/// the changes to be included in the next commit or discard them.
///
/// # Arguments
///
/// * `index` - A reference to the Git index.
/// * `working_dir` - A reference to the working directory.
///
/// # Returns
///
/// Returns an `io::Result` indicating whether the operation was successful or encountered an error.
///
/// # Errors
///
/// Returns an `io::Error` if there are issues while finding changes not staged for commit.
///
fn print_not_staged_for_commit(index: &Index, working_dir: &Path) -> io::Result<()> {
    let mut not_staged_for_commit: Vec<u8> = vec![];
    find_unstaged_changes(
        index,
        working_dir.to_string_lossy().as_ref(),
        &mut not_staged_for_commit,
    )?;

    if !not_staged_for_commit.is_empty() {
        println!();
        println!("\x1b[31mChanges not staged for commit:\x1b[0m\n");
        println!("\t(use \"git add <file>...\" to update what will be committed)");
        println!("\t(use \"git restore <file>...\" to discard changes in working directory)");

        for byte in &not_staged_for_commit {
            print!("{}", *byte as char);
        }
    }

    Ok(())
}

/// Handles the status command for a Git repository.
///
/// This function provides information about the current status of the Git repository,
/// including changes not staged for commit, changes to be committed, and untracked files.
/// It prints the status information to the console.
///
pub fn handle_status() {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };

    let git_dir = match find_git_directory(&mut current_dir, GIT_DIR) {
        Some(dir) => dir,
        None => {
            eprintln!("Error al obtener el git dir");
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

    let (index, commit_tree) = match load_index_and_commit_tree(&git_dir) {
        Ok(tuple) => tuple,
        Err(err) => {
            eprintln!("Error cargando el índice y el árbol de commit: {:?}", err);
            return;
        }
    };
    print_branch_status(&branch_name);

    if let Ok(working_dir) = get_working_directory(&git_dir) {
        if let Err(err) = print_not_staged_for_commit(&index, working_dir.as_ref()) {
            eprintln!(
                "Error al imprimir los cambios no preparados para commit: {:?}",
                err
            );
        }
    } else {
        eprintln!("Error al obtener el directorio de trabajo.");
    }

    if let Err(err) = print_changes_to_be_committed(&index, &commit_tree) {
        eprintln!(
            "Error al imprimir los cambios preparados para commit: {:?}",
            err
        );
    }

    if let Ok(working_dir) = get_working_directory(&git_dir) {
        if let Err(err) = print_untracked_files(&current_dir, working_dir.as_ref(), &index) {
            eprintln!("Error al imprimir los archivos no rastreados: {:?}", err);
        }
    } else {
        eprintln!("Error al obtener el directorio de trabajo.");
    }
}

/// Prints the current branch status to the console.
///
/// This function displays information about the current Git branch,
/// emphasizing the branch name with green color.
///
/// # Arguments
///
/// * `branch_name` - The name of the current Git branch.
///
pub fn print_branch_status(branch_name: &str) {
    println!();
    print!("On branch \x1b[32m{}", branch_name);
    print!("\n\n");
}

/// Handles the 'git add' command, adding specified files to the staging area.
///
/// This function extracts the Git directory and Git ignore path, then calls the 'add::add' function
/// to add the specified files to the index (staging area).
///
/// # Arguments
///
/// * `args` - A vector of command-line arguments, where the second element is the file or directory to add.
///
fn handle_add(args: Vec<String>) {
    let (git_dir, git_ignore_path) =
        match crate::gui::repository_window::find_git_directory_and_ignore() {
            Ok((dir, ignore_path)) => (dir, ignore_path),
            Err(err) => {
                eprintln!("Error: {:?}", err);
                return;
            }
        };
    let index_path: String = git_dir.to_string() + "/" + INDEX;
    if &args[2] == "." {
        match add::add(
            "None",
            &index_path,
            &git_dir,
            &git_ignore_path,
            Some(vec![".".to_string()]),
        ) {
            Ok(_) => {}
            Err(_err) => {
                println!("Hubo un error");
                println!("{:?}", _err);
            }
        };
    } else {
        match add::add(&args[2], &index_path, &git_dir, &git_ignore_path, None) {
            Ok(_) => {}
            Err(_err) => {}
        };
    }
}

/// Handles the 'git rm' command, removing specified files from the working directory and index.
///
/// This function retrieves the current directory, Git directory, and index path, then calls the 'git_rm'
/// function to remove the specified files from the working directory and index.
///
/// # Arguments
///
/// * `args` - A vector of command-line arguments, where the second element is the file or directory to remove.
///
fn handle_rm(args: Vec<String>) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el git dir.\n {:?}", err);
            return;
        }
    };

    let index_path = format!("{}/{}", git_dir, INDEX);
    let git_dir_parent = match Path::new(&git_dir).parent() {
        Some(dir) => dir,
        None => {
            eprintln!("Fatal error on git rm");
            return;
        }
    };

    let git_ignore_path = format!("{}/{}", git_dir_parent.to_string_lossy(), GIT_IGNORE);

    match git_rm(&args[2], &index_path, &git_dir, &git_ignore_path) {
        Ok(_) => {}
        Err(_err) => {}
    }
}

/// Handles the 'git commit' command, creating a new commit with the specified message.
///
/// This function uses the 'find_git_directory_and_ignore' function to retrieve the Git directory
/// and ignore file path. It then calls the 'new_commit' function to create a new commit with the
/// specified message.
///
/// # Arguments
///
/// * `args` - A vector of command-line arguments, where the fourth element is the commit message.
///
fn handle_commit(args: Vec<String>) {
    let (git_dir, git_ignore_path) =
        match crate::gui::repository_window::find_git_directory_and_ignore() {
            Ok((dir, ignore_path)) => (dir, ignore_path),
            Err(err) => {
                eprintln!("Error: {:?}", err);
                return;
            }
        };
    match new_commit(&git_dir, &args[3], &git_ignore_path) {
        Ok(_) => {}
        Err(_err) => {}
    };
}

/// Get the working directory based on the Git directory.
///
/// # Arguments
///
/// * `git_dir` - The Git directory.
///
/// # Returns
///
/// * `Ok(String)` - The working directory if successful.
/// * `Err(io::Error)` - An error if the working directory cannot be obtained.
fn get_working_directory(git_dir: &str) -> io::Result<String> {
    match Path::new(git_dir).parent() {
        Some(parent) => Ok(parent.to_string_lossy().to_string()),
        None => Err(io::Error::new(
            io::ErrorKind::Other,
            "Error al obtener el working dir",
        )),
    }
}

/// Handle different checkout options.
///
/// # Arguments
///
/// * `git_dir` - The path to the Git directory.
/// * `working_dir` - The working directory.
/// * `option` - The checkout option.
/// * `args` - Additional arguments.
///
/// # Returns
///
/// * `Ok(())` - If the checkout operation is successful.
/// * `Err(io::Error)` - If there is an error during the checkout operation.
fn handle_checkout_option(
    git_dir: &Path,
    working_dir: &str,
    option: &str,
    args: Vec<String>,
) -> io::Result<()> {
    match option {
        "-b" => create_and_checkout_branch(git_dir, working_dir, &args[3]),
        "-B" => create_or_reset_branch(git_dir, working_dir, &args[3]),
        "--detach" => checkout_commit_detached(git_dir, working_dir, &args[3]),
        "-f" => force_checkout(git_dir, &args[3]),
        _ => checkout_branch(git_dir, working_dir, option),
    }
}

/// Handles the 'git checkout' command, allowing various options such as creating a new branch,
/// switching branches, or checking out a specific commit.
///
/// This function retrieves the Git directory, working directory, and command-line arguments. It then
/// interprets the provided options and calls corresponding functions to perform the checkout operation.
///
/// # Arguments
///
/// * `args` - A vector of command-line arguments, where the third element is the checkout option
///            ('-b', '-B', '--detach', '-f') and the fourth element is the branch or commit to checkout.
///
pub fn handle_checkout(args: Vec<String>) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el git dir.\n {:?}", err);
            return;
        }
    };

    let working_dir = match get_working_directory(&git_dir) {
        Ok(working_dir) => working_dir,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    if args.len() < 2 {
        eprintln!("Usage: my_git_checkout <option> <branch_or_commit>");
        return;
    }

    let option = &args[2];
    let git_dir1 = Path::new(&git_dir);
    let destination = &args;

    if let Err(err) = handle_checkout_option(git_dir1, &working_dir, option, destination.to_vec()) {
        match err.kind() {
            std::io::ErrorKind::UnexpectedEof => {
                eprintln!(" ");
            }
            _ => {
                eprintln!("Error cambiar de rama : {:?}", err);
            }
        }
    }
}

/// Handles the 'git log' command, displaying commit history for the repository.
///
/// This function retrieves the current directory, finds the Git directory, and calls the 'git log'
/// function to obtain an iterator over the commit logs. It then prints the logs using the
/// 'print_logs' function.
///
fn handle_log() {
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };
    let git_dir = match find_git_directory(&mut current_dir, GIT_DIR) {
        Some(git_dir) => git_dir,
        None => {
            eprintln!("Error al obtener el git dir");
            return;
        }
    };
    println!("Current dir {}", current_dir.to_string_lossy());
    println!("Git dir {}", git_dir);
    let log_iter = match log::log(None, &git_dir, 10, 0, false) {
        Ok(iter) => iter,
        Err(_e) => {
            eprintln!("Error en git log.");
            return;
        }
    };
    print_logs(log_iter);
}

/// Handles the 'clone' command for the custom Git implementation.
///
/// This function takes a vector of command-line arguments (`_args`) and performs
/// the necessary steps to clone a remote Git repository into the current directory.
///
/// # Arguments
///
/// * `_args` - A vector of command-line arguments where the URL of the remote Git
///   repository is expected at index 2.
///
fn handle_clone(_args: Vec<String>) {
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };
    let url_text = &_args[2];
    //The remote repo url is the first part of the URL, up until the last '/'.
    let remote_repo_url = match url_text.rsplit_once('/') {
        Some((string, _)) => string,
        None => "",
    };

    //The remote repository name is the last part of the URL.
    let remote_repo_name = url_text.split('/').last().unwrap_or("");
    let _ = git_clone(
        remote_repo_url,
        remote_repo_name,
        "localhost",
        current_dir.to_str().expect("Error "),
    );
}

/// Handles the 'fetch' command, which fetches changes from a remote repository.
///
/// # Arguments
///
/// * `_args` - A vector of command-line arguments.
///
fn handle_fetch(_args: Vec<String>) {
    let _current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el directorio actual: {:?}", err);
            return;
        }
    };
    let url_text = &_args[2];
    let _remote_repo_url = match url_text.rsplit_once('/') {
        Some((string, _)) => string,
        None => "",
    };

    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("{:?}", error.to_string());
            return;
        }
    };

    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            eprintln!("No pudimos obtener el working dir");
            return;
        }
    };

    let remote_repo_name = url_text.split('/').last().unwrap_or("");
    let result = git_fetch(Some(remote_repo_name), HOST, &working_dir);

    match result {
        Ok(_) => println!("Fetch successful!"),
        Err(err) => eprintln!("Error during fetch: {:?}", err),
    }
}

/// Handles the 'git merge' command, merging changes from one branch into the current branch.
///
/// This function retrieves the current directory, finds the Git directory, and calls the 'git merge'
/// function to perform a merge operation. It requires the name of the branch to be merged as an argument.
///
/// # Arguments
///
/// * `args` - A vector of strings containing command-line arguments, where the second element is
///            expected to be the name of the branch to be merged.
///
fn handle_merge(args: Vec<String>) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el git dir.\n {:?}", err);
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

    match git_merge(&branch_name, &args[2], &git_dir, &working_dir) {
        Ok(_) => {}
        Err(_e) => {
            eprintln!("Error en git merge.");
        }
    };
}

/// Handles the 'git remote' command, allowing the user to manage remote repositories.
///
/// This function retrieves the current directory, finds the Git directory, and loads the Git
/// configuration file. It then calls the 'git remote' function to perform remote repository
/// management operations based on the provided command-line arguments.
///
/// # Arguments
///
/// * `args` - A vector of strings containing command-line arguments, starting from the third element,
///            representing the 'git remote' subcommand and its options.
///
fn handle_remote(args: Vec<String>) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el git dir.\n {:?}", err);
            return;
        }
    };

    let mut config = match Config::load(&git_dir) {
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
        }
    }
}

/// Handles the 'git pull' command, allowing the user to fetch and merge changes from a remote repository.
///
/// This function retrieves the current directory, finds the Git directory, and determines the working directory.
/// It then gets the current branch name, and finally, it calls the 'git pull' function to fetch and merge changes
/// from the specified remote repository.
///
fn handle_pull() {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el git dir.\n {:?}", err);
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
    match git_pull(&branch_name, &working_dir, None, "localhost") {
        Ok(_) => {
            println!("Pulled successfully");
        }
        Err(_e) => {
            println!("Error on git pull");
        }
    };
}

/// Handles the 'git push' command, allowing the user to push changes to a remote repository.
///
/// This function retrieves the current directory, finds the Git directory, and gets the current branch name.
/// It then calls the 'git push' function to push changes to the remote repository associated with the current branch.
///
fn handle_push() {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Error al obtener el git dir.\n {:?}", err);
            return;
        }
    };
    let branch_name = match get_branch_name(&git_dir) {
        Ok(name) => name,
        Err(_e) => {
            eprintln!("Error obteniendo branch name.");
            return;
        }
    };
    match push::git_push(&branch_name, &git_dir) {
        Ok(_) => {
            println!("Push ok")
        }
        Err(_e) => {
            eprintln!("Push not ok")
        }
    };
}

/// Handles the 'git branch' command, allowing the user to list or create branches.
///
/// This function checks the number of arguments provided. If there are only two arguments,
/// it calls the 'git_branch' function to list existing branches. If there are more than two arguments,
/// it assumes the user wants to create a new branch with the specified name and calls the 'git_branch' function accordingly.
///
/// # Arguments
///
/// * `args` - A vector of command-line arguments.
///
fn handle_branch(args: Vec<String>) {
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    if args.len() == 2 {
        let result = git_branch(None, None, None, &mut io::stdout());
        match result {
            Ok(_) => {}
            Err(_e) => {
                eprintln!("Error showing branches.");
            }
        }
    } else if args.len() == 3 {
        if args[2] == "-l" {
            let result = git_branch(None, None, None, &mut io::stdout());
            match result {
                Ok(_) => {}
                Err(_e) => {
                    eprintln!("Error showing branches.");
                }
            }
        } else {
            let name = args[2];
            let result = git_branch(Some(name.to_string()), None, None, &mut io::stdout());
            match result {
                Ok(_) => {
                    println!("Branch {name} successfully created!");
                }
                Err(_e) => {
                    eprintln!("{}", _e)
                }
            }
        }
    } else if args.len() == 4 {
        if args[2] == "-m" {
            match git_branch(None, Some("-m"), Some(args[3]), &mut io::stdout()) {
                Ok(_) => {
                    println!("Current branch name modified to {}", args[3]);
                }
                Err(e) => {
                    eprintln!("{}", e)
                }
            }
        } else if args[2] == "-c" {
            match git_branch(
                Some(args[3].to_string()),
                Some("-c"),
                None,
                &mut io::stdout(),
            ) {
                Ok(_) => {
                    println!("Branch {} successfully created!", args[3]);
                }
                Err(e) => {
                    eprintln!("{}", e)
                }
            }
        } else if args[2] == "-d" {
            match git_branch(
                Some(args[3].to_string()),
                Some("-d"),
                None,
                &mut io::stdout(),
            ) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e)
                }
            }
        } else {
            let name = args[2];
            let new_name = &args[3].to_string();
            let result = git_branch(
                Some(name.to_string()),
                None,
                Some(new_name),
                &mut io::stdout(),
            );
            match result {
                Ok(_) => {
                    println!("Branch {name} succesfully created from {new_name}");
                }
                Err(_e) => {
                    eprintln!("{}", _e)
                }
            }
        }
    } else {
        handle_branch_options(args);
    }
}

//git branch
//git branch -l
//git branch -c branch
//git branch new existent
//git branch -m new_name
//git branch -m branch new_name
//git branch -d name

/// Handles various options for Git branch operations.
///
/// This function takes a vector of command-line arguments (`args`) and performs different actions
/// based on the specified options. The supported options include:
///
/// - `-m`: Rename a Git branch.
/// - `-d`: Delete one or more Git branches.
/// - `-c`: Create a new Git branch.
///
/// # Arguments
///
/// - `args`: A vector of command-line arguments where the first element is the program name,
/// the second element is the main command (e.g., "git"), and the third element is the option
/// specifying the Git branch operation ("-m", "-d", "-c"). The subsequent elements depend on
/// the chosen option.
///
fn handle_branch_options(args: Vec<&str>) {
    match args[2] {
        "-m" => {
            if args.len() == 5 {
                match git_branch(
                    Some(args[3].to_string()),
                    Some("-m"),
                    Some(args[4]),
                    &mut io::stdout(),
                ) {
                    Ok(_) => {
                        println!("Branch {} successfully modified to {}", args[3], args[4]);
                    }
                    Err(e) => {
                        eprintln!("{}", e)
                    }
                }
            } else {
                eprintln!("fatal: too many arguments for a rename operation");
            }
        }
        "-d" => {
            let new_args: Vec<&&str> = args.iter().skip(3).collect();
            for arg in new_args {
                match git_branch(Some(arg.to_string()), Some("-d"), None, &mut io::stdout()) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e)
                    }
                }
            }
        }
        "-c" => {
            let new_args: Vec<&&str> = args.iter().skip(3).collect();
            for arg in new_args {
                match git_branch(Some(arg.to_string()), Some("-c"), None, &mut io::stdout()) {
                    Ok(_) => {
                        println!("Branch {arg} succesfully created!");
                    }
                    Err(e) => {
                        eprintln!("{}", e)
                    }
                }
            }
        }
        _ => {
            eprintln!("Invalid option!");
        }
    }
}

/// Handles the initialization of a Git repository.
///
/// # Arguments
///
/// * `args` - A vector of command-line arguments.
pub fn handle_init(args: Vec<String>) {
    // Extract configuration parameters from command-line arguments
    let (current_directory, initial_branch, template_directory) = extract_init_params(&args);

    // Initialize the Git repository with the extracted parameters
    if let Err(err) = git_init(
        &current_directory,
        GIT_DIR,
        &initial_branch,
        template_directory,
    ) {
        eprintln!("Error al inicializar el repositorio Git: {}", err);
    }
}

/// Extracts initialization parameters from command-line arguments.
///
/// # Arguments
///
/// * `args` - A vector of command-line arguments.
///
/// # Returns
///
/// A tuple containing the current directory, initial branch, and template directory.
fn extract_init_params(args: &Vec<String>) -> (String, String, Option<&str>) {
    let mut current_directory = match std::env::current_dir() {
        Ok(dir) => dir.to_string_lossy().to_string(),
        Err(_) => {
            eprintln!("Current dir not found");
            return (String::new(), String::from("main"), None);
        }
    };

    let mut initial_branch = String::from("main");
    let mut template_directory: Option<&str> = None;

    let mut index = 2;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "-b" | "--initial-branch" => {
                if index + 1 < args.len() {
                    initial_branch = args[index + 1].clone();
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

    (current_directory, initial_branch, template_directory)
}
