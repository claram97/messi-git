use crate::add::add;
use crate::branch;
use crate::branch::git_branch_for_ui;
use crate::check_ignore::git_check_ignore;
use crate::checkout::checkout_branch;
use crate::checkout::checkout_commit_detached;
use crate::checkout::create_and_checkout_branch;
use crate::checkout::create_or_reset_branch;
use crate::checkout::force_checkout;
use crate::commit;
use crate::commit::get_branch_name;
//use crate::fetch::git_fetch_for_gui;
use crate::gui::main_window::add_to_open_windows;
use crate::gui::style::apply_button_style;
use crate::gui::style::configure_repository_window;
use crate::gui::style::create_text_entry_window;
use crate::gui::style::filter_color_code;
use crate::gui::style::get_button;
use crate::gui::style::get_entry;
use crate::gui::style::get_text_view;
use crate::gui::style::load_and_get_window;
use crate::gui::style::show_message_dialog;
use crate::index;
use crate::index::Index;
use crate::log::log;
use crate::log::Log;
use crate::ls_files::git_ls_files;
use crate::merge;
use crate::pull::git_pull;
use crate::push;
use crate::rm::git_rm;
use crate::status;
use crate::tree_handler;
use crate::tree_handler::Tree;
use crate::utils;
use crate::utils::find_git_directory;
use gtk::Button;
use gtk::SwitchExt;
use gtk::prelude::BuilderExtManual;
use gtk::Builder;
use gtk::ButtonExt;
use gtk::DialogExt;
use gtk::EntryExt;
use gtk::GtkWindowExt;
use gtk::LabelExt;
use gtk::TextBufferExt;
use gtk::TextView;
use gtk::TextViewExt;
use gtk::WidgetExt;
use std::env;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::style::apply_entry_style;
use super::style::get_switch;

/// Displays a repository window with various buttons and actions in a GTK application.
///
/// This function initializes and displays a GTK repository window using a UI builder. It configures the window, adds buttons with specific actions, and sets their styles and click event handlers. The repository window provides buttons for actions like "Add," "Commit," "Push," and more.
///
pub fn show_repository_window(code_dir: &Path, working_dir: &Path) -> io::Result<()> {
    let builder: Builder = gtk::Builder::new();
    let complete_path_to_ui = code_dir.join("src/gui/new_window2.ui");
    let complete_path_to_ui_string = match complete_path_to_ui.to_str() {
        Some(string) => string,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to convert path to string.\n",
            ))
        }
    };
    if let Some(new_window) = load_and_get_window(&builder, complete_path_to_ui_string, "window") {
        match env::set_current_dir(working_dir) {
            Ok(_) => println!("Working dir was setted correctly."),
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
        }
        setup_repository_window(&builder, &new_window)?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to show repository window.\n",
        ))
    }
}

/// Setup the repository window with the given GTK builder and window.
/// This function performs various setup tasks, such as configuring buttons and text views.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder.
/// * `new_window` - A reference to a GTK window for the repository.
///
/// # Returns
///
/// Returns an `io::Result` indicating whether the setup was successful or resulted in an error.
///
fn setup_repository_window(builder: &gtk::Builder, new_window: &gtk::Window) -> io::Result<()> {
    let new_window_clone = new_window.clone();
    let builder_clone = builder.clone();
    let builder_clone1 = builder.clone();

    match set_staging_area_texts(&builder_clone) {
        Ok(_) => println!("La función 'set_staging_area_texts' se ejecutó correctamente."),
        Err(err) => println!(
            "Error al llamar a la función 'set_staging_area_texts': {:?}",
            err
        ),
    };
    match set_commit_history_view(&builder_clone1) {
        Ok(_) => println!("La función 'set_commit_history_view' se ejecutó correctamente."),
        Err(err) => println!(
            "Error al llamar a la función 'set_commit_history_view': {:?}",
            err
        ),
    };

    add_to_open_windows(&new_window_clone);
    configure_repository_window(new_window_clone)?;

    let builder_clone_for_merge = builder.clone();
    merge_window(&builder_clone_for_merge)?;

    let builder_clone_for_ls_files = builder.clone();
    list_files_window(&builder_clone_for_ls_files)?;

    let builder_clone_for_check_ignore = builder.clone();
    check_ignore_window(&builder_clone_for_check_ignore);

    setup_buttons(builder)?;

    Ok(())
}

/// Setup buttons in the repository window using the given GTK builder.
/// This function sets up various buttons based on their IDs and connects click events to corresponding actions.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder.
///
/// # Returns
///
/// Returns an `io::Result` indicating whether the button setup was successful or resulted in an error.
///
fn setup_buttons(builder: &gtk::Builder) -> io::Result<()> {
    let button_ids = [
        "show-log-button",
        "pull",
        "push",
        "show-branches-button",
        "delete-branch-button",
        "modify-branch-button",
        "add-path-button",
        "add-all-button",
        "remove-path-button",
        "commit-changes-button",
        "new-branch-button",
        "checkout1",
        "checkout2",
        "checkout3",
        "checkout4",
        "checkout5",
        "show-fetch",
    ];

    for button_id in button_ids.iter() {
        setup_button(builder, button_id)?;
    }

    Ok(())
}

