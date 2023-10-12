//These functions are already in hash_object branch.
//Check them out to read some documentation about them.

/*
->Leo el index file
->Busco los path
->Comparo e imprimo por pantalla si cambió, sino no hago nada
*/

/*
->Abro el directorio
->
->Buscar en el directorio todos los archivos que no estén en el index e imprimirlos
*/

pub(crate) const NAME_OF_INDEX_FILE: &str = "index-file";
use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::{fs, io::ErrorKind};

pub fn find_git_directory() -> Option<String> {
    if let Ok(current_dir) = env::current_dir() {
        let mut current_dir = current_dir;

        loop {
            let git_dir = current_dir.join(".git");
            if git_dir.exists() && git_dir.is_dir() {
                return Some(git_dir.display().to_string());
            }

            if !current_dir.pop() {
                break; // Llegamos al directorio raíz, no se encontró el directorio .git
            }
        }
    }

    None
}

//Actualmente cada línea del index file tiene:
//hash path
fn read_index_file(file: &mut File) -> io::Result<()> {
    let reader = BufReader::new(file);
    for line in reader.lines() {
        match line {
            Ok(line_content) => {
                let splitted_line: Vec<&str> = line_content.split_whitespace().collect();
                if let Some(path) = find_git_directory() {
                    if let Some(parent) = Path::new(&path).parent() {
                        let file_path = parent.to_string_lossy().to_string() + "/" + splitted_line[1];
                        let hash = hash_file_content(&file_path)?;
                        if !hash.eq(splitted_line[0]) {
                            println!("File {} has changed since last commit.", splitted_line[1]);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error al leer una línea: {}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}

pub fn find_files_that_changed_since_last_commit() -> io::Result<()> {
    match find_git_directory() {
        Some(dir) => {
            let file_path = dir + "/" + NAME_OF_INDEX_FILE;
            let mut file = File::open(file_path)?;
            read_index_file(&mut file)?;
            Ok(())
        }
        None => Err(io::Error::new(
            ErrorKind::NotFound,
            "Git index file couldn't be opened.",
        )),
    }
}

pub fn hash_string(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn hash_file_content(path: &str) -> io::Result<String> {
    let content = std::fs::read_to_string(path)?;
    let header = format!("blob {}\0", content.len());
    let complete = header + &content;
    Ok(hash_string(&complete))
}

fn buscar_en_directorio(
    directorio_actual: &Path,
    archivos_lista: &HashSet<String>,
    carpetas_ignoradas: &HashSet<String>,
) -> Result<(), std::io::Error> {
    for entrada in fs::read_dir(directorio_actual)? {
        let entrada = entrada?;
        let elemento_path = entrada.path();
        let elemento_path_str = elemento_path.to_string_lossy().to_string();

        if carpetas_ignoradas.iter().any(|ignorada| elemento_path_str.starts_with(ignorada)) {
            continue;
        }

        if !archivos_lista.contains(&elemento_path_str)
            && elemento_path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(true, |s| !s.starts_with('.'))
        {
            println!("No encontrado en la lista: {:?}", elemento_path);
        }

        if elemento_path.is_dir()
            && elemento_path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |s| !s.starts_with('.'))
        {
            buscar_en_directorio(&elemento_path, archivos_lista, carpetas_ignoradas)?;
        }
    }
    Ok(())
}

pub fn buscar_no_en_lista(
    directorio_base: &Path,
    archivo_lista: &str,
    archivo_carpetas_ignoradas: &str,
) -> Result<(), std::io::Error> {
    let mut archivos_lista = HashSet::new();
    let mut carpetas_ignoradas = HashSet::new();

    let lista_reader = BufReader::new(std::fs::File::open(archivo_lista)?);
    for line in lista_reader.lines() {
        let linea = line?;
        let partes: Vec<&str> = linea.splitn(2, ' ').collect();
        if partes.len() == 2 {
            archivos_lista.insert(partes[1].to_string());
        }
    }

    let ignore_reader = BufReader::new(std::fs::File::open(archivo_carpetas_ignoradas)?);
    for line in ignore_reader.lines() {
        let linea = line?;
        let complete_path = directorio_base.to_string_lossy().to_string() + &linea;
        carpetas_ignoradas.insert(complete_path);
    }

    buscar_en_directorio(directorio_base, &archivos_lista, &carpetas_ignoradas)?;
    Ok(())
}
