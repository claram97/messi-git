use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

use messi::{
    config::Config,
    parse_commands::{get_user_input, handle_git_command, parse_git_command},
    remote_handler::Remote,
    utils, remote::{git_remote, self},
};

// fn main() {
//     let args = get_user_input();
//     let second_argument = match args.get(1) {
//         Some(arg) => arg,
//         None => {
//             eprintln!("No se ha ingresado el segundo argumento.");
//             return;
//         }
//     };

//     if let Some(git_command) = parse_git_command(second_argument) {
//         handle_git_command(git_command, args);
//     }
// }

fn main() -> io::Result<()> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir = match utils::find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ))
        }
    };

    //Esto lo tiene que crear init
    let path = "/home/claram97/taller/23C2-messi/prueba/config";
    let mut file = File::create(&path)?;
    file.write_all(b"[core]\n")?;
    file.write_all(b"    repositoryformatversion = 0\n")?;
    file.write_all(b"    filemode = true\n")?;
    file.write_all(b"    bare = false\n")?;
    file.write_all(b"    logallrefupdates = true\n")?;
    drop(file);

    let mut config = Config::load("/home/claram97/taller/23C2-messi/prueba")?;
    println!("len: {}", config.remotes.len());
    let line = vec!["add","new_remote","my_url"];
    remote::git_remote(&mut config, line, &mut io::stdout())?;
    println!("len: {}", config.remotes.len());
    let line = vec!["get-url","new_remote"];
    let result = remote::git_remote(&mut config, line, &mut io::stdout());
    if result.is_ok() {
        println!("Ok!");
    }
    // println!("len: {}", config.remotes.len());
    // let line = vec!["rename","new_remote","new_name"];
    // let _result = remote::git_remote(&mut config, line, &mut io::stdout())?;
    // println!("len: {}", config.remotes.len());

 
    Ok(())
}
