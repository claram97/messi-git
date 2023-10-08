use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// `crear_directorio_si_no_existe` es una función de utilidad que crea un directorio si no existe.
///
/// ## Parámetros
///
/// - `directorio`: Ruta al directorio a crear.
///
/// ## Retorna
///
/// Retorna un `io::Result<()>` que indica si la creación del directorio fue exitosa o si se produjo un error.
///
fn crear_directorio_si_no_existe(directorio: &str) -> io::Result<()> {
    fs::create_dir_all(directorio)?;
    Ok(())
}

/// `crear_archivo_si_no_existe` es una función de utilidad que crea un archivo si no existe y escribe el contenido especificado en él.
///
/// ## Parámetros
///
/// - `archivo`: Ruta al archivo a crear.
/// - `contenido`: Contenido a escribir en el archivo.
///
/// ## Retorna
///
/// Retorna un `io::Result<()>` que indica si la creación del archivo y la operación de escritura fueron exitosas o si se produjo un error.
///
fn crear_archivo_si_no_existe(archivo: &str, contenido: &str) -> io::Result<()> {
    if !fs::metadata(archivo).is_ok() {
        let mut file = fs::File::create(archivo)?;
        file.write_all(contenido.as_bytes())?;
    }
    Ok(())
}

/// `git_init` es una función que inicializa un repositorio Git simulado en el directorio especificado.
///
/// ## Parámetros
///
/// - `directorio`: Ruta al directorio donde se inicializará el repositorio.
/// - `initial_branch`: Nombre de la rama inicial.
/// - `template_directory`: Ruta opcional a un directorio de plantilla para copiar archivos desde él.
///
/// ## Retorna
///
/// Retorna un `io::Result<()>` que indica si la operación fue exitosa o si se produjo un error.
///
fn git_init(directorio: &str, initial_branch: &str, template_directory: Option<&str>) -> io::Result<()> {
    // Crear directorio si no existe
    if !Path::new(directorio).exists() {
        fs::create_dir_all(directorio)?;
    }

    // Directorios necesarios
    let git_dir = format!("{}/.git", directorio);
    crear_directorio_si_no_existe(&git_dir)?;

    crear_directorio_si_no_existe(&format!("{}/objects", &git_dir))?;
    crear_directorio_si_no_existe(&format!("{}/refs/heads", &git_dir))?;

    // Archivo HEAD
    let contenido_head = format!("ref: refs/heads/{}\n", initial_branch);
    let head_file = format!("{}/HEAD", &git_dir);
    crear_archivo_si_no_existe(&head_file, &contenido_head)?;

    // Copiar archivos desde el directorio de plantilla 
    if let Some(template) = template_directory {
        let template_dir = Path::new(template);
        let repo_dir = Path::new(directorio);
        for entry in fs::read_dir(template_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let src = entry.path();
            let destino = repo_dir.join(file_name);
            fs::copy(&src, &destino)?;
        }
    }

    println!("Repositorio Git inicializado correctamente en '{}'.", directorio);
    Ok(())
}

// fn main() {
//     let args: Vec<String> = env::args().collect();

//     let mut directorio = ".";
//     let mut initial_branch = "main";
//     let mut template_directory = None;

//     // Procesar argumentos de línea de comandos
//     for (index, arg) in args.iter().enumerate() {
//         match arg.as_str() {
//             "-b" | "-initial-branch" => {
//                 if let Some(branch_name) = args.get(index + 1) {
//                     initial_branch = branch_name;
//                 }
//             }
//             "--template" => {
//                 if let Some(template_dir) = args.get(index + 1) {
//                     template_directory = Some(template_dir.as_str()); // Convertir a &str
//                 }
//             }
//             _ => {
//                 if index == 1 {
//                     directorio = arg;
//                 }
//             }
//         }
//     }

//     if let Err(e) = git_init(directorio, initial_branch, template_directory) {
//         eprintln!("Error al inicializar el repositorio: {}", e);
//     }
// }