use crate::gui::gui::add_to_open_windows;
use crate::gui::style::apply_button_style;
use crate::gui::style::get_button;
use crate::gui::run_main_window;
use crate::gui::gui::close_all_windows;
use crate::gui::style::create_text_entry_window;
use gtk::GtkWindowExt;
use gtk::ButtonExt;
use std::io;
use gtk::TextViewExt;
use gtk::TextBufferExt;
use gtk::prelude::BuilderExtManual;
use gtk::WidgetExt;
use crate::gui::style::show_message_dialog;
use crate::branch::git_branch_for_ui;
use crate::gui::style::configure_repository_window;
use crate::gui::style::load_and_get_window;
use crate::add::add;
use std::path::Path;
use crate::utils::find_git_directory;
use crate::rm::git_rm;
use crate::checkout::force_checkout;
use crate::checkout::checkout_commit_detached;
use crate::checkout::create_or_reset_branch;
use crate::checkout::create_and_checkout_branch;
use crate::checkout::checkout_branch;
use crate::log::Log;
use crate::log::log;
use crate::gui::style::filter_color_code;
use std::path::PathBuf;
use gtk::DialogExt;
use gtk::EntryExt;
use gtk::LabelExt;
use crate::index;
use crate::gui::style::get_text_view;
use crate::gui::style::get_entry;
use crate::branch;
use crate::commit;
use crate::merge;
use crate::status;
use crate::utils;
use crate::tree_handler;
use gtk::Builder;