/// Handles the Git pull operation in the current working directory.
///
/// # Errors
///
/// This function may return an error in the following cases:
/// - If it fails to determine the current directory.
/// - If it can't find the Git directory (".mgit").
/// - If it can't find the working directory based on the Git directory.
/// - If it fails to determine the current branch name.
/// - If there is an error during the Git pull operation.
///
fn handle_git_pull() -> io::Result<()> {
    let mut current_dir = std::env::current_dir()?;

    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Can't find git dir.\n",
            ));
        }
    };

    let working_dir = match Path::new(&git_dir).parent() {
        Some(parent) => parent.to_string_lossy().to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Can't find working dir.\n",
            ));
        }
    };

    let current_branch = get_branch_name(&git_dir)?;

    git_pull(&current_branch, &working_dir, None, "localhost")?;

    Ok(())
}

/// Handles the Git push operation in the current working directory.
///
/// # Errors
///
/// This function may return an error in the following cases:
/// - If it fails to determine the current directory.
/// - If it can't find the Git directory (".mgit").
/// - If it fails to determine the current branch name.
/// - If there is an error during the Git push operation.
///
fn handle_git_push() -> io::Result<()> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Can't find git dir.\n",
            ));
        }
    };
    let branch_name = get_branch_name(&git_dir)?;
    push::git_push(&branch_name, &git_dir)
}

/// Setup a button with the specified `button_id` using the given GTK builder. This function applies the
/// button's style, connects the click event to the corresponding action, and sets up various buttons based
/// on their IDs.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder.
/// * `button_id` - A string representing the button's ID.
///
/// # Returns
///
/// Returns an `io::Result` indicating whether the button setup was successful or resulted in an error.
///
fn setup_button(builder: &gtk::Builder, button_id: &str) -> io::Result<()> {
    let button = get_button(builder, button_id);
    apply_button_style(&button).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let builder_clone = builder.clone(); // Clonar el builder
    let button: gtk::Button = builder_clone
        .get_object(button_id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get the button object"))?;
    let merge_text_view = match get_text_view(&builder_clone, "merge-text-view") {
        Some(text_view) => text_view,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Couldn't find merge text view.",
            ));
        }
    };
    match button_id {
        "show-fetch" => {
            button.connect_clicked(move |_| {
                handle_fetch_button(&builder_clone);
            });
        }
        "show-log-button" => {
            button.connect_clicked(move |_| {
                handle_show_log_button_click(&builder_clone);
            });
        }
        "checkout1" => {
            button.connect_clicked(move |_| {
                let result = handle_checkout_branch_window();
                if result.is_ok() {
                    let result = show_current_branch_on_merge_window(&merge_text_view);
                    if result.is_err() {
                        eprintln!("No se pudo actualizar la rama actual en la ventana merge.");
                    }
                } else {
                    eprintln!("Error handling checkout branch window.")
                }
            });
        }
        "checkout2" => {
            button.connect_clicked(move |_| {
                let result = handle_create_and_checkout_branch_button();
                if result.is_ok() {
                    let result = show_current_branch_on_merge_window(&merge_text_view);
                    if result.is_err() {
                        eprintln!("No se pudo actualizar la rama actual en la ventana merge.");
                    }
                } else {
                    eprintln!("Error handling create and checkout branch button.");
                }
            });
        }
        "checkout3" => {
            button.connect_clicked(move |_| {
                let result = handle_create_or_reset_branch_button();
                if result.is_ok() {
                    let result = show_current_branch_on_merge_window(&merge_text_view);
                    if result.is_err() {
                        eprintln!("No se pudo actualizar la rama actual en la ventana merge.");
                    }
                } else {
                    eprintln!("Error handling create or reset branch button.");
                }
            });
        }
        "checkout4" => {
            button.connect_clicked(move |_| {
                let result = handle_checkout_commit_detached_button();
                if result.is_ok() {
                    let result = show_current_branch_on_merge_window(&merge_text_view);
                    if result.is_err() {
                        eprintln!("No se pudo actualizar la rama actual en la ventana merge.");
                    }
                } else {
                    eprintln!("Error handling checkout commit detached button.");
                }
            });
        }
        "checkout5" => {
            button.connect_clicked(move |_| {
                let result = handle_force_checkout_button();
                if result.is_ok() {
                    let result = show_current_branch_on_merge_window(&merge_text_view);
                    if result.is_err() {
                        eprintln!("No se pudo actualizar la rama actual en la ventana merge.");
                    }
                } else {
                    eprintln!("Error handling force checkout button.");
                }
            });
        }
        "pull" => {
            button.connect_clicked(move |_| {
                let result = handle_git_pull();
                match result {
                    Ok(_) => {
                        show_message_dialog("Éxito", "Succesfully pulled");
                    }
                    Err(err) => {
                        show_message_dialog("Error", &err.to_string());
                    }
                }
            });
        }
        "push" => {
            button.connect_clicked(move |_| {
                let result = handle_git_push();
                match result {
                    Ok(_) => {
                        show_message_dialog("Éxito", "Succesfully pushed");
                    }
                    Err(err) => {
                        show_message_dialog("Error", &err.to_string());
                    }
                }
            });
        }
        "show-branches-button" => {
            button.connect_clicked(move |_| {
                handle_show_branches_button(&builder_clone);
            });
        }
        "new-branch-button" => {
            button.connect_clicked(move |_| {
                let result = handle_create_branch_button();
                if result.is_err() {
                    eprintln!("Error handling create branch button.")
                }
            });
        }
        "delete-branch-button" => {
            button.connect_clicked(move |_| {
                let result = handle_delete_branch_button();
                if result.is_err() {
                    eprintln!("Error handling create branch button.")
                }
            });
        }
        "modify-branch-button" => {
            button.connect_clicked(move |_| {
                let result = handle_modify_branch_button();
                if result.is_err() {
                    eprintln!("Error handling create branch button.")
                }
            });
        }

        "add-path-button" => {
            button.connect_clicked(move |_| {
                let result = handle_add_path_button(&builder_clone);
                if result.is_err() {
                    eprintln!("Error handling add path button.")
                }
            });
        }
        "add-all-button" => {
            button.connect_clicked(move |_| {
                let result = handle_add_all_button(&builder_clone);
                if result.is_err() {
                    eprintln!("Error handling add path button.")
                }
            });
        }
        "remove-path-button" => {
            button.connect_clicked(move |_| {
                let result = handle_remove_path_window(&builder_clone);
                if result.is_err() {
                    eprintln!("Error handling remove path button.")
                }
            });
        }
        "commit-changes-button" => {
            button.connect_clicked(move |_| {
                let result = make_commit(&builder_clone);
                if result.is_err() {
                    eprintln!("Error in commit.");
                }
            });
        }
        _ => {}
    }
    Ok(())
}

