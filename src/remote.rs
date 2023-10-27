//git remote add origin https://ejemplo.com/repo.git
//git remote remove origin
//git remote set-url origin https://nueva-url.com/repo.git
//git remote get-url origin
//git remote rename nombre-viejo nombre-nuevo


use std::io::{Write, self};

use crate::config::Config;

pub fn git_remote(git_dir_path: &str, line : Vec<&str>, output: &mut impl Write) -> io::Result<()> {
    if (line.len() != 2) && (line.len() != 3) {
        //Error         
    }
    let mut config = Config::load(git_dir_path)?;
    let _comand = match line[0] {
        "add" => {
            if line.len() != 3 {
                //error
            }
            let fetch = "fetch".to_string(); //Esto hay que ver bien de dónde sale para armarlo como coresponde
            config.add_remote(line[1].to_string(), line[2].to_string(), fetch)?;
        },
        "remove" => {
            if line.len() != 2 {
                //error
            }
            config.remove_remote(line[1])?;
        },
        "set-url" => {
            if line.len() != 3{
                //error
            }
            config.set_url(line[1], line[2], output)?;

        },
        "get-url" => {
            if line.len() != 2 {
                //error
            }
            config.get_url(line[1],output)?;
        },
        "rename" => {
            if line.len() != 3 {
                //error
            }
            config.change_remote_name(line[1], line[2], &mut io::stdout())?;

        },
        _ => {
            let error_mesagge = format!("error: Unknown subcommand {}\n",line[0]);
            output.write_all(error_mesagge.as_bytes())?;
            // Este es el caso por defecto, si line[0] no coincide con ninguno de los casos anteriores.
            // Puedes manejarlo de acuerdo a tus necesidades.
            // Por ejemplo, mostrar un mensaje de error.
            // println!("Comando desconocido: {}", line[0]);
        }
    };


    Ok(())
}