/// Displays a repository window with various buttons and actions in a GTK application.
///
/// This function initializes and displays a GTK repository window using a UI builder. It configures the window, adds buttons with specific actions, and sets their styles and click event handlers. The repository window provides buttons for actions like "Add," "Commit," "Push," and more.
///
pub fn show_repository_window() -> io::Result<()> {
    let builder = gtk::Builder::new();
    if let Some(new_window) = load_and_get_window(&builder, "src/gui/new_window2.ui", "window") {
        let _new_window_clone = new_window.clone();
        let builder_clone = builder.clone();
        let builder_clone1 = builder.clone();

        set_staging_area_texts(&builder_clone)?;
        set_commit_history_view(&builder_clone1)?;

        add_to_open_windows(&new_window);
        configure_repository_window(new_window)?;

        let builder_clone_for_merge = builder.clone();
        merge_window(&builder_clone_for_merge)?;

        let show_log_button = get_button(&builder, "show-log-button");
        let show_pull_button = get_button(&builder, "pull");
        let show_push_button = get_button(&builder, "push");

        let show_branches_button = get_button(&builder, "show-branches-button");

        let add_path_button = get_button(&builder, "add-path-button");
        let add_all_button = get_button(&builder, "add-all-button");

        let remove_path_button = get_button(&builder, "remove-path-button");
        let remove_all_button = get_button(&builder, "remove-all-button");
        let commit_changes_button = get_button(&builder, "commit-changes-button");
        let button8 = get_button(&builder, "button8");
        let button9 = get_button(&builder, "button9");
        let create_branch_button = get_button(&builder, "new-branch-button");
        let button11 = get_button(&builder, "button11");
        let close_repo_button = get_button(&builder, "close");

        let checkout1_button = get_button(&builder, "checkout1");
        let checkout2_button = get_button(&builder, "checkout2");
        let checkout3_button = get_button(&builder, "checkout3");
        let checkout4_button = get_button(&builder, "checkout4");
        let checkout5_button = get_button(&builder, "checkout5");

        apply_button_style(&show_log_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&show_branches_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&add_path_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&add_all_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&remove_path_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&remove_all_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&commit_changes_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&button8).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&button9).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&create_branch_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&button11).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&show_pull_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&show_push_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&close_repo_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&checkout1_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&checkout2_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&checkout3_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&checkout4_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        apply_button_style(&checkout5_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        close_repo_button.connect_clicked(move |_| {
            close_all_windows();
            let result = run_main_window().map_err(|err| io::Error::new(io::ErrorKind::Other, err));
            if result.is_err() {
                eprintln!("Error trying to run main window");
            }
        });

        show_log_button.connect_clicked(move |_| {
            let log_text_view_result = builder_clone.get_object("log-text");
            if log_text_view_result.is_none() {
                eprintln!("We couldn't find log text view 'log-text'");
                return;
            }
            let log_text_view: gtk::TextView = log_text_view_result.unwrap();

            //let label: Label = builder_clone.get_object("show-log-label").unwrap();
            let text_from_function = obtain_text_from_log();
            match text_from_function {
                Ok(texto) => {
                    //let font_description = pango::FontDescription::from_string("Sans 2"); // Cambia "Serif 12" al tamaño y estilo de fuente deseado
                    //log_text_view.override_font(&font_description);
                    log_text_view.set_hexpand(true);
                    log_text_view.set_halign(gtk::Align::Start);

                    // label.set_ellipsize(pango::EllipsizeMode::End);
                    // label.set_text(&texto);
                    //let text_view: TextView = builder.get_object("text_view").unwrap();
                    let buffer_result = log_text_view.get_buffer();
                    if buffer_result.is_none() {
                        eprintln!("Fatal error in show repository window.");
                        return;
                    }
                    let buffer = buffer_result.unwrap();
                    buffer.set_text(texto.as_str());
                }
                Err(err) => {
                    eprintln!("Error al obtener el texto: {}", err);
                }
            }
        });
        checkout1_button.connect_clicked(move |_| {
            let result = create_text_entry_window("Enter the path of the file", move |text| {
                let resultado = obtain_text_from_checkout_branch(&text);
                match resultado {
                    Ok(texto) => {
                        show_message_dialog(
                            "Éxito",
                            &format!("Changed correctly to branch '{}'", texto),
                        );
                    }
                    Err(_err) => {
                        show_message_dialog("Error", "La rama indicada no existe.");
                    }
                }
            });
            if result.is_err() {
                eprintln!("Error creating text entry window.");
            }
        });
        checkout2_button.connect_clicked(move |_| {
            let result = create_text_entry_window("Enter the path of the file", move |text| {
                let resultado = obtain_text_from_create_and_checkout_branch(&text);
                match resultado {
                    Ok(texto) => {
                        show_message_dialog(
                            "Éxito",
                            &format!("Changed correctly to branch '{}'", texto),
                        );
                    }
                    Err(_err) => {
                        show_message_dialog("Error", "La rama indicada no existe.");
                    }
                }
            });
            if result.is_err() {
                eprintln!("Error creating text entry window.");
            }
        });
        checkout3_button.connect_clicked(move |_| {
            let result = create_text_entry_window("Enter the path of the file", move |text| {
                let resultado = obtain_text_from_create_or_reset_branch(&text);
                match resultado {
                    Ok(texto) => {
                        show_message_dialog(
                            "Éxito",
                            &format!("Changed correctly to branch '{}'", texto),
                        );
                    }
                    Err(_err) => {
                        show_message_dialog("Error", "La rama indicada no existe.");
                    }
                }
            });
            if result.is_err() {
                eprintln!("Error creating text entry window.");
            }
        });
        checkout4_button.connect_clicked(move |_| {
            let result = create_text_entry_window("Enter the path of the file", move |text| {
                let resultado = obtain_text_from_checkout_commit_detached(&text);
                match resultado {
                    Ok(texto) => {
                        show_message_dialog(
                            "Éxito",
                            &format!("Changed correctly to branch '{}'", texto),
                        );
                    }
                    Err(_err) => {
                        show_message_dialog("Error", "La rama indicada no existe.");
                    }
                }
            });
            if result.is_err() {
                eprintln!("Error creating text entry window.");
            }
        });
        checkout5_button.connect_clicked(move |_| {
            let result = create_text_entry_window("Enter the path of the file", move |text| {
                let resultado = obtain_text_from_force_checkout(&text);
                match resultado {
                    Ok(texto) => {
                        show_message_dialog(
                            "Éxito",
                            &format!("Changed correctly to branch '{}'", texto),
                        );
                    }
                    Err(_err) => {
                        show_message_dialog("Error", "La rama indicada no existe.");
                    }
                }
            });
            if result.is_err() {
                eprintln!("Error creating text entry window.");
            }
        });
        show_pull_button.connect_clicked(move |_| {
            println!("Pull");
        });
        show_push_button.connect_clicked(move |_| {
            println!("Push");
        });
        show_branches_button.connect_clicked(move |_| {
            let builder_clone = builder.clone();
            let branch_text_view: gtk::TextView =
                builder_clone.get_object("show-branches-text").unwrap();

            let text_from_function = git_branch_for_ui(None);
            match text_from_function {
                Ok(texto) => {
                    let buffer = branch_text_view.get_buffer().unwrap();
                    buffer.set_text(texto.as_str());
                }
                Err(err) => {
                    eprintln!("Error al obtener el texto: {}", err);
                }
            }
        });

        create_branch_button.connect_clicked(move |_| {
            let create_result = create_text_entry_window("Enter the name of the branch", |text| {
                let result = git_branch_for_ui(Some(text));
                if result.is_err() {
                    eprintln!("Error creating text entry window.");
                    return;
                }
                close_all_windows();
                let result = show_repository_window();
                if result.is_err() {
                    eprintln!("Error creating text entry window.");
                }
            });
            if create_result.is_err() {
                eprintln!("Error creating text entry window.");
            }
        });

        add_path_button.connect_clicked(move |_| {
            let create_result =
                create_text_entry_window("Enter the path of the file", move |text| {
                    match obtain_text_from_add(&text) {
                        Ok(_texto) => {
                            show_message_dialog("Operación exitosa", "Agregado correctamente");
                        }
                        Err(_err) => {
                            show_message_dialog("Error", "El path ingresado no es correcto.");
                        }
                    }
                });
            if create_result.is_err() {
                eprintln!("Error creating text entry window.");
            }
        });

        let builder_clone2 = builder_clone1.clone();
        add_all_button.connect_clicked(move |_| {
            let result = obtain_text_from_add(".");
            match result {
                Ok(texto) => {
                    println!("Texto: {}", texto);
                }
                Err(err) => {
                    eprintln!("Error al obtener el texto: {}", err);
                }
            }
            let _ = set_staging_area_texts(&builder_clone1)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        });

        remove_path_button.connect_clicked(move |_| {
            let result = create_text_entry_window("Enter the path of the file", move |text| {
                let resultado = obtain_text_from_remove(&text);
                match resultado {
                    Ok(texto) => {
                        println!("Texto: {}", texto);
                    }
                    Err(err) => {
                        eprintln!("Error al obtener el texto: {}", err);
                    }
                }
            });
            if result.is_err() {
                eprintln!("Error on crating text entry window");
            }
        });

        remove_all_button.connect_clicked(move |_| {
            println!("Button 6 clicked.");
        });

        commit_changes_button.connect_clicked(move |_| {
            let _ = make_commit(&builder_clone2);
        });
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to show repository window.\n",
        ))
    }
}