pub fn obtain_text_from_fetch() -> Result<String, std::io::Error> {
    // let current_dir = match std::env::current_dir() {
    //     Ok(dir) => dir,
    //     Err(err) => {
    //         eprintln!("Error al obtener el directorio actual: {:?}", err);

    //     }
    // };
    // let url_text = &_args[2];//aca hay q poner la url
    // //The remote repo url is the first part of the URL, up until the last '/'.
    // let _remote_repo_url = match url_text.rsplit_once('/') {
    //     Some((string, _)) => string,
    //     None => "",
    // };

    // //The remote repository name is the last part of the URL.
    // let remote_repo_name = url_text.split('/').last().unwrap_or("");
    // let result = git_fetch_for_gui(
    //     Some(remote_repo_name),
    //     "localhost",
    //     current_dir.to_str().expect("Error "),
    // );
    // let refs_text: String = result.join("\n");

    // Ok(refs_text)
    Ok("hola".to_string())
}

fn handle_fetch_button(builder: &gtk::Builder) {
    let log_text_view_result: Option<gtk::TextView> = builder.get_object("fetch-text");

    if let Some(log_text_view) = log_text_view_result {
        let text_from_function = obtain_text_from_fetch();

        match text_from_function {
            Ok(texto) => {
                log_text_view.set_hexpand(true);
                log_text_view.set_halign(gtk::Align::Start);

                if let Some(buffer) = log_text_view.get_buffer() {
                    buffer.set_text(texto.as_str());
                } else {
                    eprintln!("Fatal error in show repository window.");
                }
            }
            Err(err) => {
                eprintln!("Error al obtener el texto: {}", err);
            }
        }
    } else {
        eprintln!("We couldn't find log text view 'log-text'");
    }
}

