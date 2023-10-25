use std::{io, fs::File};

use messi::{parse_commands::{get_user_input, handle_git_command, parse_git_command}, branch, init, commit, index};

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

fn main() -> io::Result<()>{
    /*init::git_init("/home/claram97/taller/23C2-messi","branch",None)?;
    let git_dir_path = "/home/claram97/taller/23C2-messi/.mgit";
    let git_ignore_path = "/home/claram97/taller/23C2-messi/.mgitignore";
    let index_path = "/home/claram97/taller/23C2-messi/.mgit/index";
    let _file = File::create(index_path)?;
    commit::new_commit(git_dir_path, "message", git_ignore_path)?;*/
    //branch::create_new_branch("aeiou",&mut io::stdout())?;
    //branch::git_branch(Some("holi".to_string()))?;
    branch::git_branch(None)?;
    Ok(())
}