/// Stage changes for Git commit in a GTK+ application.
///
/// This is the public interface for staging changes for a Git commit. It takes a `texto` parameter
/// to specify the files to stage.
///
/// # Arguments
///
/// * `texto` - A string representing the files to be staged. Use `"."` to stage all changes.
///
/// # Returns
///
/// - `Ok("Ok".to_string())`: If the changes are successfully staged.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error.
pub fn obtain_text_from_add(texto: &str) -> Result<String, io::Error> {
    let current_dir = std::env::current_dir()?;
    let (git_dir, git_ignore_path) = find_git_directory_and_ignore(&current_dir)?;

    stage_changes(&current_dir, &git_dir, &git_ignore_path, texto)
}

/// Find the Git directory and Git ignore file path.
///
/// Searches for the Git directory and Git ignore file in the given current directory.
/// Returns a tuple containing the Git directory path and Git ignore file path if found.
fn find_git_directory_and_ignore(current_dir: &Path) -> Result<(String, String), io::Error> {
    let current_dir = std::env::current_dir()?;
    let mut current_dir_buf = current_dir.to_path_buf();
    let git_dir = find_git_directory(&mut current_dir_buf, ".mgit")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Git directory not found"))?;

    let git_dir_parent = current_dir.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found")
    })?;

    let git_ignore_path = format!("{}/.mgitignore", git_dir_parent.to_string_lossy());

    Ok((git_dir, git_ignore_path))
}