/// Handle the create and checkout branch button's click event. This function prompts the user to enter a path
/// and attempts to create and checkout a new branch based on the provided path. It shows a success message
/// dialog if the operation is successful, and an error message dialog if the branch doesn't exist.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_create_and_checkout_branch_button() -> io::Result<()> {
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        let resultado = obtain_text_from_create_and_checkout_branch(&text);
        match resultado {
            Ok(texto) => {
                show_message_dialog("Éxito", &format!("Changed correctly to branch '{}'", texto));
            }
            Err(_err) => {
                show_message_dialog("Error", "La rama indicada no existe.");
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    result
}

/// Handle the create or reset branch button's click event. This function prompts the user to enter a path
/// and attempts to create or reset a branch based on the provided path. It shows a success message
/// dialog if the operation is successful, and an error message dialog if the branch doesn't exist.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_create_or_reset_branch_button() -> io::Result<()> {
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        let resultado = obtain_text_from_create_or_reset_branch(&text);
        match resultado {
            Ok(texto) => {
                show_message_dialog("Éxito", &format!("Changed correctly to branch '{}'", texto));
            }
            Err(_err) => {
                show_message_dialog("Error", "La rama indicada no existe.");
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    result
}

/// Handle the checkout commit detached button's click event. This function prompts the user to enter a path
/// and attempts to check out a commit detached from the provided path. It shows a success message
/// dialog if the operation is successful, and an error message dialog if the branch doesn't exist.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_checkout_commit_detached_button() -> io::Result<()> {
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        let resultado = obtain_text_from_checkout_commit_detached(&text);
        match resultado {
            Ok(texto) => {
                show_message_dialog("Éxito", &format!("Changed correctly to branch '{}'", texto));
            }
            Err(_err) => {
                show_message_dialog("Error", "La rama indicada no existe.");
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    result
}

/// Handle the force checkout button's click event. This function prompts the user to enter a path
/// and attempts to perform a force checkout operation on the provided path. It shows a success message
/// dialog if the operation is successful, and an error message dialog if the branch doesn't exist.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_force_checkout_button() -> io::Result<()> {
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        let resultado = obtain_text_from_force_checkout(&text);
        match resultado {
            Ok(texto) => {
                show_message_dialog("Éxito", &format!("Changed correctly to branch '{}'", texto));
            }
            Err(_err) => {
                show_message_dialog("Error", "La rama indicada no existe.");
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    result
}

/// Handle the "Show Branches" button's click event. This function retrieves information about Git branches
/// and displays them in a text view within the GUI. If the operation is successful, it updates the text view
/// with the branch information. If there is an error, it prints an error message to the standard error.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_show_branches_button(builder: &gtk::Builder) {
    let branch_text_view: gtk::TextView = builder.get_object("show-branches-text").unwrap();

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
}

/// Handle the "Create Branch" button's click event. This function opens a text entry window for users to enter
/// the name of the branch they want to create. Once the branch name is entered and confirmed, it attempts to create
/// the new branch and updates the repository window. If the operation is successful, it closes all windows.
/// If there is an error, it prints an error message to the standard error.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
/// # Errors
///
/// This function returns an `io::Result` where `Ok(())` indicates success, and `Err` contains an error description.
///
fn handle_create_branch_button() -> io::Result<()> {
    let create_result = create_text_entry_window("Enter the name of the branch", |text| {
        let result = git_branch_for_ui(Some(text));
        if result.is_err() {
            eprintln!("Error creating text entry window.");
        }
    });

    if create_result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

fn handle_delete_branch_button() -> io::Result<()> {
    let create_result = create_text_entry_window("Enter the name of the branch", |text| {
        let result = git_branch_for_ui(Some(text)); // te dejo pa q le metas la llamdad
        if result.is_err() {
            eprintln!("Error creating text entry window.");
        }
    });

    if create_result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}
fn handle_modify_branch_button() -> io::Result<()> {
    let create_result = create_text_entry_window("Enter the name of the branch", |text| {
        let result = git_branch_for_ui(Some(text)); // aca te dejo pa q le metas la llamada
        if result.is_err() {
            eprintln!("Error creating text entry window.");
        }
    });

    if create_result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

/// Handle the "Add Path" button's click event. This function opens a text entry window for users to enter the path of
/// the file they want to add to the staging area. Once the path is entered and confirmed, it attempts to add the file
/// and displays a success message or an error message if there was an issue.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
/// # Errors
///
/// This function returns an `io::Result` where `Ok(())` indicates success, and `Err` contains an error description.
///
fn handle_add_path_button(builder: &Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let create_result = create_text_entry_window("Enter the path of the file", move |text| {
        match obtain_text_from_add(&text) {
            Ok(_texto) => {
                //show_message_dialog("Operación exitosa", "Agregado correctamente");
                let result = set_staging_area_texts(&builder_clone);
                if result.is_err() {
                    eprintln!("No se pudo actualizar la vista de staging.");
                }
            }
            Err(_err) => {
                show_message_dialog("Error", "El path ingresado no es correcto.");
            }
        }
    });

    if create_result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

/// Handles the action when the "Add All" button is clicked in the user interface.
///
/// # Arguments
///
/// * `builder` - A reference to the GUI builder used to interact with the user interface.
///
/// # Errors
///
/// This function may return an error in the following cases:
/// - If it fails to determine the Git directory or the Git ignore path.
/// - If there is an error during the Git add operation.
/// - If there is an error updating the staging area view in the user interface.
///
fn handle_add_all_button(builder: &Builder) -> io::Result<()> {
    let builder_clone = builder.clone();

    let (git_dir, git_ignore_path) = find_git_directory_and_ignore()?;
    let index_path = format!("{}/index", git_dir);
    match add(
        "None",
        &index_path,
        &git_dir,
        &git_ignore_path,
        Some(vec![".".to_string()]),
    ) {
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
    let result = set_staging_area_texts(&builder_clone);
    if result.is_err() {
        eprintln!("No se pudo actualizar la vista de staging.");
    }

    Ok(())
}

/// Handle the "Remove Path" button's click event. This function opens a text entry window for users to enter
/// the path of the file they want to remove. Once the path is entered and confirmed, it attempts to remove the file
/// and prints the result. If the operation is successful, it prints the removed file's path. If there is an error,
/// it prints an error message to the standard error.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
/// # Errors
///
/// This function returns an `io::Result` where `Ok(())` indicates success, and `Err` contains an error description.
///
fn handle_remove_path_window(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        let resultado = obtain_text_from_remove(&text);
        match resultado {
            Ok(_texto) => {
                let result = set_staging_area_texts(&builder_clone);
                if result.is_err() {
                    eprintln!("No se pudo actualizar la vista de staging.");
                }
                //                println!("Texto: {}", texto);
            }
            Err(_err) => {
                show_message_dialog("Error", "El path ingresado no es correcto.");
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Handle the "Checkout Branch" button's click event. This function opens a text entry window for users to enter
/// the name of the branch they want to check out. Once the branch name is entered and confirmed, it attempts to check
/// out the branch and updates the repository window. If the operation is successful, it displays a success message.
/// If there is an error, it displays an error message.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
/// # Errors
///
/// This function returns an `io::Result` where `Ok(())` indicates success, and `Err` contains an error description.
///
fn handle_checkout_branch_window() -> io::Result<()> {
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        let resultado = obtain_text_from_checkout_branch(&text);
        match resultado {
            Ok(texto) => {
                show_message_dialog("Éxito", &format!("Changed correctly to branch '{}'", texto));
            }
            Err(_err) => match _err.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    show_message_dialog("Éxito", "Changed correctly to branch ");
                }
                _ => {
                    show_message_dialog("Error", "La rama indicada no existe.");
                }
            },
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Handle the "Show Log" button's click event. This function retrieves a text view widget from the GTK builder
/// and populates it with the Git log data. If the operation is successful, it displays the log data in the text view.
/// If there is an error, it prints an error message to the standard error.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_show_log_button_click(builder: &gtk::Builder) {
    let log_text_view_result: Option<gtk::TextView> = builder.get_object("log-text");

    if let Some(log_text_view) = log_text_view_result {
        let text_from_function = obtain_text_from_log();

        match text_from_function {
            Ok(texto) => {
                log_text_view.set_hexpand(true);
                log_text_view.set_halign(gtk::Align::Start);

                if let Some(buffer) = log_text_view.get_buffer() {
                    buffer.set_text(texto.as_str());
                } else {
                    eprintln!("Fatal error in show repository window.");
                }
            }
            Err(err) => {
                eprintln!("Error al obtener el texto: {}", err);
            }
        }
    } else {
        eprintln!("We couldn't find log text view 'log-text'");
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
    let (git_dir, git_ignore_path) = find_git_directory_and_ignore()?;

    stage_changes(&git_dir, &git_ignore_path, texto)
}

/// Find the Git directory and Git ignore file path.
///
/// Searches for the Git directory and Git ignore file in the given current directory.
/// Returns a tuple containing the Git directory path and Git ignore file path if found.
pub fn find_git_directory_and_ignore() -> Result<(String, String), io::Error> {
    let current_dir = std::env::current_dir()?;
    let mut current_dir_buf = current_dir.to_path_buf();
    let git_dir = find_git_directory(&mut current_dir_buf, ".mgit")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Git directory not found"))?;

    let git_dir_parent = current_dir
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found"))?;

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
fn stage_changes(git_dir: &str, git_ignore_path: &str, texto: &str) -> Result<String, io::Error> {
    let index_path = format!("{}/index", git_dir);
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
    let git_dir = find_git_directory(&mut current_dir, ".mgit")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Git directory not found\n"))?;
    let index_path = format!("{}/{}", git_dir, "index");
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_ignore_path = format!("{}/{}", git_dir_parent.to_string_lossy(), ".mgitignore");

    git_rm(texto, &index_path, &git_dir, &git_ignore_path)?;

    Ok("La función 'rm' se ejecutó correctamente.".to_string())
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
    if let Err(err) = force_checkout(&git_dir, texto) {
        eprintln!(
            "Error al forzar el cambio de rama o commit (descartando cambios sin confirmar): {:?}",
            err
        );
    }
    //force_checkout(&git_dir, texto);

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
    let git_dir = find_git_directory(&mut current_dir, ".mgit")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Git directory not found\n"))?;
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_dir_path = Path::new(&git_dir);
    let result = match checkout_commit_detached(
        git_dir_path,
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
    let git_dir = find_git_directory(&mut current_dir, ".mgit")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Git directory not found\n"))?;
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_dir_path = Path::new(&git_dir);
    let result = match create_or_reset_branch(
        git_dir_path,
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
    let git_dir = find_git_directory(&mut current_dir, ".mgit")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Git directory not found\n"))?;
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_dir_path = Path::new(&git_dir);

    let result = match create_and_checkout_branch(
        git_dir_path,
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
    let git_dir = find_git_directory(&mut current_dir, ".mgit")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Git directory not found\n"))?;
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_dir_path = Path::new(&git_dir);

    let _result = match checkout_branch(
        git_dir_path,
        git_dir_parent.to_string_lossy().as_ref(),
        text,
    ) {
        Ok(_) => Ok("The 'checkout branch' function executed successfully.".to_string()),
        Err(err) => {
            {
                match err.kind() {
                    std::io::ErrorKind::UnexpectedEof => {
                        eprintln!("exito.");
                    }
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "Error calling the 'checkout branch' function\n",
                        ));
                    }
                };
            };
            Err(())
        }
    };
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
pub fn call_git_merge(their_branch: &str) -> io::Result<Vec<String>> {
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
    let our_branch = commit::get_branch_name(&git_dir)?;
    let (_, conflicts) = merge::git_merge(
        &our_branch,
        their_branch,
        &git_dir,
        root_dir.to_string_lossy().as_ref(),
    )?;
    Ok(conflicts)
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
                Ok(conflicts) => {
                    match text_view_clone.get_buffer() {
                        Some(buff) => {
                            if conflicts.is_empty() {
                                buff.set_text("Merged successfully!");
                            } else {
                                let text = "Conflicts on merge!\n".to_string()
                                    + &conflicts.join("\n")
                                    + "\nPlease resolve the conflicts and commit the changes.";
                                buff.set_text(&text);
                            }
                        }
                        None => {
                            eprintln!("Couldn't write the output on the text view.");
                        }
                    };
                }
                Err(_e) => {
                    show_message_dialog("Error", "Merge interrupted due to an error.");
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

fn show_current_branch_on_merge_window(merge_text_view: &TextView) -> io::Result<()> {
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

    let buffer = match merge_text_view.get_buffer() {
        Some(buff) => buff,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Text view buffer can't be accessed.\n",
            ));
        }
    };

    let current_branch = commit::get_branch_name(&git_dir)?;
    buffer.set_text(
        &("La rama actual es: ".to_string()
            + &current_branch
            + ".\nIngrese la rama que quiere mergear con la rama actual.\n"),
    );

    Ok(())
}

pub fn list_modified_button_on_clicked(button : &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let mut current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("No se pudo obtener el directorio actual");
                show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

                return;
            }
        };
    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            eprintln!("No se pudo obtener el git dir.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;}
    };
    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            eprintln!("No se pudo obtener el working dir.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;
        }
    };
    let current_dir = &current_dir.to_string_lossy().to_string();
    let line = vec!["git".to_string(), "ls-files".to_string(), "-m".to_string()];
    let index_path = format!("{}/{}",git_dir,"index");
    let gitignore_path = format!("{}/{}",git_dir,".mgitignore");
    let index = match Index::load(&index_path, &git_dir, &gitignore_path) {
        Ok(index) => index,
        Err(_e) => {
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;
        }
    };
    let mut output : Vec<u8> = vec![];
    let result = git_ls_files(&working_dir, &git_dir, current_dir, line, &index, &mut output);
    if result.is_err() {
        show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
        return;
    }
    let buffer = match cloned_text_view.get_buffer() {
        Some(buf) => buf,
        None => {
            eprintln!("No se pudo obtener el text buffer");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
            return;
        }
    };

    let string = match String::from_utf8(output) {
        Ok(str) => str,
        Err(_e) => {
            eprintln!("No se pudo convertir el resultado a string.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
            return;
        }
    };
        buffer.set_text(string.as_str());
    });

}

pub fn list_index_button_on_clicked(button : &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let mut current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("No se pudo obtener el directorio actual");
                show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

                return;
            }
        };
    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            eprintln!("No se pudo obtener el git dir.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;}
    };
    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            eprintln!("No se pudo obtener el working dir.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;
        }
    };
    let current_dir = &current_dir.to_string_lossy().to_string();
    let line = vec!["git".to_string(), "ls-files".to_string()];
    let index_path = format!("{}/{}",git_dir,"index");
    let gitignore_path = format!("{}/{}",git_dir,".mgitignore");
    let index = match Index::load(&index_path, &git_dir, &gitignore_path) {
        Ok(index) => index,
        Err(_e) => {
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;
        }
    };
    let mut output : Vec<u8> = vec![];
    let result = git_ls_files(&working_dir, &git_dir, current_dir, line, &index, &mut output);
    if result.is_err() {
        show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
        return;
    }
    let buffer = match cloned_text_view.get_buffer() {
        Some(buf) => buf,
        None => {
            eprintln!("No se pudo obtener el text buffer");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
            return;
        }
    };

    let string = match String::from_utf8(output) {
        Ok(str) => str,
        Err(_e) => {
            eprintln!("No se pudo convertir el resultado a string.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
            return;
        }
    };
        buffer.set_text(string.as_str());
    });

}

pub fn list_untracked_button_on_clicked(button : &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let mut current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("No se pudo obtener el directorio actual");
                show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

                return;
            }
        };
    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            eprintln!("No se pudo obtener el git dir.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;}
    };
    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            eprintln!("No se pudo obtener el working dir.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;
        }
    };
    let current_dir = &current_dir.to_string_lossy().to_string();
    let line = vec!["git".to_string(), "ls-files".to_string(), "-o".to_string()];
    let index_path = format!("{}/{}",git_dir,"index");
    let gitignore_path = format!("{}/{}",git_dir,".mgitignore");
    let index = match Index::load(&index_path, &git_dir, &gitignore_path) {
        Ok(index) => index,
        Err(_e) => {
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;
        }
    };
    let mut output : Vec<u8> = vec![];
    let result = git_ls_files(&working_dir, &git_dir, current_dir, line, &index, &mut output);
    if result.is_err() {
        show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
        return;
    }
    let buffer = match cloned_text_view.get_buffer() {
        Some(buf) => buf,
        None => {
            eprintln!("No se pudo obtener el text buffer");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
            return;
        }
    };

    let string = match String::from_utf8(output) {
        Ok(str) => str,
        Err(_e) => {
            eprintln!("No se pudo convertir el resultado a string.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
            return;
        }
    };
        buffer.set_text(string.as_str());
    });

}

