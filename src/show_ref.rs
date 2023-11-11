use std::{io::{self, Write}, path::Path, fs};

use std::fs::File;
use std::io::prelude::*;

pub fn git_show_ref(git_dir: &str, line: Vec<String>, output: &mut impl Write) -> io::Result<()> {
    if line.len() == 1 {
        // Caso base
        let heads_path = format!("{}/{}", git_dir, "refs/heads");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        for entry in fs::read_dir(&heads_path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_file() {
                let file_name = match file_path.file_name() {
                    Some(name) => name.to_string_lossy().to_string(),
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::Interrupted,
                            "Fatal error",
                        ));
                    }
                };
             
                let mut file = fs::File::open(&file_path)?;

                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                writeln!(output, "{}\trefs/heads/{}\n", contents.trim(), file_name)?;
            }
        }

        for entry in fs::read_dir(&tags_path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_file() {
                let file_name = match file_path.file_name() {
                    Some(name) => name.to_string_lossy().to_string(),
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::Interrupted,
                            "Fatal error",
                        ));
                    }
                };
             
                let mut file = fs::File::open(&file_path)?;

                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                writeln!(output, "{}\trefs/tags/{}\n", contents.trim(), file_name)?;
            }
        }

    } else if line.len() == 2 {
        if line[1].eq("--heads") {
            let heads_path = format!("{}/{}", git_dir, "refs/heads");
            for entry in fs::read_dir(&heads_path)? {
                let entry = entry?;
                let file_path = entry.path();
    
                if file_path.is_file() {
                    let file_name = match file_path.file_name() {
                        Some(name) => name.to_string_lossy().to_string(),
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::Interrupted,
                                "Fatal error",
                            ));
                        }
                    };
                 
                    let mut file = fs::File::open(&file_path)?;
    
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
    
                    writeln!(output, "{}\trefs/heads/{}\n", contents.trim(), file_name)?;
                }
            }
        } else if line[1].eq("--tags") {
            let tags_path = format!("{}/{}", git_dir, "refs/tags");
            for entry in fs::read_dir(&tags_path)? {
                let entry = entry?;
                let file_path = entry.path();
    
                if file_path.is_file() {
                    let file_name = match file_path.file_name() {
                        Some(name) => name.to_string_lossy().to_string(),
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::Interrupted,
                                "Fatal error",
                            ));
                        }
                    };
                 
                    let mut file = fs::File::open(&file_path)?;
    
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
    
                    writeln!(output, "{}\trefs/tags/{}\n", contents.trim(), file_name)?;
                }
            }
        } else if line[1].eq("--hash") {
            let heads_path = format!("{}/{}", git_dir, "refs/heads");
            let tags_path = format!("{}/{}", git_dir, "refs/tags");
            for entry in fs::read_dir(&heads_path)? {
                let entry = entry?;
                let file_path = entry.path();
    
                if file_path.is_file() {
                    match file_path.file_name() {
                        Some(name) => name.to_string_lossy().to_string(),
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::Interrupted,
                                "Fatal error",
                            ));
                        }
                    };
                 
                    let mut file = fs::File::open(&file_path)?;
    
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
    
                    writeln!(output, "{}\n", contents.trim())?;
                }
            }
    
            for entry in fs::read_dir(&tags_path)? {
                let entry = entry?;
                let file_path = entry.path();
    
                if file_path.is_file() {
                    match file_path.file_name() {
                        Some(name) => name.to_string_lossy().to_string(),
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::Interrupted,
                                "Fatal error",
                            ));
                        }
                    };
                 
                    let mut file = fs::File::open(&file_path)?;
    
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
    
                    writeln!(output, "{}\n", contents.trim())?;
                }
            }
        } else if line[1].eq("--verify") {
            writeln!(output, "fatal: --verify requires a reference")?;
        } 
        else if line[1].starts_with("--") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid option specified",
            ));
        }
    } else if line.len() >= 3 {
        if line[1].eq("--verify") {
            for i in 2..line.len() {
                let path_to_verify = format!("{}/{}", git_dir, &line[i]);
                let path = Path::new(&path_to_verify);
                
                if !path.exists() {
                    writeln!(output, "fatal: '{}' - not a valid ref\n", &line[i])?;
                } else {
                    let mut file = File::open(&path)?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    writeln!(output, "{}\t{}\n", &line[i], contents.trim())?;
                }
            }
        } 
        else if line[1].starts_with("--") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid option specified",
            ));
        }

    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid number of arguments",
        ));
    }
    Ok(())
}