/// Stage changes for Git commit in a GTK+ application.
///
/// This function stages changes for a Git commit by adding specified files or all changes in the
/// working directory. Depending on the provided `texto`, it stages specific files or all changes for the commit.
///
/// # Arguments
///
/// * `current_dir` - The current working directory.
/// * `git_dir` - The Git directory path.
/// * `git_ignore_path` - The Git ignore file path.
/// * `texto` - A string representing the files to be staged. Use `"."` to stage all changes.
///
/// # Returns
///
/// - `Ok("Ok".to_string())`: If the changes are successfully staged.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error`.
fn stage_changes(current_dir: &Path, git_dir: &str, git_ignore_path: &str, texto: &str) -> Result<String, io::Error> {
    let index_path = format!("{}/index", git_dir);

    if texto == "." {
        let options = Some(vec!["-u".to_string()]);
        match add("", &index_path, git_dir, git_ignore_path, options) {
            Ok(_) => {
                println!("La función 'add' se ejecutó correctamente.");
            }
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Error al llamar a la función 'add': {:?}", err),
                ))
            }
        }
    }

    match add(texto, &index_path, git_dir, git_ignore_path, None) {
        Ok(_) => {
            println!("La función 'add' se ejecutó correctamente.");
        }
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Error al llamar a la función 'add': {:?}", err),
            ))
        }
    }

    Ok("Ok".to_string())
}

///
/// This function attempts to remove a file specified by `texto` from a custom version control system similar to Git.
/// It first identifies the Git-like directory (".mgit") in the current directory, and then it calls a function `git_rm`
/// to remove the file. If the removal is successful, it returns a message indicating success. If any errors occur
/// during the process, it returns an `io::Error`.
///
/// # Arguments
///
/// * `texto` - A string representing the file to be removed.
///
/// # Returns
///
/// * `Result<String, io::Error>` - A `Result` containing a success message or an `io::Error` if any issues occur.
///
pub fn obtain_text_from_remove(texto: &str) -> Result<String, io::Error> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir = find_git_directory(&mut current_dir, ".mgit").ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Git directory not found\n")
    })?;
    let index_path = format!("{}/{}", git_dir, "index");
    let git_dir_parent = Path::new(&git_dir).parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n")
    })?;
    let git_ignore_path = format!("{}/{}", git_dir_parent.to_string_lossy(), ".mgitignore");
    
    git_rm(texto, &index_path, &git_dir, &git_ignore_path)?;
    
    Ok(format!("La función 'rm' se ejecutó correctamente."))
}

/// Force checkout a file from a custom Git-like version control system.
///
/// This function attempts to force checkout a file specified by `texto` from a custom version control system similar to Git.
/// It first identifies the Git-like directory (".mgit") in the current directory, and then it calls a function `force_checkout`
/// to force the checkout of the file. If the checkout is successful, it returns a "Ok" message. If any errors occur
/// during the process, it returns an `io::Error`.
///
/// # Arguments
///
/// * `texto` - A string representing the file to be forcefully checked out.
///
/// # Returns
///
/// * `Result<String, io::Error>` - A `Result` containing a success message "Ok" or an `io::Error` if any issues occur.
///
pub fn obtain_text_from_force_checkout(texto: &str) -> Result<String, io::Error> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir: PathBuf = match find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir.into(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ));
        }
    };

    force_checkout(&git_dir, texto);

    Ok("Ok".to_string())
}

/// Checkout a commit in detached HEAD state from a custom Git-like version control system.
///
/// This function attempts to checkout a commit specified by `texto` in a detached HEAD state from a custom version control
/// system similar to Git. It first identifies the Git-like directory (".mgit") in the current directory and its parent
/// directory, and then it calls a function `checkout_commit_detached` to perform the checkout. If the checkout is successful,
/// it returns a message indicating success. If any errors occur during the process, it returns an `io::Error`.
///
/// # Arguments
///
/// * `texto` - A string representing the commit to be checked out in a detached HEAD state.
///
/// # Returns
///
/// * `Result<String, io::Error>` - A `Result` containing a success message or an `io::Error` if any issues occur.
///
pub fn obtain_text_from_checkout_commit_detached(texto: &str) -> Result<String, io::Error> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir: PathBuf = match find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir.into(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ));
        }
    };
    let git_dir_parent: &Path = git_dir
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;

    let result = match checkout_commit_detached(
        &git_dir,
        git_dir_parent.to_string_lossy().as_ref(),
        texto,
    ) {
        Ok(_) => Ok("La función 'checkout branch' se ejecutó correctamente.".to_string()),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Error al llamar a la función 'checkout branch': {:?}\n",
                err
            ),
        )),
    };

    if result.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Error al llamar a la función 'checkout branch'\n",
        ));
    }

    Ok("Ok".to_string())
}