pub fn list_files_window(builder: &Builder) -> io::Result<()> {
    let list_untracked_button = get_button(builder, "list-untracked-button");
    let list_index_button = get_button(builder, "list-index-button");
    let list_modified_button = get_button(builder, "list-modified-button");
    let text_view = match get_text_view(builder, "ls-files-view") {
        Some(text_view) => text_view,
        None => {
            eprintln!("Error!");
            return Ok(());
        }
    };
    apply_button_style(&list_untracked_button).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    apply_button_style(&list_index_button).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    apply_button_style(&list_modified_button).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    list_untracked_button_on_clicked(&list_untracked_button, &text_view);
    list_index_button_on_clicked(&list_index_button, &text_view);
    list_modified_button_on_clicked(&list_modified_button, &text_view);
    Ok(())
}

pub fn check_ignore_button_on_clicked(button : &Button, text_view: &gtk::TextView, entry : &gtk::Entry, switch : &gtk::Switch) {
    let cloned_text_view = text_view.clone();
    let cloned_entry = entry.clone();
    let cloned_siwtch = switch.clone();
    button.connect_clicked(move |_| {
        let mut current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("No se pudo obtener el directorio actual");
                show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

                return;
            }
        };
    let git_dir = match find_git_directory(&mut current_dir, ".mgit") {
        Some(dir) => dir,
        None => {
            eprintln!("No se pudo obtener el git dir.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;}
    };
    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            eprintln!("No se pudo obtener el working dir.");
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;
        }
    };
    let current_dir = &current_dir.to_string_lossy().to_string();
    let index_path = format!("{}/{}",git_dir,"index");
    let gitignore_path = format!("{}/{}",git_dir,".mgitignore");
    let index = match Index::load(&index_path, &git_dir, &gitignore_path) {
        Ok(index) => index,
        Err(_e) => {
            show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");

            return;
        }
    };

    let path = cloned_entry.get_text();
    if path.is_empty() {
        show_message_dialog("Error", "Debe ingresar un path");
    }
    else {
        let mut line : Vec<String> = vec![];
        if cloned_siwtch.get_active() {
            line = vec!["git".to_string(), "check-ignore".to_string(), "-v".to_string(), path.to_string()];
        }
        else {
            line = vec!["git".to_string(), "check-ignore".to_string(), path.to_string()];
        }
        let mut output : Vec<u8> = vec![];

        match git_check_ignore(".mgitignore", &gitignore_path, line, &mut output) {
            Ok(_) => {
                let buffer = match cloned_text_view.get_buffer() {
                    Some(buf) => buf,
                    None => {
                        eprintln!("No se pudo obtener el text buffer");
                        show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
                        return;
                    }
                };
        
                let string = match String::from_utf8(output) {
                    Ok(str) => str,
                    Err(_e) => {
                        eprintln!("No se pudo convertir el resultado a string.");
                        show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
                        return;
                    }
                };
                    buffer.set_text(string.as_str());
            },
            Err(e) => {
                eprintln!("{}",e); 
                show_message_dialog("Fatal error", "Algo sucedió mientras intentábamos obtener los datos :(");
                return;
                //no sé, personalizar esto jiji
            }
        }
        
}});
    
}

