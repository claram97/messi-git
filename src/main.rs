use std::{io::{self, Write}, fs::File};

use messi::{parse_commands::{get_user_input, handle_git_command, parse_git_command}, init, commit, tree_handler::{self, print_tree_console}, cat_file, add};

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

fn main() {
    // let mut file = File::create("tests/checkout/archivo.txt").unwrap();
    // file.write_all(b"Hello Wo22rld!").unwrap();
    // let mut file = File::create("tests/checkout/otro_archivo.txt").unwrap();
    // file.write_all(b"Hellooo World!").unwrap();
    // let mut file = File::create("tests/checkout/directorio_para_probar/nuevo_archivo.txt").unwrap();
    // file.write_all(b"Hellooo World!").unwrap();
    
    // add::add("tests/checkout/archivo.txt", "tests/checkout/.mgit/index", "tests/checkout/.mgit", "", None);
    // add::add("tests/checkout/otro_archivo.txt", "tests/checkout/.mgit/index", "tests/checkout/.mgit", "", None);
    // add::add("tests/checkout/directorio_para_probar/nuevo_archivo.txt", "tests/checkout/.mgit/index", "tests/checkout/.mgit", "", None);

    // let result = commit::new_commit("tests/checkout/.mgit", "Hola", "");
    // println!("{:?}", result);
    // let commit_hash = &result.unwrap();
    // let result = cat_file::cat_file(commit_hash, "tests/checkout/.mgit",&mut io::stdout());
    let commit_hash = "6ba70ff970e11ec8e5824054a6a94e8900e2884d";
    let tree = tree_handler::load_tree_from_commit(commit_hash, "tests/checkout/.mgit").unwrap();

    // print_tree_console(&tree, 0);

    let resultt = tree.delete_directories("");
    let result = tree.create_directories("", "tests/checkout/.mgit");
    
    println!("{:?}", result);
}