/// Create or reset a branch in a custom Git-like version control system.
///
/// This function attempts to create a new branch or reset an existing branch specified by `texto` in a custom version control
/// system similar to Git. It first identifies the Git-like directory (".mgit") in the current directory and its parent
/// directory, and then it calls a function `create_or_reset_branch` to perform the operation. If the operation is successful,
/// it returns a message indicating success. If any errors occur during the process, it returns an `io::Error`.
///
/// # Arguments
///
/// * `texto` - A string representing the branch name to be created or reset.
///
/// # Returns
///
/// * `Result<String, io::Error>` - A `Result` containing a success message or an `io::Error` if any issues occur.
///
pub fn obtain_text_from_create_or_reset_branch(texto: &str) -> Result<String, io::Error> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir: PathBuf = match find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir.into(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ));
        }
    };
    let git_dir_parent: &Path = git_dir
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;

    let result =
        match create_or_reset_branch(&git_dir, git_dir_parent.to_string_lossy().as_ref(), texto) {
            Ok(_) => Ok("La función 'checkout branch' se ejecutó correctamente.".to_string()),
            Err(err) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Error al llamar a la función 'checkout branch': {:?}\n",
                    err
                ),
            )),
        };
    if result.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Error al llamar a la función 'checkout branch'\n",
        ));
    }
    Ok("Ok".to_string())
}

/// Create and checkout a branch in a custom Git-like version control system.
///
/// This function attempts to create a new branch specified by `texto` and checks it out in a custom version control system
/// similar to Git. It first identifies the Git-like directory (".mgit") in the current directory and its parent directory,
/// and then it calls a function `create_and_checkout_branch` to perform the operation. If the operation is successful, it
/// returns a message indicating success. If any errors occur during the process, it returns an `io::Error`.
///
/// # Arguments
///
/// * `texto` - A string representing the branch name to be created and checked out.
///
/// # Returns
///
/// * `Result<String, io::Error>` - A `Result` containing a success message or an `io::Error` if any issues occur.
///
pub fn obtain_text_from_create_and_checkout_branch(texto: &str) -> Result<String, io::Error> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir: PathBuf = match find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir.into(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ));
        }
    };

    let git_dir_parent: &Path = git_dir
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;

    let result = match create_and_checkout_branch(
        &git_dir,
        git_dir_parent.to_string_lossy().as_ref(),
        texto,
    ) {
        Ok(_) => Ok("La función 'checkout branch' se ejecutó correctamente.".to_string()),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Error al llamar a la función 'checkout branch': {:?}\n",
                err
            ),
        )),
    };
    if result.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Error al llamar a la función 'checkout branch'\n",
        ));
    }
    Ok("Ok".to_string())
}

/// Checkout a branch in a custom Git-like version control system.
///
/// This function attempts to checkout an existing branch specified by `text` in a custom version control system similar to Git.
/// It first identifies the Git-like directory (".mgit") in the current directory and its parent directory, and then it calls a
/// function `checkout_branch` to perform the checkout operation. If the operation is successful, it returns a message indicating
/// success. If any errors occur during the process, it returns an `io::Error`.
///
/// # Arguments
///
/// * `text` - A string representing the name of the branch to be checked out.
///
/// # Returns
///
/// * `Result<String, io::Error>` - A `Result` containing a success message or an `io::Error` if any issues occur.
///
pub fn obtain_text_from_checkout_branch(text: &str) -> Result<String, io::Error> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir: PathBuf = match find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir.into(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ));
        }
    };
    let git_dir_parent: &Path = git_dir
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;

    let result = match checkout_branch(&git_dir, git_dir_parent.to_string_lossy().as_ref(), text) {
        Ok(_) => Ok("The 'checkout branch' function executed successfully.".to_string()),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Error calling the 'checkout branch' function: {:?}\n", err),
        )),
    };
    if result.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Error calling the 'checkout branch' function\n",
        ));
    }
    Ok("Ok".to_string())
}