pub fn check_ignore_window(builder : &Builder) {
    let check_ignore_button = get_button(builder, "check-ignore-button");
    let check_ignore_entry = match get_entry(builder, "check-ignore-entry") {
        Some(entry) => entry,
        None => {
            eprintln!("No se pudo obtener el entry.");
            return;
        }
    };
    let check_ignore_view = match get_text_view(builder, "check-ignore-view") {
        Some(view) => view,
        None => {
            eprintln!("No se pudo obtener el text view.");
            return;
        }
    };

    let check_ignore_switch = match get_switch(builder, "check-ignore-switch") {
        Some(view) => view,
        None => {
            eprintln!("No se pudo obtener el switch.");
            return;
        }
    };

    apply_button_style(&check_ignore_button);
    apply_entry_style(&check_ignore_entry);

    check_ignore_button_on_clicked(&check_ignore_button, &check_ignore_view, &check_ignore_entry, &check_ignore_switch);
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
    apply_button_style(&merge_button).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
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

    show_current_branch_on_merge_window(&merge_text_view)?;

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
    match get_not_staged_text() {
        Ok(text) => update_text_view(builder, "not-staged-view", &text)?,
        Err(err) => {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Error getting not staged text: {}", err),
            ))?;
        }
    }
    match get_staged_text() {
        Ok(text) => update_text_view(builder, "staged-view", &text)?,
        Err(err) => {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Error getting staged text: {}", err),
            ))?;
        }
    }
    Ok(())
}

