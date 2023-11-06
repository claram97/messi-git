use std::{env, io};

use messi::gui::run_main_window;
use messi::parse_commands::get_user_input;
use messi::parse_commands::{handle_git_command, parse_git_command};
use messi::server;

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
fn run_without_gui() -> io::Result<()> {
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
            // Si se ingresa "exit", salimos del bucle.
            break;
        }

        if let Some(git_command) = parse_git_command(second_argument) {
            handle_git_command(git_command, args);
        }
    }
    Ok(())
}

// fn run_without_gui() -> io::Result<()> {
//     let args = get_user_input();
//     let second_argument = match args.get(1) {
//         Some(arg) => arg,
//         None => {
//             return Err(io::Error::new(
//                 io::ErrorKind::Other,
//                 "No se ha ingresado el segundo argumento.\n",
//             ));
//         }
//     };

//     if let Some(git_command) = parse_git_command(second_argument) {
//         handle_git_command(git_command, args);
//     }
//     Ok(())
// }

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