/// Obtain the Git log as a filtered and formatted string.
///
/// This function obtains the Git log from the Git directory, filters out color codes, and returns
/// it as a formatted string.
///
/// # Returns
///
/// - `Ok(log_text_filtered)`: If the Git log is obtained and processed successfully, it returns
///   the filtered and formatted log as a `String`.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error`.
///
pub fn obtain_text_from_log() -> Result<String, std::io::Error> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(git_dir) => git_dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ))
        }
    };

    let log_iter = log(None, &git_dir, 10, 0, true);
    let log_iter = log_iter?;
    let log_text = get_logs_as_string(log_iter);
    let log_text_filtered = filter_color_code(&log_text);

    Ok(log_text_filtered)
}

/// Convert a log iterator into a formatted log string.
///
/// This function takes an iterator of log entries and converts it into a formatted log string.
///
/// # Arguments
///
/// * `log_iter` - An iterator that yields `Log` entries.
///
/// # Returns
///
/// A formatted log string containing log entries separated by newline characters.
pub fn get_logs_as_string(log_iter: impl Iterator<Item = Log>) -> String {
    let mut log_text = String::new();

    for log in log_iter {
        log_text.push_str(&log.to_string());
        log_text.push('\n');
    }

    log_text
}

/// ## `call_git_merge`
///
/// The `call_git_merge` function initiates a Git merge operation with the specified branch name.
///
/// ### Parameters
/// - `their_branch`: A string containing the name of the branch to merge.
///
/// ### Returns
/// Returns an `io::Result<()>` indicating success or an error.
///
pub fn call_git_merge(their_branch: &str) -> io::Result<()> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Not a git directory.\n",
            ));
        }
    };
    let root_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Parent of git dir not found.\n",
            ));
        }
    };

    let our_branch = branch::get_current_branch_path(&git_dir)?;
    merge::git_merge(
        &our_branch,
        their_branch,
        &git_dir,
        root_dir.to_string_lossy().as_ref(),
    )?;
    Ok(())
}

/// ## `merge_button_connect_clicked`
///
/// The `merge_button_connect_clicked` function connects a GTK button's click event to perform a Git merge operation.
/// It also handles error messages and displays the merge result in a GTK text view.
///
/// ### Parameters
/// - `button`: A reference to the GTK button that triggers the merge operation.
/// - `entry`: A reference to the GTK entry where the user enters the branch name.
/// - `text_view`: A reference to the GTK text view where the merge result is displayed.
/// - `git_directory`: A string containing the path to the Git directory.
///
pub fn merge_button_connect_clicked(
    button: &gtk::Button,
    entry: &gtk::Entry,
    text_view: &gtk::TextView,
    git_directory: String,
) {
    let entry_clone = entry.clone();
    let text_view_clone = text_view.clone();
    let git_dir = git_directory.clone();
    button.connect_clicked(move |_| {
        let branch = entry_clone.get_text();
        if branch.is_empty() {
            show_message_dialog("Error", "Por favor, ingrese una rama.");
        } else if !branch::is_an_existing_branch(&branch, &git_dir) {
            show_message_dialog("Error", "Rama no encontrada.");
        } else {
            match call_git_merge(&branch) {
                Ok(_) => {
                    match text_view_clone.get_buffer() {
                        Some(buff) => {
                            buff.set_text("Merged successfully!");
                        }
                        None => {
                            eprintln!("Couldn't write the output on the text view.");
                        }
                    };
                }
                Err(_e) => {
                    match text_view_clone.get_buffer() {
                        Some(buff) => {
                            buff.set_text("Conflicts on merge!");
                        }
                        None => {
                            eprintln!("Couldn't write the output on the text view.");
                        }
                    };
                }
            };
        }
    });
}