/// Get the text for not staged changes in a Git-like repository.
///
/// This function retrieves the text for changes that are not staged in a Git-like repository.
/// It finds the Git directory, index, and Gitignore file, and then fetches the not staged changes.
///
/// # Returns
///
/// - `Ok(String)`: If the operation is successful, it returns the text for not staged changes.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error`.
fn get_not_staged_text() -> io::Result<String> {
    let current_dir =
        std::env::current_dir().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let current_dir_str = current_dir.to_str().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to convert current directory to string",
    ))?;

    let git_dir = find_git_directory(&mut current_dir.clone(), ".mgit").ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to find git directory",
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

    Ok(not_staged_files + &untracked_string)
}

/// Get the text for staged changes in a Git-like repository.
///
/// This function retrieves the text for changes that are staged in a Git-like repository.
/// It finds the Git directory, index, and Gitignore file, and then fetches the staged changes.
///
/// # Returns
///
/// - `Ok(String)`: If the operation is successful, it returns the text for staged changes.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error`.
fn get_staged_text() -> io::Result<String> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir = find_git_directory(&mut current_dir, ".mgit").ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to find git directory",
    ))?;
    println!("LLEGUE");
    let last_commit = match branch::get_current_branch_commit(&git_dir) {
        Ok(commit) => commit,
        Err(_) => "0000000000000000000000000000000000000000".to_string(),
    };
    let last_commit_tree: Option<Tree> =
        match tree_handler::load_tree_from_commit(&last_commit, &git_dir) {
            Ok(tree) => Some(tree),
            Err(_) => None,
        };
    let index_file = format!("{}{}", git_dir, "/index");
    let gitignore_path = format!("{}{}", current_dir.to_str().unwrap(), "/.gitignore");
    let index = index::Index::load(&index_file, &git_dir, &gitignore_path)?;
    let staged_files = status::get_staged_changes(&index, last_commit_tree)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(staged_files)
}

