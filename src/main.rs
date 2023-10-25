use std::{io::{self, Write}, fs::File};

use messi::{parse_commands::{get_user_input, handle_git_command, parse_git_command}, utils};

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
        None => return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Git directory not found\n",
        ))
    };
    //Esto lo tiene que crear init
    /*let path = (&git_dir).to_string() + "/config";
    let mut file = File::create(&path)?;
    file.write_all(b"[core]\n")?;
    file.write_all(b"    repositoryformatversion = 0\n")?;
    file.write_all(b"    filemode = true\n")?;
    file.write_all(b"    bare = false\n")?;
    file.write_all(b"    logallrefupdates = true\n")?;
    drop(file);*/
    Ok(())
}