/// ## `set_merge_button_behavior`
///
/// The `set_merge_button_behavior` function sets the behavior for a GTK button to perform a Git merge operation.
/// It is responsible for connecting the button's click event and handling errors.
///
/// ### Parameters
/// - `button`: A reference to the GTK button that triggers the merge operation.
/// - `entry`: A reference to the GTK entry where the user enters the branch name.
/// - `text_view`: A reference to the GTK text view where the merge result is displayed.
///
pub fn set_merge_button_behavior(
    button: &gtk::Button,
    entry: &gtk::Entry,
    text_view: &gtk::TextView,
) -> io::Result<()> {
    //let entry_clone = entry.clone();
    //let text_view_clone = text_view.clone();
    let mut current_dir = std::env::current_dir()?;
    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found.\n",
            ));
        }
    };

    merge_button_connect_clicked(button, entry, text_view, git_dir);

    Ok(())
}

/// ## `merge_window`
///
/// The `merge_window` function initializes the GTK merge window by connecting UI elements to Git merge functionality.
///
/// ### Parameters
/// - `builder`: A reference to the GTK builder for constructing the UI.
///
pub fn merge_window(builder: &Builder) -> io::Result<()> {
    let merge_button = get_button(builder, "merge-button");
    apply_button_style(&merge_button)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let merge_input_branch_entry = match get_entry(builder, "merge-input-branch") {
        Some(merge) => merge,
        None => {
            return Err(io::Error::new(io::ErrorKind::Other, "Entry not found.\n"));
        }
    };
    let merge_text_view = match get_text_view(builder, "merge-text-view") {
        Some(text_view) => text_view,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Text view not found.\n",
            ));
        }
    };

    set_merge_button_behavior(&merge_button, &merge_input_branch_entry, &merge_text_view)?;
    Ok(())
}

/// Sets the text content of staging area views in a GTK+ application.
///
/// This function retrieves GTK+ text views from a provided builder, obtains information about the
/// staging area and the last commit in a Git repository, and sets the text content of the "not-staged"
/// and "staged" views accordingly.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK+ builder containing the text views.
///
/// # Returns
///
/// - `Ok(())`: If the staging area views are successfully updated with text content.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error`.
///
pub fn set_staging_area_texts(builder: &gtk::Builder) -> io::Result<()> {
    let staging_area_text_view: gtk::TextView = builder.get_object("not-staged-view").ok_or(
        io::Error::new(io::ErrorKind::Other, "Failed to get not-staged-view object"),
    )?;
    let buffer = staging_area_text_view.get_buffer().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to get buffer for not-staged-view\n",
    ))?;

    let current_dir =
        std::env::current_dir().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let _binding = current_dir.clone();
    let current_dir_str = current_dir.to_str().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to convert current directory to string\n",
    ))?;

    let git_dir = find_git_directory(&mut current_dir.clone(), ".mgit").ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to find git directory\n",
    ))?;

    let index_file = format!("{}{}", git_dir, "/index");
    let gitignore_path = format!("{}{}", current_dir.to_str().unwrap(), "/.gitignore");
    let index = index::Index::load(&index_file, &git_dir, &gitignore_path)?;
    let not_staged_files = status::get_unstaged_changes(&index, current_dir_str)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut untracked_files_output: Vec<u8> = Vec::new();
    status::find_untracked_files(
        &current_dir,
        &current_dir,
        &index,
        &mut untracked_files_output,
    )?;
    let mut untracked_string = String::from_utf8(untracked_files_output)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    untracked_string = untracked_string.replace("\x1b[31m\t\t", "");
    untracked_string = untracked_string.replace("x1b[0m\n", "\n");
    let not_staged_files = not_staged_files + &untracked_string;

    buffer.set_text(&not_staged_files);

    let staged_area_text_view: gtk::TextView = builder.get_object("staged-view").ok_or(
        io::Error::new(io::ErrorKind::Other, "Failed to get staged-view object"),
    )?;

    let staged_buffer = staged_area_text_view.get_buffer().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to get buffer for staged-view",
    ))?;
    //Get the repos last commit
    let last_commit = branch::get_current_branch_commit(&git_dir)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let last_commit_tree = tree_handler::load_tree_from_commit(&last_commit, &git_dir)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let staged_files = status::get_staged_changes(&index, &last_commit_tree)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    staged_buffer.set_text(&staged_files);
    Ok(())
}

