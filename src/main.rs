use std::{io::{self, Write}, fs::File, path::{Path, PathBuf}};

use messi::{parse_commands::{get_user_input, handle_git_command, parse_git_command}, init, commit, tree_handler::{self, print_tree_console}, cat_file, add, checkout};

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

    //Create an empty index file
    let mut file = File::create("tests/checkout/.mgit/index").unwrap();
    //Rewrite HEAD
    let mut head = File::create("tests/checkout/.mgit/HEAD").unwrap();
    head.write_all(b"ref: refs/heads/main").unwrap();

    let mut file_1 = File::create("tests/checkout/archivo.txt").unwrap();
    file_1.write_all(b"Hello Wo22rld!").unwrap();
    let mut file_2 = File::create("tests/checkout/otro_archivo.txt").unwrap();
    file_2.write_all(b"Hellooo World!").unwrap();

    //If dir doesnt exist, create it
    if !Path::new("tests/checkout/directorio_para_probar").exists() {
        std::fs::create_dir("tests/checkout/directorio_para_probar").unwrap();
    }
    let mut file_3 = File::create("tests/checkout/directorio_para_probar/nuevo_archivo.txt").unwrap();
    file_3.write_all(b"Hellooo World!").unwrap();
    
    add::add("tests/checkout/archivo.txt", "tests/checkout/.mgit/index", "tests/checkout/.mgit", "", None);
    add::add("tests/checkout/otro_archivo.txt", "tests/checkout/.mgit/index", "tests/checkout/.mgit", "", None);
    add::add("tests/checkout/directorio_para_probar/nuevo_archivo.txt", "tests/checkout/.mgit/index", "tests/checkout/.mgit", "", None);

    let result = commit::new_commit("tests/checkout/.mgit", "Hola", "");
    println!("{:?}", result);

    let mut file_4 = File::create("tests/checkout/archivo3.txt").unwrap();
    file_4.write_all(b"Helleeeo World!").unwrap();

    //Edit file
    let mut edit = File::write(&mut file_1, b"Este archivo fue cambiado").unwrap();

    add::add("tests/checkout/archivo3.txt", "tests/checkout/.mgit/index", "tests/checkout/.mgit", "", None);
    add::add("tests/checkout/archivo.txt", "tests/checkout/.mgit/index", "tests/checkout/.mgit", "", None);

    let result2 = commit::new_commit("tests/checkout/.mgit", "Hola2", "");
    println!("{:?}", result2);

    //Get current directory
    let current_dir = std::env::current_dir().unwrap();
    let git_dir: PathBuf = current_dir.join("tests/checkout/.mgit");
    let result3 = checkout::checkout_commit_detached(&git_dir, &result.unwrap());
}