/// Update a GTK text view with the specified text.
///
/// This function takes a GTK Builder, the name of a text view, and the text to be displayed in the view.
/// It retrieves the text view and its buffer, then sets the provided text in the view.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK Builder.
/// * `view_name` - The name of the text view in the builder.
/// * `text` - The text to set in the view.
///
/// # Returns
///
/// - `Ok(())`: If the text view is successfully updated.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error`.
fn update_text_view(builder: &gtk::Builder, view_name: &str, text: &str) -> io::Result<()> {
    let text_view: gtk::TextView = builder.get_object(view_name).ok_or(io::Error::new(
        io::ErrorKind::Other,
        format!("Failed to get {} object", view_name),
    ))?;

    let buffer = text_view.get_buffer().ok_or(io::Error::new(
        io::ErrorKind::Other,
        format!("Failed to get buffer for {}", view_name),
    ))?;

    buffer.set_text(text);
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

/// Get the current working directory as a string.
fn get_current_dir_string() -> io::Result<String> {
    let current_dir = std::env::current_dir()?;
    current_dir
        .to_str()
        .map(String::from) // Convert the &str to String
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Failed to convert current directory to string",
            )
        })
}

/// Get the Git directory path or return an error if not found.
fn get_git_directory_path(current_dir: &Path) -> io::Result<String> {
    match utils::find_git_directory(&mut current_dir.to_path_buf(), ".mgit") {
        Some(path) => Ok(path),
        None => Err(io::Error::new(
            io::ErrorKind::Other,
            "Git directory not found",
        )),
    }
}

/// Check if the commit message is empty and show an error dialog if it is.
fn check_commit_message(message: &str) -> io::Result<()> {
    if message.is_empty() {
        let dialog = gtk::MessageDialog::new(
            None::<&gtk::Window>,
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Error,
            gtk::ButtonsType::Ok,
            "Debe ingresar un mensaje de commit.",
        );

        dialog.run();
        dialog.close();
        return Ok(());
    }
    Ok(())
}

/// Make a new commit with the provided message.
fn create_new_commit(git_dir_path: &str, message: &str, git_ignore_path: &str) -> io::Result<()> {
    let result = commit::new_commit(git_dir_path, message, git_ignore_path);
    println!("{:?}", result);
    Ok(())
}

/// Perform the commit operation.
fn perform_commit(builder: &gtk::Builder, message: String) -> io::Result<()> {
    let current_dir_str = get_current_dir_string()?;
    let git_dir_path = get_git_directory_path(&PathBuf::from(&current_dir_str))?;
    let git_ignore_path = format!("{}/{}", current_dir_str, ".mgitignore");

    check_commit_message(&message)?;
    create_new_commit(&git_dir_path, &message, &git_ignore_path)?;

    set_commit_history_view(builder)?;
    Ok(())
}

/// Commit changes to a custom Git-like version control system.
fn make_commit(builder: &gtk::Builder) -> io::Result<()> {
    let message_view: gtk::Entry =
        builder
            .get_object("commit-message-text-view")
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to get commit message text view",
                )
            })?;

    let message = message_view.get_text().to_string();

    perform_commit(builder, message)
}