// use std::{
//     fs::{self, File},
//     io::{self, Read, Write},
//     path::{Path, PathBuf},
// };

// pub fn git_show_ref(git_dir: &str, line: Vec<String>, output: &mut impl Write) -> io::Result<()> {
//     match line.len() {
//         1 => {
//             show_refs(&format!("{}/{}", git_dir, "refs/heads"), "refs/heads", output)?;
//             show_refs(&format!("{}/{}", git_dir, "refs/tags"), "refs/tags", output)?;
//         }
//         2 => {
//             match line[1].as_str() {
//                 "--heads" => show_refs(&format!("{}/{}", git_dir, "refs/heads"), "refs/heads", output)?,
//                 "--tags" => show_refs(&format!("{}/{}", git_dir, "refs/tags"), "refs/tags", output)?,
//                 "--hash" => {
//                     show_hashes(&format!("{}/{}", git_dir, "refs/heads"), output)?;
//                     show_hashes(&format!("{}/{}", git_dir, "refs/tags"), output)?;
//                 }
//                 "--verify" => writeln!(output, "fatal: --verify requires a reference")?,
//                 _ if line[1].starts_with("--") => {
//                     return Err(io::Error::new(
//                         io::ErrorKind::InvalidInput,
//                         "Invalid option specified",
//                     ));
//                 }
//                 _ => {}
//             }
//         }
//         n if n >= 3 => {
//             match line[1].as_str() {
//                 "--verify" => verify_refs(git_dir, &line[2..], output)?,
//                 _ if line[1].starts_with("--") => {
//                     return Err(io::Error::new(
//                         io::ErrorKind::InvalidInput,
//                         "Invalid option specified",
//                     ));
//                 }
//                 _ => {}
//             }
//         }
//         _ => {
//             return Err(io::Error::new(
//                 io::ErrorKind::InvalidInput,
//                 "Invalid number of arguments",
//             ));
//         }
//     }
//     Ok(())
// }

// fn show_refs(refs_path: &str, prefix: &str, output: &mut impl Write) -> io::Result<()> {
//     for entry in fs::read_dir(refs_path)? {
//         let entry = entry?;
//         if let Some(file_name) = entry.file_name().to_str() {
//             let file_path = entry.path();
//             if file_path.is_file() {
//                 let mut file = File::open(&file_path)?;
//                 let mut contents = String::new();
//                 file.read_to_string(&mut contents)?;
//                 writeln!(output, "{}\t{}/{}\n", contents.trim(), prefix, file_name)?;
//             }
//         } else {
//             return Err(io::Error::new(io::ErrorKind::Interrupted, "Fatal error"));
//         }
//     }
//     Ok(())
// }

// fn show_hashes(refs_path: &str, output: &mut impl Write) -> io::Result<()> {
//     for entry in fs::read_dir(refs_path)? {
//         let entry = entry?;
//         if entry.path().is_file() {
//             let mut file = File::open(&entry.path())?;
//             let mut contents = String::new();
//             file.read_to_string(&mut contents)?;
//             writeln!(output, "{}\n", contents.trim())?;
//         }
//     }
//     Ok(())
// }

// fn verify_refs(git_dir: &str, refs: &[String], output: &mut impl Write) -> io::Result<()> {
//     for ref_name in refs {
//         let path_to_verify = format!("{}/{}", git_dir, ref_name);
//         let path = Path::new(&path_to_verify);

//         if !path.exists() {
//             writeln!(output, "fatal: '{}' - not a valid ref\n", ref_name)?;
//         } else {
//             let mut file = File::open(&path)?;
//             let mut contents = String::new();
//             file.read_to_string(&mut contents)?;
//             writeln!(output, "{}\t{}\n", ref_name, contents.trim())?;
//         }
//     }
//     Ok(())
// }
