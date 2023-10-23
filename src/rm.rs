use crate::index::Index;
use std::fs;
use std::io;

pub fn git_rm(file_name: &str, index_path: &str, git_dir_path: &str) -> io::Result<()> {
    if let Some(mut index) = Index::load_from_path_if_exists(index_path, git_dir_path)? {
        if !index.contains(file_name) {
            eprintln!("El archivo no está en el índice.");
            return Ok(());
        }

        if let Err(err) = remove_path(&mut index, file_name) {
            eprintln!("Error al eliminar el archivo: {}", err);
            return Err(err);
        }

        if let Err(err) = index.write_file() {
            eprintln!("Error al guardar el índice: {}", err);
            return Err(err);
        }

        if let Err(err) = fs::remove_file(file_name) {
            eprintln!(
                "Error al eliminar el archivo del sistema de archivos de trabajo: {}",
                err
            );
            return Err(err);
        }
    } else {
        eprintln!("No se pudo cargar el índice.");
    }

    Ok(())
}

fn remove_directory(dir_path: &str) -> io::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            remove_directory(path.to_str().unwrap())?;
        } else {
            fs::remove_file(path.to_str().unwrap())?;
        }
    }

    fs::remove_dir(dir_path)?;

    Ok(())
}

pub fn remove_path(index: &mut Index, path: &str) -> io::Result<()> {
    if !index.contains(path) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Path not found in index: {}. Cannot remove", path),
        ));
    }

    index.remove_file(path)?;

    if fs::metadata(path)?.is_dir() {
        remove_directory(path)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_git_rm_file_not_in_index() -> io::Result<()> {
        // Prueba git_rm cuando el archivo no está en el índice
        let index_path = "ruta_al_indice";
        let git_dir_path = "ruta_al_git_dir";
        let file_name = "archivo_no_en_indice.txt";

        let result = git_rm(file_name, index_path, git_dir_path);

        assert!(result.is_ok());
        assert!(!fs::metadata(file_name).is_ok());

        Ok(())
    }
    fn setup_mgit(git_dir: &str) -> io::Result<()> {
        fs::create_dir_all(format!("{}/objects", git_dir))
    }
    #[test]
    fn test_add_path_file() -> io::Result<()> {
        let mut index = Index::new("", ".mgit");
        setup_mgit(".mgit")?;

        let path = "tests/add/dir_to_add/non_empty/a.txt";

        index.add_path(path)?;

        assert!(index.contains(path));
        Ok(())
    }

    #[test]
    fn test_git_rm_file_in_index() -> io::Result<()> {
        let index_path = "";
        let git_dir_path = ".mgit";
        let file_name = "a.txt";
        let mut index = Index::new(index_path, git_dir_path);
        setup_mgit(".mgit")?;

        fs::write(file_name, "contenido del archivo")?;
        let path = "tests/add/dir_to_add/non_empty/a.txt";

        index.add_path(file_name)?;
        assert!(index.contains(file_name));

        let result = git_rm(file_name, index_path, git_dir_path);
        assert!(result.is_ok());
        let result1 = Index::load_from_path_if_exists(path, git_dir_path);
        if let Ok(Some(index1)) = result1 {
        } else {
            assert!(result1.is_ok());
        }

        Ok(())
    }

    #[test]
    fn test_remove_directory() -> io::Result<()> {
        let dir_path = "directorio_a_eliminar";
        fs::create_dir_all(dir_path)?;

        let file_path = format!("{}/archivo.txt", dir_path);
        fs::write(&file_path, "contenido del archivo")?;

        remove_directory(dir_path)?;

        assert!(!fs::metadata(dir_path).is_ok());
        assert!(!fs::metadata(&file_path).is_ok());

        Ok(())
    }
}
