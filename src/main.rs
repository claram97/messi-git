use std::{env, io};

use messi::configuration::GIT_DIR;
use messi::gui::main_window::run_main_window;
use messi::parse_commands::get_user_input;
use messi::parse_commands::{handle_git_command, parse_git_command};
use messi::{api, server};
use std::io::Write;

/// Runs the application with a graphical user interface (GUI) using GTK.
///
/// This function initializes the GTK library and attempts to create and run the main application
/// window. If the initialization or window creation fails, an error is returned. Otherwise, the
/// function enters the GTK main loop and continues until the application exits.
///
/// # Returns
///
/// A `std::io::Result<()>` indicating whether the GUI application ran successfully.
///
fn run_with_gui() -> io::Result<()> {
    if gtk::init().is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to initialize GTK.\n",
        ));
    }

    run_main_window().map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    gtk::main();
    Ok(())
}

/// Runs the application in a command-line mode without a graphical user interface (GUI).
///
/// This function prompts the user to initiate a Git repository using 'git init'. It then enters a
/// loop where the user can provide Git commands. The loop continues until the user enters 'exit'.
/// If the second argument is 'init', the function attempts to set the current directory based on
/// the user input and then processes the Git command. Otherwise, it processes the provided Git
/// command directly.
///
/// # Returns
///
/// A `std::io::Result<()>` indicating whether the command-line application ran successfully.
///
fn run_without_gui() -> io::Result<()> {
    print_init_message();
    loop {
        let args = get_user_input();
        let second_argument = match args.get(1) {
            Some(arg) => arg,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "No se ha ingresado el segundo argumento.\n",
                ));
            }
        };

        if second_argument == "exit" {
            break;
        }
        if second_argument == "init" {
            let args_clone = args.clone();
            handle_init_command(args_clone)?;
            log_to_file(second_argument, "Command executed")?;
        } else if let Some(git_command) = parse_git_command(second_argument) {
            let args_clone = args.clone();
            handle_git_command(git_command, args_clone);
            log_to_file(second_argument, "Command executed")?;
        }
    }

    process_user_input()
}

/// Logs a command and its associated message to a file.
///
/// # Arguments
///
/// * `command` - A string representing the command being logged.
/// * `message` - A string containing the message to be logged along with the command.
///
/// # Returns
///
/// Returns a `Result` indicating the success or failure of the logging operation.
///
fn log_to_file(command: &str, message: &str) -> Result<(), std::io::Error> {
    let mut logger = messi::logger::Logger::new("logger_comands.txt")?;

    let full_message = format!("Command '{}': {}", command, message);
    logger.write_all(full_message.as_bytes())?;

    logger.flush()?;

    Ok(())
}

/// Handles the "init" command, initializing a new Git repository.
///
/// This function checks the provided arguments to determine whether to initialize
/// a repository in the current directory or a specified directory.
///
/// # Arguments
///
/// * `args` - A vector of strings representing the command-line arguments. The
///   first argument is expected to be the command ("init").
///
/// # Errors
///
/// Returns an `io::Result` indicating success or failure. Failure occurs if the
/// provided arguments are invalid or if there is an error setting the current directory.
///
fn handle_init_command(args: Vec<String>) -> io::Result<()> {
    if let Some(git_command) = parse_git_command(&args[1]) {
        if args.len() == 2 {
            handle_git_command(git_command, args);
        } else {
            env::set_current_dir(&args[2])?;
            handle_git_command(git_command, args);
        }
    }
    Ok(())
}

/// Prints an initialization message for a Git repository.
///
/// This function prints a message prompting the user to initialize a Git repository
/// using the 'git init' command.
///
fn print_init_message() {
    println!("Por favor, inicie un repositorio de Git utilizando 'git init'.");
}

/// Processes user input for Git commands in a loop.
///
/// This function continuously prompts the user for Git commands until the user
/// enters "exit." It parses the user input, identifies the Git command, and
/// delegates the handling of the command to the appropriate function.
///
/// # Errors
///
/// Returns an `io::Result` that may contain an `Err` variant if an I/O operation fails.
///
fn process_user_input() -> io::Result<()> {
    loop {
        let args = get_user_input();
        let second_argument = match args.get(1) {
            Some(arg) => arg,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "No se ha ingresado el segundo argumento.\n",
                ));
            }
        };

        if second_argument == "exit" {
            break;
        }

        if let Some(git_command) = parse_git_command(second_argument) {
            handle_git_command(git_command, args);
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 1 && args.len() != 2 && args.len() != 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cantidad inválida de parámetros\n",
        ));
    }

    if args.len() == 2 {
        if args[1] != "gui" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Comando no reconocido\n",
            ));
        }

        run_with_gui()?;
    } else if args.len() == 5 && args[1] == "server" {
        server::run(&args[2], &args[3], &args[4], GIT_DIR)?;
    } else if args.len() == 5 && args[1] == "server_api" {
        api::server::run(&args[2], &args[3], &args[4])?;
    } else if args.len() == 1 {
        run_without_gui()?;
    }
    Ok(())
}
