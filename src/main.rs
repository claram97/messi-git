use std::{env, io};

use messi::gui::main_window::run_main_window;
use messi::parse_commands::get_user_input;
use messi::parse_commands::{handle_git_command, parse_git_command};
use messi::server;

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
    println!("Por favor, inicie un repositorio de Git utilizando 'git init'.");

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
            if let Some(git_command) = parse_git_command(second_argument) {
                if args.len() == 2 {
                    handle_git_command(git_command, args);
                } else {
                    env::set_current_dir(&args[2]).unwrap();
                    handle_git_command(git_command, args);
                }
            }
            break;
        }
    }

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
        server::run(&args[2], &args[3], &args[4], ".mgit")?;
    } else if args.len() == 1 {
        run_without_gui()?;
    }
    Ok(())
}