/// Format a list of branch history entries into a single string.
///
/// This function takes a vector of branch history entries, where each entry consists of a commit
/// hash and a commit message. It formats these entries into a single string, with each entry
/// presented as a compact line with the abbreviated commit hash and commit message.
///
/// # Arguments
///
/// * `history_vec` - A vector of tuples, where each tuple contains a commit hash and a commit message.
///
/// # Returns
///
/// A formatted string containing the branch history entries, each presented as a single line
/// with the abbreviated commit hash and commit message.
///
pub fn format_branch_history(history_vec: Vec<(String, String)>) -> String {
    let mut string_result: String = "".to_string();
    for commit in history_vec {
        let hash_abridged = &commit.0[..6];
        let commit_line = hash_abridged.to_string() + "\t" + &commit.1 + "\n";
        string_result.push_str(&commit_line);
    }
    string_result.to_string()
}

/// Set the commit history view in a GTK+ application.
///
/// This function populates the commit history view in the GTK+ application by obtaining the
/// current branch name, retrieving the commit history for the branch, formatting it, and
/// setting it in the view. It also updates a label to display the current branch.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK+ builder containing the UI elements.
///
/// # Returns
///
/// - `Ok(())`: If the commit history view is successfully updated.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error`.
///
pub fn set_commit_history_view(builder: &gtk::Builder) -> io::Result<()> {
    let label_current_branch: gtk::Label = builder
        .get_object("commit-current-branch-commit")
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get label"))?;

    let mut current_dir = std::env::current_dir()?;
    let binding = current_dir.clone();
    let _current_dir_str = binding.to_str().unwrap();
    let git_dir_path_result = utils::find_git_directory(&mut current_dir, ".mgit");

    let git_dir_path = match git_dir_path_result {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Git directory not found\n",
            ))
        }
    };

    let current_branch_name = commit::get_branch_name(&git_dir_path)?;

    let current_branch_text: String = "Current branch: ".to_owned() + &current_branch_name;

    label_current_branch.set_text(&current_branch_text);
    let branch_last_commit = branch::get_current_branch_commit(&git_dir_path)?;
    let branch_commits_history =
        utils::get_branch_commit_history_with_messages(&branch_last_commit, &git_dir_path)?;
    let branch_history_formatted = format_branch_history(branch_commits_history);

    let text_view_history: gtk::TextView = builder
        .get_object("commit-history-view")
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get history view"))?;

    let history_buffer = text_view_history
        .get_buffer()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get history buffer"))?;

    history_buffer.set_text(&branch_history_formatted);
    Ok(())
}

/// Create a new Git commit in a GTK+ application.
///
/// This function allows the user to create a new Git commit by providing a commit message
/// through a GTK+ entry widget. It retrieves the current working directory, Git directory,
/// and commit message, then creates the commit. After the commit is created, it updates
/// the commit history view in the application.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK+ builder containing the UI elements.
///
/// # Returns
///
/// - `Ok(())`: If the commit is successfully created, and the commit history view is updated.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error`.
///
pub fn make_commit(builder: &gtk::Builder) -> io::Result<()> {
    let mut current_dir = std::env::current_dir()?;
    let binding = current_dir.clone();
    let current_dir_str = match binding.to_str() {
        Some(str) => str.to_owned(), // Asignar el valor
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to convert current directory to string\n",
            ))
        }
    };

    let git_dir_path = match utils::find_git_directory(&mut current_dir, ".mgit") {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Git directory not found\n",
            ))
        }
    };

    let git_ignore_path = format!("{}/{}", current_dir_str, ".mgitignore");

    let message_view: gtk::Entry =
        builder
            .get_object("commit-message-text-view")
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to get commit message text view\n",
                )
            })?;

    let message = message_view.get_text().to_string();

    if message.is_empty() {
        // El mensaje de commit está vacío, muestra un diálogo de error
        let dialog = gtk::MessageDialog::new(
            None::<&gtk::Window>,
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Error,
            gtk::ButtonsType::Ok,
            "Debe ingresar un mensaje de commit.\n",
        );

        dialog.run();
        dialog.close();
        return Ok(());
    }

    let result = commit::new_commit(&git_dir_path, &message, &git_ignore_path);
    println!("{:?}", result);
    set_commit_history_view(builder)?;
    Ok(())
}

