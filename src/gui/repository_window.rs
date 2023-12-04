use crate::add;
use crate::add::add;
use crate::branch;
use crate::branch::git_branch;
use crate::branch::is_an_existing_branch;
use crate::check_ignore::git_check_ignore;
use crate::checkout;
use crate::checkout::checkout_branch;
use crate::checkout::checkout_commit_detached;
use crate::checkout::create_and_checkout_branch;
use crate::checkout::create_or_reset_branch;
use crate::checkout::force_checkout;
use crate::commit;
use crate::commit::get_branch_name;
use crate::config::Config;
use crate::configuration::GIT_DIR;
use crate::configuration::GIT_IGNORE;
use crate::configuration::HOST;
use crate::configuration::INDEX;
use crate::fetch::git_fetch;
use crate::git_config::git_config;
use crate::rebase;
use crate::tag::git_tag;
use crate::utils::obtain_git_dir;
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;
use std::str;
//use crate::fetch::git_fetch_for_gui;
use super::style::apply_entry_style;
use super::style::apply_label_style;
use super::style::create_text_entry_window_with_switch;
use super::style::get_combo_box;
use super::style::get_label;
use super::style::get_switch;
use super::visual_branches;
use crate::gui::main_window::add_to_open_windows;
use crate::gui::style::apply_button_style;
use crate::gui::style::configure_repository_window;
use crate::gui::style::create_text_entry_window;
use crate::gui::style::create_text_entry_window2;
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
use crate::ls_tree::ls_tree;
use crate::merge;
use crate::pull::git_pull;
use crate::push;
use crate::remote::git_remote;
use crate::rm::git_rm;
use crate::show_ref::git_show_ref;
use crate::status;
use crate::tree_handler;
use crate::tree_handler::Tree;
use crate::utils;
use crate::utils::find_git_directory;
use gtk::prelude::BuilderExtManual;
use gtk::Builder;
use gtk::Button;
use gtk::ButtonExt;
use gtk::ComboBoxExt;
use gtk::ComboBoxText;
use gtk::ComboBoxTextExt;
use gtk::ContainerExt;
use gtk::DialogExt;
use gtk::Entry;
use gtk::EntryExt;
use gtk::GtkWindowExt;
use gtk::LabelExt;
use gtk::ScrolledWindow;
use gtk::ScrolledWindowExt;
use gtk::SwitchExt;
use gtk::TextBufferExt;
use gtk::TextView;
use gtk::TextViewExt;
use gtk::WidgetExt;
use std::env;
use std::io;
use std::path::Path;
use std::path::PathBuf;

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
        Ok(_) => println!(" 'set_staging_area_texts' works correctly."),
        Err(err) => println!("Error in 'set_staging_area_texts': {:?}", err),
    };
    match set_commit_history_view(&builder_clone1) {
        Ok(_) => println!("La función 'set_commit_history_view' works correctly."),
        Err(err) => println!("Error in 'set_commit_history_view': {:?}", err),
    };

    add_to_open_windows(&new_window_clone);
    configure_repository_window(new_window_clone)?;

    let builder_clone_for_merge = builder.clone();
    merge_window(&builder_clone_for_merge)?;

    let builder_clone_for_ls_files = builder.clone();
    list_files_window(&builder_clone_for_ls_files)?;

    let builder_clone_for_check_ignore = builder.clone();
    check_ignore_window(&builder_clone_for_check_ignore);

    let builder_clone_for_show_ref = builder.clone();
    show_ref_window(&builder_clone_for_show_ref);

    let builder_clone_for_git_config = builder.clone();
    config_window(&builder_clone_for_git_config);

    let builder_clone_for_rebase = builder.clone();
    rebase_window(&builder_clone_for_rebase)?;

    let builder_clone_for_fetch = builder.clone();
    match apply_style_to_fetch(&builder_clone_for_fetch) {
        Ok(_) => {}
        Err(error) => {
            eprintln!("{:?}", error);
        }
    }

    let builder_clone_for_checkout_view = builder.clone();
    match handle_show_branches_button(&builder_clone_for_checkout_view) {
        Ok(_) => {}
        Err(error) => {
            eprintln!("{:?}", error.to_string());
        }
    }

    let builder_clone_for_config_window = builder.clone();
    match update_config_window(&builder_clone_for_config_window) {
        Ok(_) => {}
        Err(error) => {
            eprintln!("{:?}", error.to_string());
        }
    }

    setup_buttons(builder)?;

    Ok(())
}

fn update_config_window(builder: &Builder) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;
    let config = Config::load(&git_dir)?;
    let label = match get_label(builder, "config-title-label", 13.0) {
        Some(label) => label,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se encontró el config label!".to_string(),
            ));
        }
    };
    if let Ok((user, email)) = config.get_user_name_and_email() {
        let text = format!("Welcome {user}!\nIt seems that {email} is stored in\nour database.\nRemember that you can change it if you want :)");
        label.set_text(&text);
    } else {
        label.set_text("Welcome!\nSome functionalities may fail if you don't tell us who are you.\nPlease, leave your name and email below.");
    }
    Ok(())
}

fn apply_style_to_fetch(builder: &Builder) -> io::Result<()> {
    let fetch_entry = match get_entry(builder, "fetch-entry") {
        Some(entry) => entry,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se encontró el fetch entry!".to_string(),
            ));
        }
    };

    apply_entry_style(&fetch_entry);

    let fetch_label = match get_label(builder, "fetch-label", 14.0) {
        Some(entry) => entry,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "No se encontró el fetch label!".to_string(),
            ));
        }
    };

    apply_label_style(&fetch_label);

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
        "visual-branches-button",
        "show-log-button",
        "pull",
        "push",
        "show-branches-button",
        "delete-branch-button",
        "modify-branch-button",
        "another-branch",
        "add-path-button",
        "add-all-button",
        "remove-path-button",
        "reload-staging-view-button",
        "commit-changes-button",
        "new-branch-button",
        "checkout1",
        "checkout2",
        "checkout3",
        "checkout4",
        "checkout5",
        "remote-add",
        "remote-rm",
        "remote-set-url",
        "remote-get-url",
        "remote-rename",
        "remote",
        "list-tags",
        "add-normal-tag",
        "remove-tag",
        "add-annotated-tag",
        "verify-tag",
        "tag-from-tag",
        "trees-button",
        "r-trees",
        "d-trees",
        "rt-trees",
        "fetch",
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

    let git_dir = match find_git_directory(&mut current_dir, GIT_DIR) {
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
    let git_dir = match find_git_directory(&mut current_dir, GIT_DIR) {
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
    let builder_clone = builder.clone();
    let button: gtk::Button = builder_clone
        .get_object(button_id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get the button object"))?;
    match button_id {
        "trees-button" => {
            button.connect_clicked(move |_| match handle_ls_trees(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "r-trees" => {
            button.connect_clicked(move |_| match handle_ls_trees_r(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "d-trees" => {
            button.connect_clicked(move |_| match handle_ls_trees_d(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "rt-trees" => {
            button.connect_clicked(move |_| match handle_ls_trees_rt(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "verify-tag" => {
            button.connect_clicked(move |_| match handle_tag_verify(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "tag-from-tag" => {
            button.connect_clicked(move |_| match handle_tag_from_tag(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "list-tags" => {
            button.connect_clicked(move |_| match handle_list_tags(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "add-normal-tag" => {
            button.connect_clicked(move |_| match handle_tag_add_normal(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "add-annotated-tag" => {
            button.connect_clicked(move |_| match handle_tag_add_annotated(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "remove-tag" => {
            button.connect_clicked(move |_| match handle_tag_remove(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "another-branch" => {
            button.connect_clicked(move |_| {
                match handle_create_branch_from_branch_button(&builder_clone) {
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("{:?}", error);
                    }
                }
            });
        }
        "remote" => {
            button.connect_clicked(move |_| match handle_remote(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "remote-add" => {
            button.connect_clicked(move |_| match handle_remote_add(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "remote-rm" => {
            button.connect_clicked(move |_| match handle_remote_rm(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "remote-set-url" => {
            button.connect_clicked(move |_| match handle_remote_set_url() {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "remote-get-url" => {
            button.connect_clicked(move |_| match handle_remote_get_url() {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "remote-rename" => {
            button.connect_clicked(move |_| match handle_remote_rename(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "fetch" => {
            button.connect_clicked(move |_| match handle_fetch_button(&builder_clone) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            });
        }
        "show-log-button" => {
            button.connect_clicked(move |_| {
                handle_show_log_button_click(&builder_clone);
            });
        }
        "checkout1" => {
            button.connect_clicked(move |_| {
                handle_checkout_branch_window(&builder_clone);
            });
        }
        "checkout2" => {
            button.connect_clicked(move |_| {
                handle_create_and_checkout_branch_button(&builder_clone);
            });
        }
        "checkout3" => {
            button.connect_clicked(move |_| {
                handle_create_or_reset_branch_button(&builder_clone);
            });
        }
        "checkout4" => {
            button.connect_clicked(move |_| {
                handle_checkout_commit_detached_button(&builder_clone);
            });
        }
        "checkout5" => {
            button.connect_clicked(move |_| {
                handle_force_checkout_button(&builder_clone);
            });
        }
        "pull" => {
            button.connect_clicked(move |_| {
                let result = handle_git_pull();
                match result {
                    Ok(_) => {
                        show_message_dialog("Success", "Succesfully pulled");
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
                        show_message_dialog("Success", "Succesfully pushed");
                    }
                    Err(err) => {
                        show_message_dialog("Error", &err.to_string());
                    }
                }
            });
        }
        "show-branches-button" => {
            button.connect_clicked(move |_| match handle_show_branches_button(&builder_clone) {
                Ok(_) => {}
                Err(err) => {
                    show_message_dialog("Error", &err.to_string());
                }
            });
        }
        "new-branch-button" => {
            button.connect_clicked(move |_| {
                let result = handle_create_branch_button(&builder_clone);
                if result.is_err() {
                    eprintln!("Error handling create branch button.")
                }
            });
        }
        "delete-branch-button" => {
            button.connect_clicked(move |_| {
                let result = handle_delete_branch_button(&builder_clone);
                if result.is_err() {
                    eprintln!("Error handling create branch button.")
                }
            });
        }
        "modify-branch-button" => {
            button.connect_clicked(move |_| {
                let result = handle_modify_branch_button(&builder_clone);
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
        "visual-branches-button" => {
            button.connect_clicked(move |_| {
                let result = handle_visual_branches_button(&builder_clone);
                if result.is_err() {
                    eprintln!("Error handling visual branches button.")
                }
            });
        }
        "reload-staging-view-button" => {
            button.connect_clicked(move |_| {
                let result = handle_reload_staging_view_button(&builder_clone);
                if result.is_err() {
                    eprintln!("Error handling reload staging view button.")
                }
            });
        }
        _ => {}
    }
    Ok(())
}

/// Handles the user interaction with the "Fetch" button in the GUI.
///
/// This function is responsible for obtaining user input, validating the input,
/// and initiating a Git fetch operation based on the provided information.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder object that is used to access
///              and manipulate the GUI components.
///
/// # Returns
///
/// Returns `Ok(())` on success and an `io::Error` on failure.
///
/// # Errors
///
/// This function may return an `io::Error` in the following scenarios:
///
/// * The GTK entry field named "fetch-entry" is not found in the provided builder.
/// * The entered remote name is empty.
/// * There is an issue loading the Git directory or configuration.
/// * The entered remote name does not exist in the Git configuration.
///
/// # Panics
///
/// This function does not intentionally panic under normal circumstances.
///
fn handle_fetch_button(builder: &gtk::Builder) -> io::Result<()> {
    let fetch_entry = match get_entry(builder, "fetch-entry") {
        Some(entry) => entry,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "No se encontró el fetch entry!",
            ));
        }
    };

    let remote_name = fetch_entry.get_text().to_string();
    if remote_name.is_empty() {
        show_message_dialog("Error", "Debe ingresar el nombre de un remote");
    } else {
        let git_dir = obtain_git_dir()?;
        let working_dir = match Path::new(&git_dir).parent() {
            Some(dir) => dir.to_string_lossy().to_string(),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "No pudimos obtener el working dir",
                ));
            }
        };
        let config = Config::load(&git_dir)?;
        if !config.is_an_existing_remote(&remote_name) {
            show_message_dialog("Error", "El remoto no existe. Revise los remotos existentes o cree uno nuevo en la pestaña remote.");
        } else {
            match git_fetch(Some(&remote_name), HOST, &working_dir) {
                Ok(_) => {
                    show_message_dialog("Éxito", "Fetched successfully");
                }
                Err(error) => {
                    show_message_dialog("Error", &error.to_string());
                }
            }
        }
    }
    Ok(())
}

/// Handle the create and checkout branch button's click event. This function prompts the user to enter a path
/// and attempts to create and checkout a new branch based on the provided path. It shows a success message
/// dialog if the operation is successful, and an error message dialog if the branch doesn't exist.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_create_and_checkout_branch_button(builder: &Builder) {
    let builder_clone = builder.clone();
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        if text.is_empty() {
            show_message_dialog("Error", "The branch does not exists");
        } else {
            let resultado = obtain_text_from_create_and_checkout_branch(&text);
            match resultado {
                Ok(_) => {
                    update_views_since_checkout_was_called(&builder_clone);
                }
                Err(err) => {
                    let error_message = err.to_string();
                    show_message_dialog("Error", &error_message);
                }
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
}

/// Handle the create or reset branch button's click event. This function prompts the user to enter a path
/// and attempts to create or reset a branch based on the provided path. It shows a success message
/// dialog if the operation is successful, and an error message dialog if the branch doesn't exist.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_create_or_reset_branch_button(builder: &Builder) {
    let builder_clone = builder.clone();
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        if text.is_empty() {
            show_message_dialog("Error", "The branch does not exists.");
        } else {
            let resultado = obtain_text_from_create_or_reset_branch(&text);
            match resultado {
                Ok(_) => {
                    update_views_since_checkout_was_called(&builder_clone);
                }
                Err(err) => {
                    let error_message = err.to_string();
                    show_message_dialog("Error", &error_message);
                }
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
}

/// Handle the checkout commit detached button's click event. This function prompts the user to enter a path
/// and attempts to check out a commit detached from the provided path. It shows a success message
/// dialog if the operation is successful, and an error message dialog if the branch doesn't exist.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_checkout_commit_detached_button(builder: &Builder) {
    let builder_clone = builder.clone();
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        if text.is_empty() {
            show_message_dialog("Error", "The branch does not exists.");
        } else {
            let resultado = obtain_text_from_checkout_commit_detached(&text);
            match resultado {
                Ok(_) => {
                    update_views_since_checkout_was_called(&builder_clone);
                }
                Err(err) => {
                    let error_message = err.to_string();
                    show_message_dialog("Error", &error_message);
                }
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
}

/// Handle the force checkout button's click event. This function prompts the user to enter a path
/// and attempts to perform a force checkout operation on the provided path. It shows a success message
/// dialog if the operation is successful, and an error message dialog if the branch doesn't exist.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
///
fn handle_force_checkout_button(builder: &gtk::Builder) {
    let builder_clone = builder.clone();
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        if text.is_empty() {
            show_message_dialog("Error", "The branch does not exists.");
        } else {
            let resultado = obtain_text_from_force_checkout(&text);
            match resultado {
                Ok(_) => {
                    update_views_since_checkout_was_called(&builder_clone);
                }
                Err(err) => {
                    let error_message = err.to_string();
                    show_message_dialog("Error", &error_message);
                }
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
}

/// Retrieves information about Git branches in the repository.
///
/// # Returns
///
/// Returns a tuple containing the result of the `git_branch` operation and a string
/// representing the output of the operation.
///
/// The tuple consists of:
/// - `io::Result<()>`: Result indicating the success or failure of the `git_branch` operation.
/// - `String`: Output of the `git_branch` operation, representing information about Git branches.
///
/// # Errors
///
/// Returns an `io::Error` if there are issues obtaining the output or if the `git_branch` operation fails.
///
fn show_branches() -> io::Result<(io::Result<()>, String)> {
    let mut output: Vec<u8> = vec![];
    let result = git_branch(None, None, None, &mut output);
    let output_string = match String::from_utf8(output) {
        Ok(string) => string,
        Err(_e) => {
            show_message_dialog("Fatal error", "Something unexpected happened.");
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Couldn't get the output\n",
            ));
        }
    };
    Ok((result, output_string))
}

/// Retrieves a `TextView` and a `ScrolledWindow` from a GTK builder for branch GUI.
///
/// This function is designed to obtain the necessary GTK widgets (TextView and ScrolledWindow)
/// for displaying Git branch information in a graphical user interface (GUI).
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns a tuple containing a `TextView` and a `ScrolledWindow`.
///
/// The tuple consists of:
/// - `TextView`: The GTK TextView widget for displaying text information.
/// - `ScrolledWindow`: The GTK ScrolledWindow widget for providing scrolling functionality.
///
/// # Errors
///
/// Returns an `io::Error` if any of the required GTK widgets are not found in the builder.
///
fn get_text_view_and_scroll_for_branch_gui(
    builder: &gtk::Builder,
) -> io::Result<(TextView, ScrolledWindow)> {
    let branch_text_view: gtk::TextView = match builder.get_object("show-branches-text") {
        Some(text_view) => text_view,
        None => {
            eprintln!("Couldn't get show branches text view");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Couldn't get show branches text view\n",
            ));
        }
    };

    let scrolled_window: gtk::ScrolledWindow = match builder.get_object("scrolled-window") {
        Some(scrolled) => scrolled,
        None => {
            eprintln!("Couldn't get show branches scrolled window");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Couldn't get show branches scrolled window\n",
            ));
        }
    };

    Ok((branch_text_view, scrolled_window))
}

/// Handles the result of the `show_branches` operation and updates the GTK UI components accordingly.
///
/// This function takes the result of the `show_branches` operation, the GTK TextView,
/// the ScrolledWindow, and the output string. It then updates the UI components based on the result.
///
/// # Arguments
///
/// - `result`: Result indicating the success or failure of the `show_branches` operation.
/// - `branch_text_view`: A reference to the GTK TextView widget for displaying text information.
/// - `scrolled_window`: A reference to the GTK ScrolledWindow widget for providing scrolling functionality.
/// - `output_string`: The output string representing information about Git branches.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the GTK components.
///
fn handle_show_branches_result(
    result: io::Result<()>,
    branch_text_view: &gtk::TextView,
    scrolled_window: &ScrolledWindow,
    output_string: String,
) -> io::Result<()> {
    let buffer = match branch_text_view.get_buffer() {
        Some(buf) => buf,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Couldn't get the text view\n",
            ));
        }
    };

    match result {
        Ok(_) => {
            let clean_output = branch::remove_ansi_escape_codes(&output_string);
            buffer.set_text(&clean_output);
            scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
            scrolled_window.add(branch_text_view);
        }
        Err(_e) => {
            show_message_dialog("Error", &_e.to_string());
        }
    }
    Ok(())
}

/// Handles the action triggered by the "Show Branches" button in a GTK application.
///
/// This function orchestrates the process of displaying Git branches in a GTK UI.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components or the Git operation.
///
fn handle_show_branches_button(builder: &gtk::Builder) -> io::Result<()> {
    let (branch_text_view, scrolled_window) = get_text_view_and_scroll_for_branch_gui(builder)?;

    let (result, output_string) = show_branches()?;

    handle_show_branches_result(result, &branch_text_view, &scrolled_window, output_string)?;

    Ok(())
}

/// Creates a new Git branch with the specified name and updates the branch view.
///
/// This function utilizes the `git_branch` operation with the `-c` option to create a new Git branch
/// with the provided name. If successful, it updates the branch view by calling `handle_show_branches_button`.
/// Displays error messages using GTK message dialogs and the console if any issues occur.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `name`: The name of the new Git branch to be created.
///
fn create_branch(builder: &Builder, name: String) {
    let mut output: Vec<u8> = vec![];
    match git_branch(Some(name), Some("-c"), None, &mut output) {
        Ok(_) => match handle_show_branches_button(builder) {
            Ok(_) => {}
            Err(_) => {
                show_message_dialog("Error", "We couldn't update the view");
            }
        },
        Err(_) => {
            let output_string = match String::from_utf8(output) {
                Ok(string) => string,
                Err(_e) => {
                    show_message_dialog("Fatal error", "Something unexpected happened.");
                    return;
                }
            };
            show_message_dialog("Error", &output_string);
        }
    }
}

/// Handles the action triggered by the "Create Branch" button in a GTK application.
///
/// This function prompts the user to enter the name of the branch via a text entry window.
/// Upon receiving the branch name, it attempts to create a new Git branch using the entered name.
/// If successful, it updates the Git branch view by calling `handle_show_branches_button`.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components or the Git operation.
///
fn handle_create_branch_button(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let create_result = create_text_entry_window("Enter the name of the branch", move |name| {
        create_branch(&builder_clone, name);
    });

    if create_result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

/// Creates a new Git branch with the specified name based on an existing branch.
///
/// This function utilizes the `git_branch` operation to create a new branch with the provided name.
/// If successful, it updates the Git branch view by calling `handle_show_branches_button`.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `new_branch_name`: The name of the new branch to be created.
/// - `existing_branch_name`: The name of the existing branch that the new branch will be based on.
///
fn create_branch_from_another_branch(
    builder: &Builder,
    new_branch_name: String,
    existing_branch_name: &str,
) {
    let mut output: Vec<u8> = Vec::new();
    match git_branch(
        Some(new_branch_name),
        None,
        Some(existing_branch_name),
        &mut output,
    ) {
        Ok(_) => match handle_show_branches_button(builder) {
            Ok(_) => {}
            Err(_) => {
                show_message_dialog("Error", "We couldn't update the view");
            }
        },
        Err(_) => {
            let output_string = match String::from_utf8(output) {
                Ok(string) => string,
                Err(_e) => {
                    show_message_dialog("Fatal error", "Something unexpected happened.");
                    return;
                }
            };
            show_message_dialog("Error", &output_string);
        }
    }
}

/// Handles the action triggered by the "Create Branch from Branch" button in a GTK application.
///
/// This function prompts the user to enter the name of the new branch and the name of the existing branch
/// via a text entry window. It then creates a new Git branch based on the existing branch using the entered names.
/// If successful, it updates the Git branch view by calling `handle_show_branches_button`.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components.
///
fn handle_create_branch_from_branch_button(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let create_result = create_text_entry_window2(
        "Name of new branch",
        "Name of base branch",
        move |new_branch_name, existing_branch_name| {
            create_branch_from_another_branch(
                &builder_clone,
                new_branch_name,
                &existing_branch_name,
            );
        },
    );

    if let Err(err) = create_result {
        eprintln!("Error creating text entry window: {}", err);
    }

    Ok(())
}

/// Deletes a Git branch with the specified name.
///
/// This function utilizes the `git_branch` operation to delete a Git branch with the provided name.
/// If successful, it updates the Git branch view by calling `handle_show_branches_button`.
/// Displays success or error messages using GTK message dialogs accordingly.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `name`: The name of the branch to be deleted.
///
fn delete_branch(builder: &Builder, name: String) {
    let mut output: Vec<u8> = Vec::new();
    match git_branch(Some(name), Some("-d"), None, &mut output) {
        Ok(_) => {
            match handle_show_branches_button(builder) {
                Ok(_) => {}
                Err(_e) => {
                    show_message_dialog("Error", "view not updated");
                }
            }
            let texto = match str::from_utf8(&output) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Error turning result into string.");
                    "Error obtaining TextView"
                }
            };

            show_message_dialog("Success", texto);
        }
        Err(error) => {
            show_message_dialog("Error", &error.to_string());
        }
    }
}

/// Handles the action triggered by the "Delete Branch" button in a GTK application.
///
/// This function prompts the user to enter the name of the branch via a text entry window.
/// It then deletes the specified Git branch using the entered name and updates the Git branch view.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components or the Git operation.
///
fn handle_delete_branch_button(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();

    let create_result = create_text_entry_window("Enter the name of the branch", move |name| {
        delete_branch(&builder_clone, name);
    });

    if let Err(err) = create_result {
        eprintln!("Error creating text entry window: {}", err);
    }

    Ok(())
}

/// Modifies the name of the current Git branch.
///
/// This function attempts to modify the name of the current Git branch using the provided text parameters.
/// If successful, it updates the Git branch view by calling `handle_show_branches_button`.
/// Displays error messages using GTK message dialogs accordingly.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `text1`: The first text parameter, indicating the current branch name. If not empty, an error is displayed.
/// - `text2`: The second text parameter, representing the new name for the current branch.
///
fn modify_current_branch(builder: &gtk::Builder, text1: &str, text2: &str) {
    if !text1.is_empty() {
        show_message_dialog("Error", "Option 'actual branch' is on. Please disable this option if you want to change the name of another branch or leave the first field empty to change the name of the current branch.");
    } else if text2.is_empty() {
        show_message_dialog("Error", "Please , insert new branch name.");
    } else {
        let mut output: Vec<u8> = vec![];
        match git_branch(None, Some("-m"), Some(text2), &mut output) {
            Ok(_) => match handle_show_branches_button(builder) {
                Ok(_) => {}
                Err(_error) => {
                    show_message_dialog("Error", "view not updated");
                }
            },
            Err(error) => {
                show_message_dialog("Error", &error.to_string());
            }
        }
    }
}

/// Updates the name of a specified Git branch.
///
/// This function attempts to update the name of a specified Git branch using the provided text parameters.
/// If successful, it updates the Git branch view by calling `handle_show_branches_button`.
/// Displays error messages using GTK message dialogs accordingly.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `text1`: The current name of the branch to be updated.
/// - `text2`: The new name for the specified branch.
///
fn update_git_branch(builder: &Builder, text1: String, text2: &str) {
    let mut output: Vec<u8> = vec![];
    match git_branch(Some(text1), Some("-m"), Some(text2), &mut output) {
        Ok(_) => match handle_show_branches_button(builder) {
            Ok(_) => {}
            Err(_error) => {
                show_message_dialog("Error", "View not updated");
            }
        },
        Err(error) => {
            show_message_dialog("Error", &error.to_string());
        }
    }
}

/// Modifies the name of a specific Git branch.
///
/// This function attempts to modify the name of a specified Git branch using the provided text parameters.
/// If successful, it updates the Git branch view by calling `update_git_branch`.
/// Displays an error message using GTK message dialog if either of the text parameters is empty.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `text1`: The first text parameter, representing the current name of the branch to be modified.
/// - `text2`: The second text parameter, indicating the new name for the specified branch.
///
fn modify_branch(builder: &gtk::Builder, text1: &str, text2: &str) {
    if text1.is_empty() || text2.is_empty() {
        show_message_dialog(
            "Error",
            "You must enter the name of the branch to modify and the new name.",
        );
    } else {
        update_git_branch(builder, text1.to_string(), text2);
    }
}

/// Handles the action triggered by the "Modify Branch" button in a GTK application.
///
/// This function prompts the user to enter the current and new names of the branch via a text entry window,
/// along with an option to specify if the modification applies to the current branch.
/// It then either modifies the name of the current Git branch or another specified branch
/// and updates the Git branch view accordingly.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components.
///
fn handle_modify_branch_button(builder: &gtk::Builder) -> io::Result<()> {
    let message1 = "actual Name";
    let message2 = "New name ";
    let builder_clone = builder.clone();
    create_text_entry_window_with_switch(message1, message2, move |text1, text2, switch_value| {
        if switch_value {
            modify_current_branch(&builder_clone, &text1, &text2);
        } else {
            modify_branch(&builder_clone, &text1, &text2);
        }
    })?;

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
                let result = set_staging_area_texts(&builder_clone);
                if result.is_err() {
                    eprintln!("staging view not updated.");
                }
            }
            Err(_err) => {
                show_message_dialog("Error", "wrong path.");
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

    let index_path = format!("{}/{}", git_dir, INDEX);
    match add(
        "None",
        &index_path,
        &git_dir,
        &git_ignore_path,
        Some(vec![".".to_string()]),
    ) {
        Ok(_) => {}
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Error in 'add': {:?}", err),
            ))
        }
    }
    let result = set_staging_area_texts(&builder_clone);
    if result.is_err() {
        eprintln!("staging view not updated.");
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
                    eprintln!("staging view not updated");
                }
            }
            Err(_err) => {
                show_message_dialog("Error", "wrong path");
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Adds a new remote to the Git configuration.
///
/// This function adds a new remote to the Git configuration using the provided
/// name and URL. It interacts with the `git_remote` function to perform the
/// necessary Git commands.
///
/// # Arguments
///
/// * `name` - The name of the remote to be added.
/// * `url` - The URL of the remote repository.
///
/// # Returns
///
/// A `Result` indicating whether the operation was successful or resulted in an error.
///
pub fn obtain_text_from_remote_add(name: &str, url: &str) -> Result<String, io::Error> {
    let git_dir = utils::obtain_git_dir()?;

    let mut config = Config::load(&git_dir)?;

    let line: Vec<&str> = vec!["add", name, url];

    let mut output: Vec<u8> = vec![];
    let result = git_remote(&mut config, line, &mut output);
    let string =
        String::from_utf8(output).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    drop(config);
    if result.is_err() {
        return Err(io::Error::new(io::ErrorKind::Other, string));
    }

    Ok(string)
}

/// Removes a remote from the Git configuration.
///
/// This function removes a remote from the Git configuration using the provided
/// remote name. It interacts with the `git_remote` function to perform the
/// necessary Git commands.
///
/// # Arguments
///
/// * `text` - The name of the remote to be removed.
///
/// # Returns
///
/// A `Result` indicating whether the operation was successful or resulted in an error.
///
pub fn obtain_text_from_remote_rm(text: &str) -> Result<String, io::Error> {
    let git_dir = obtain_git_dir()?;

    let mut config = Config::load(&git_dir)?;

    let line: Vec<&str> = vec!["remove", text];

    let mut output: Vec<u8> = vec![];
    let result = git_remote(&mut config, line, &mut output);
    drop(config);
    let string =
        String::from_utf8(output).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if result.is_err() {
        return Err(io::Error::new(io::ErrorKind::Other, string));
    }

    Ok(string)
}

/// Sets the URL of a remote repository in the Git configuration.
///
/// This function sets the URL of a remote repository in the Git configuration using the
/// provided remote name, and the new URL. It interacts with the `git_remote` function to
/// perform the necessary Git commands.
///
/// # Arguments
///
/// * `name` - The name of the remote repository.
/// * `url` - The new URL to set for the remote repository.
///
/// # Returns
///
/// A `Result` indicating whether the operation was successful or resulted in an error.
///
pub fn obtain_text_from_remote_set_url(name: &str, url: &str) -> Result<String, io::Error> {
    let git_dir = obtain_git_dir()?;

    let mut config = Config::load(&git_dir)?;

    let line: Vec<&str> = vec!["set-url", name, url];

    let mut output: Vec<u8> = vec![];
    let result = git_remote(&mut config, line, &mut output);
    drop(config);
    let string =
        String::from_utf8(output).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if result.is_err() {
        return Err(io::Error::new(io::ErrorKind::Other, string));
    }

    Ok(string)
}

/// Obtains the URL of a remote repository from the Git configuration.
///
/// This function retrieves the URL of a remote repository from the Git configuration using the
/// provided remote name. It interacts with the `git_remote` function to perform the necessary
/// Git commands.
///
/// # Arguments
///
/// * `text` - The name of the remote repository.
///
/// # Returns
///
/// A `Result` containing the URL of the remote repository if the operation was successful,
/// otherwise an error indicating the failure.
///
pub fn obtain_text_from_remote_get_url(text: &str) -> Result<String, io::Error> {
    let git_dir = utils::obtain_git_dir()?;

    let mut config = Config::load(&git_dir)?;

    let line: Vec<&str> = vec!["get-url", text];

    let mut output: Vec<u8> = vec![];
    let result = git_remote(&mut config, line, &mut output);
    drop(config);
    let string =
        String::from_utf8(output).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if result.is_err() {
        return Err(io::Error::new(io::ErrorKind::Other, string));
    }

    Ok(string)
}

/// Renames a remote repository in the Git configuration.
///
/// This function renames a remote repository in the Git configuration from the old name to the new
/// name. It interacts with the `git_remote` function to perform the necessary Git commands.
///
/// # Arguments
///
/// * `old_name` - The current name of the remote repository.
/// * `new_name` - The new name to be assigned to the remote repository.
///
/// # Returns
///
/// A `Result` containing a success message if the operation was successful, otherwise an error
/// indicating the failure.
///
pub fn obtain_text_from_remote_rename(old_name: &str, new_name: &str) -> Result<String, io::Error> {
    let git_dir = obtain_git_dir()?;

    let mut config = Config::load(&git_dir)?;

    let line: Vec<&str> = vec!["rename", old_name, new_name];

    let mut output: Vec<u8> = vec![];
    let result = git_remote(&mut config, line, &mut output);
    drop(config);
    let string =
        String::from_utf8(output).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if result.is_err() {
        return Err(io::Error::new(io::ErrorKind::Other, string));
    }

    Ok(string)
}

/// Handles the addition of a remote repository.
///
/// This function displays a text entry window with fields for the name and URL of a remote repository.
/// It then calls `obtain_text_from_remote_add` to perform the necessary Git commands based on the
/// provided name and URL.
///
/// # Returns
///
/// A `Result` indicating success or failure. If successful, a message is displayed; otherwise, an
/// error message is shown.
///
fn handle_remote_add(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let result =
        create_text_entry_window2("Enter remote name", "Enter remote URL", move |name, url| {
            let resultado = obtain_text_from_remote_add(&name, &url);
            match resultado {
                Ok(_texto) => match handle_remote(&builder_clone) {
                    Ok(_) => {}
                    Err(_e) => {
                        show_message_dialog("Error", "Couldn't update the view!");
                    }
                },
                Err(_err) => {
                    let error_mesage = _err.to_string();
                    show_message_dialog("Error", &error_mesage)
                }
            }
        });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Handles the removal of a remote repository.
///
/// This function displays a text entry window for the user to enter the name of the remote repository
/// to be removed. It then calls `obtain_text_from_remote_rm` to perform the necessary Git commands
/// based on the provided repository name.
///
/// # Returns
///
/// A `Result` indicating success or failure. If successful, a message is displayed; otherwise, an
/// error message is shown.
///
fn handle_remote_rm(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let result = create_text_entry_window("Enter repository name", move |text| {
        let resultado = obtain_text_from_remote_rm(&text);
        match resultado {
            Ok(_texto) => match handle_remote(&builder_clone) {
                Ok(_) => {}
                Err(_e) => {
                    show_message_dialog("Error", "Couldn't update the view!");
                }
            },
            Err(_err) => {
                let error_mesage = _err.to_string();
                show_message_dialog("Error", &error_mesage)
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Adds a normal Git tag with the specified name.
///
/// This function utilizes the `git_tag` operation to add a normal Git tag with the provided name.
/// If successful, it updates the Git tag view by calling `handle_list_tags`.
/// Displays error messages using GTK message dialogs accordingly.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `git_dir`: The path to the Git directory.
/// - `tag_name`: The name of the normal Git tag to be added.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the Git operation.
///
pub fn add_normal_tag(builder: &gtk::Builder, git_dir: &str, tag_name: &str) -> io::Result<()> {
    let line = vec![
        String::from("git"),
        String::from("tag"),
        tag_name.to_string(),
    ];
    let mut output: Vec<u8> = vec![];

    match git_tag(git_dir, line, &mut output) {
        Ok(_texto) => match handle_list_tags(builder) {
            Ok(_) => {}
            Err(_e) => {
                show_message_dialog("Error", "We had a problem trying to refresh the view.");
            }
        },
        Err(_err) => {
            let error_message = _err.to_string();
            show_message_dialog("Error", &error_message);
        }
    }

    Ok(())
}

/// Handles the action triggered by the "Add Normal Tag" button in a GTK application.
///
/// This function prompts the user to enter a tag name via a text entry window
/// and then attempts to add a normal Git tag using the entered name.
/// Displays an error message in the console if there are issues with the Git operation.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components or the Git operation.
///
fn handle_tag_add_normal(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let git_dir = obtain_git_dir()?;

    let result = create_text_entry_window("Enter tag name", move |tag_name| {
        match add_normal_tag(&builder_clone, &git_dir, &tag_name) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("{}", error);
            }
        }
    });

    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

/// Removes a Git tag with the specified name.
///
/// This function utilizes the `git_tag` operation with the `-d` option to remove a Git tag with the provided name.
/// If successful, it updates the Git tag view by calling `handle_list_tags`.
/// Displays error messages using GTK message dialogs accordingly.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `name`: The name of the Git tag to be removed.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the Git operation.
///
pub fn remove_tag(builder: &Builder, name: &str) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;

    let line = vec![
        String::from("git"),
        String::from("tag"),
        String::from("-d"),
        name.to_string(),
    ];

    let mut output: Vec<u8> = vec![];
    match git_tag(&git_dir, line, &mut output) {
        Ok(_) => match handle_list_tags(builder) {
            Ok(_) => {}
            Err(_e) => {
                show_message_dialog("Error", "We had a problem trying to refresh the view.");
            }
        },
        Err(_err) => {
            let error_message = _err.to_string();
            show_message_dialog("Error", &error_message);
        }
    }

    Ok(())
}

/// Handles the action triggered by the "Remove Tag" button in a GTK application.
///
/// This function prompts the user to enter a tag name via a text entry window
/// and then attempts to remove the specified Git tag.
/// Displays an error message in the console if there are issues with the Git operation.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components or the Git operation.
///
fn handle_tag_remove(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let result = create_text_entry_window("Enter tag name", move |name| {
        match remove_tag(&builder_clone, &name) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("{}", error)
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Updates the GTK TextView with verified tag information.
///
/// This function takes a GTK builder and the output vector from a Git operation
/// as input and updates the specified TextView with the verified tag information.
/// If successful, it returns the tag information as a String.
/// Displays error messages using the console and an `io::Error` if any issues occur.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `output`: A mutable reference to a vector containing the output of a Git operation.
///
/// # Returns
///
/// Returns the tag information as a `String` on success or an `io::Error` if there are issues with the UI components
/// or converting the Git output to a string.
///
fn update_view_with_verified_tag(builder: &Builder, output: &mut [u8]) -> io::Result<String> {
    let tags_text_view: gtk::TextView = match builder.get_object("tag-text") {
        Some(text_view) => text_view,
        None => {
            eprintln!("Text view not found.");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error turning result into string",
            ));
        }
    };

    let text = match str::from_utf8(output) {
        Ok(s) => s.to_string(),
        Err(_) => {
            eprintln!("Error turning result into string.");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error turning result into string",
            ));
        }
    };

    if let Some(buffer) = tags_text_view.get_buffer() {
        buffer.set_text(&text);
    } else {
        eprintln!("Error obtaining TextView.");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error obtaining TextView",
        ));
    }
    Ok(text)
}

/// Verifies a Git tag with the specified name and updates the GTK TextView.
///
/// This function utilizes the `git_tag` operation with the `-v` option to verify a Git tag with the provided name.
/// If successful, it updates the specified TextView with the verified tag information.
/// Returns the tag information as a `Result<String, io::Error>`.
/// Displays error messages using the console and an `io::Error` if any issues occur.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `tag_name`: The name of the Git tag to be verified.
///
/// # Returns
///
/// Returns the verified tag information as a `Result<String, io::Error>`.
///
pub fn verify_tag(builder: &Builder, tag_name: &str) -> Result<String, io::Error> {
    let git_dir = obtain_git_dir()?;

    let line = vec![
        String::from("git"),
        String::from("tag"),
        String::from("-v"),
        tag_name.to_string(),
    ];

    let mut output: Vec<u8> = vec![];
    match git_tag(&git_dir, line, &mut output) {
        Ok(_config) => {}
        Err(error) => {
            eprintln!("{:?}", error);
            return Err(error);
        }
    }

    let text = update_view_with_verified_tag(builder, &mut output)?;

    Ok(text)
}

/// Handles the action triggered by the "Verify Tag" button in a GTK application.
///
/// This function prompts the user to enter a tag name via a text entry window
/// and then attempts to verify the specified Git tag.
/// Displays an error message using GTK message dialogs if there are issues with the Git operation.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components or the Git operation.
///
fn handle_tag_verify(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();

    let result = create_text_entry_window("Enter tag name", move |name| {
        let resultado = verify_tag(&builder_clone, &name);
        match resultado {
            Ok(_texto) => {}
            Err(_err) => {
                let error_message =
                    format!("error: {name}: cannot verify a non-tag object of type commit.");
                show_message_dialog("Error", &error_message);
            }
        }
    });

    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

/// Adds an annotated Git tag with the specified name and message.
///
/// This function utilizes the `git_tag` operation with the `-a` option to add an annotated Git tag
/// with the provided name and message. If successful, it updates the Git tag view by calling `handle_list_tags`.
/// Displays error messages using GTK message dialogs and the console if any issues occur.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `name`: The name of the Git tag to be added.
/// - `message`: The message associated with the Git tag.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the Git operation.
///
pub fn add_annotated_tag(builder: &gtk::Builder, name: &str, message: &str) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;

    let line = vec![
        String::from("git"),
        String::from("tag"),
        String::from("-a"),
        name.to_string(),
        String::from("-m"),
        message.to_string(),
    ];

    let mut output: Vec<u8> = vec![];
    println!("Git dir: {}, line {:?}", git_dir, line);
    match git_tag(&git_dir, line, &mut output) {
        Ok(_texto) => match handle_list_tags(builder) {
            Ok(_) => {}
            Err(_e) => {
                show_message_dialog("Error", "We had a problem trying to refresh the view.");
            }
        },
        Err(_err) => {
            let error_message = _err.to_string();
            show_message_dialog("Error", &error_message);
        }
    }

    Ok(())
}

/// Handles the action triggered by the "Add Annotated Tag" button in a GTK application.
///
/// This function prompts the user to enter a tag name and a tag message via a text entry window
/// and then attempts to add an annotated Git tag using the entered name and message.
/// Displays error messages in the console and GTK message dialogs accordingly.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components or the Git operation.
///
fn handle_tag_add_annotated(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let result = create_text_entry_window2(
        "Enter tag name",
        "Enter tag message",
        move |name, message| {
            if name.is_empty() || message.is_empty() {
                show_message_dialog("Error", "Insert name and message for tag ");
            } else {
                let resultado = add_annotated_tag(&builder_clone, &name, &message);
                match resultado {
                    Ok(_) => {}
                    Err(_err) => {
                        let error_message = _err.to_string();
                        eprintln!("{}", error_message);
                    }
                }
            }
        },
    );

    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

/// Creates a new Git tag with a specified name based on an existing Git tag.
///
/// This function utilizes the `git_tag` operation to create a new Git tag with the provided name (`new_name`)
/// based on an existing Git tag with the name (`old_name`). If successful, it updates the Git tag view
/// by calling `handle_list_tags`. Displays error messages using GTK message dialogs and the console
/// if any issues occur.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `new_name`: The name of the new Git tag to be created.
/// - `old_name`: The name of the existing Git tag to base the new tag on.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the Git operation.
///
pub fn create_tag_from_other_tag(
    builder: &gtk::Builder,
    new_name: &str,
    old_name: &str,
) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;

    let line = vec![
        String::from("git"),
        String::from("tag"),
        new_name.to_string(),
        old_name.to_string(),
    ];

    let mut output: Vec<u8> = vec![];
    match git_tag(&git_dir, line, &mut output) {
        Ok(_texto) => match handle_list_tags(builder) {
            Ok(_) => {}
            Err(_e) => {
                show_message_dialog("Error", "We had a problem trying to refresh the view.");
            }
        },
        Err(_err) => {
            let error_message = _err.to_string();
            show_message_dialog("Error", &error_message);
        }
    }

    Ok(())
}

/// Handles the action triggered by the "Create Tag from Tag" button in a GTK application.
///
/// This function prompts the user to enter the name of the new tag and the name of the base tag
/// via a text entry window and then attempts to create a new Git tag based on an existing tag.
/// Displays error messages in the console and GTK message dialogs accordingly.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components or the Git operation.
///
fn handle_tag_from_tag(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let result = create_text_entry_window2(
        "New tag name",
        "Name of base tag ",
        move |new_tag_name, base_tag_name| {
            if new_tag_name.is_empty() || base_tag_name.is_empty() {
                show_message_dialog(
                    "Error",
                    "Debe proveer el New tag name y el Name of base tag ",
                );
            } else {
                let resultado =
                    create_tag_from_other_tag(&builder_clone, &new_tag_name, &base_tag_name);
                match resultado {
                    Ok(_) => {}
                    Err(_err) => {
                        let error_message = _err.to_string();
                        eprintln!("{error_message}");
                    }
                }
            }
        },
    );
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Calls the `git ls-tree` command to retrieve tree information based on the specified hash and option.
///
/// This function obtains the Git directory (assumed to be in a folder named ".mgit") and
/// executes the `git ls-tree` command with the provided hash and option. The output of the command
/// is then used to update a GTK text view within the specified GTK builder.
///
/// # Arguments
/// * `option` - The option to be used with the `git ls-tree` command.
/// * `hash` - The hash or branch name to identify the specific tree.
/// * `builder` - A reference to the GTK builder containing the text view to be updated.
///
fn call_ls_trees(option: &str, hash: &str, builder: &gtk::Builder) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("git directory not found .");
            return;
        }
    };

    let mut output: Vec<u8> = vec![];
    match ls_tree(hash, &git_dir, option, &mut output) {
        Ok(_texto) => match update_list_trees_view(builder, output, "trees-text") {
            Ok(_) => {}
            Err(_e) => {
                show_message_dialog("Error", "View not updated");
            }
        },
        Err(_err) => {
            show_message_dialog("Error", &_err.to_string());
        }
    }
}

/// Handles the execution of the `git ls-tree` command based on user input through a text entry window.
///
/// This function prompts the user to enter a hash or branch name using a text entry window. Upon
/// receiving the input, it calls the `call_ls_trees` function to retrieve tree information and
/// update a GTK text view within the provided GTK builder.
///
/// # Arguments
/// * `builder` - A reference to the GTK builder containing the elements required for interaction.
///
/// # Returns
/// An `io::Result` indicating success or an error in creating the text entry window.
///
fn handle_ls_trees(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();

    let result = create_text_entry_window("Enter hash", move |hash| {
        if hash.is_empty() {
            show_message_dialog("Error", "insert hash");
        } else {
            call_ls_trees("", &hash, &builder_clone);
        }
    });

    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

/// Handles the execution of the `git ls-tree -r` command based on user input through a text entry window.
///
/// This function prompts the user to enter a hash or branch name using a text entry window. Upon
/// receiving the input, it calls the `call_ls_trees` function with the `-r` option to retrieve
/// recursive tree information and updates a GTK text view within the provided GTK builder.
///
/// # Arguments
/// * `builder` - A reference to the GTK builder containing the elements required for interaction.
///
/// # Returns
/// An `io::Result` indicating success or an error in creating the text entry window.
///
fn handle_ls_trees_r(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();

    let result = create_text_entry_window("Enter hash", move |hash| {
        if hash.is_empty() {
            show_message_dialog("Error", "insert hash");
        } else {
            call_ls_trees("-r", &hash, &builder_clone);
        }
    });

    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Handles the execution of the `git ls-tree -d` command based on user input through a text entry window.
///
/// This function prompts the user to enter a hash or branch name using a text entry window. Upon
/// receiving the input, it calls the `call_ls_trees` function with the `-d` option to retrieve
/// only directories in the tree and updates a GTK text view within the provided GTK builder.
///
/// # Arguments
/// * `builder` - A reference to the GTK builder containing the elements required for interaction.
///
/// # Returns
/// An `io::Result` indicating success or an error in creating the text entry window.
///
fn handle_ls_trees_d(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();

    let result = create_text_entry_window("Enter hash", move |hash| {
        if hash.is_empty() {
            show_message_dialog("Error", "insert hash");
        } else {
            call_ls_trees("-d", &hash, &builder_clone);
        }
    });

    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

/// Handles the execution of the `git ls-tree -r -t` command based on user input through a text entry window.
///
/// This function prompts the user to enter a hash or branch name using a text entry window. Upon
/// receiving the input, it calls the `call_ls_trees` function with the `-r-t` option to retrieve
/// only subtrees in the recursive tree and updates a GTK text view within the provided GTK builder.
///
/// # Arguments
/// * `builder` - A reference to the GTK builder containing the elements required for interaction.
///
/// # Returns
/// An `io::Result` indicating success or an error in creating the text entry window.
///
fn handle_ls_trees_rt(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();

    let result = create_text_entry_window("Enter hash", move |hash| {
        if hash.is_empty() {
            show_message_dialog("Error", "insert hash");
        } else {
            call_ls_trees("-r-t", &hash, &builder_clone);
        }
    });

    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }

    Ok(())
}

/// Updates a GTK text view with the provided output.
///
/// This function takes a reference to a GTK builder, a vector of bytes representing the
/// output of a command, and the ID of the text view widget. It attempts to retrieve the
/// specified text view from the builder, convert the byte vector to a UTF-8 string, and
/// sets the content of the text view to the resulting string.
///
/// # Arguments
/// * `builder` - A reference to the GTK builder containing the text view to be updated.
/// * `output` - A vector of bytes representing the output content.
/// * `id` - The ID of the GTK text view widget within the builder.
///
/// # Returns
/// A `Result` containing the converted output string on success or an `io::Error` on failure.
///
fn update_list_trees_view(builder: &gtk::Builder, output: Vec<u8>, id: &str) -> io::Result<String> {
    let tree_text_view: gtk::TextView = match builder.get_object(id) {
        Some(text_view) => text_view,
        None => {
            eprintln!("Error obtaining text view for list trees.");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error obtaining text view for list trees",
            ));
        }
    };

    let text = match String::from_utf8(output) {
        Ok(s) => s,

        Err(_) => {
            eprintln!("Error turning result into string.");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error turning result into string",
            ));
        }
    };

    if let Some(buffer) = tree_text_view.get_buffer() {
        buffer.set_text(&text);
    } else {
        eprintln!("Error obtaining TextView.");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error obtaining TextView",
        ));
    }
    Ok(text)
}

/// Handles setting the URL for a remote repository.
///
/// This function displays a text entry window for the user to enter the name of the remote repository
/// and the new URL. It then calls `obtain_text_from_remote_set_url` to perform the necessary Git
/// commands based on the provided repository name and URL.
///
/// # Returns
///
/// A `Result` indicating success or failure. If successful, a message is displayed; otherwise, an
/// error message is shown.
///
fn handle_remote_set_url() -> io::Result<()> {
    let result = create_text_entry_window2("Enter repo name", "Enter new URL", |name, url| {
        let resultado = obtain_text_from_remote_set_url(&name, &url);
        match resultado {
            Ok(_texto) => {
                show_message_dialog("Success", "URL updated correctly.");
            }
            Err(_err) => {
                let error_mesage = _err.to_string();
                show_message_dialog("Error", &error_mesage)
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Handles getting the URL for a remote repository.
///
/// This function displays a text entry window for the user to enter the name of the remote repository.
/// It then calls `obtain_text_from_remote_get_url` to perform the necessary Git commands based on the
/// provided repository name and obtain the URL.
///
/// # Returns
///
/// A `Result` indicating success or failure. If successful, a message is displayed with the obtained URL;
/// otherwise, an error message is shown.
///
fn handle_remote_get_url() -> io::Result<()> {
    let result = create_text_entry_window("Enter the name of the repository", move |text| {
        let resultado = obtain_text_from_remote_get_url(&text);
        match resultado {
            Ok(texto) => {
                show_message_dialog("Success", &format!("{}'", texto));
            }
            Err(_err) => {
                let error_mesage = _err.to_string();
                show_message_dialog("Error", &error_mesage)
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Handles renaming a remote repository.
///
/// This function displays a text entry window for the user to enter the old and new names of the remote repository.
/// It then calls `obtain_text_from_remote_rename` to perform the necessary Git commands based on the provided
/// old and new repository names and renames the remote repository.
///
/// # Returns
///
/// A `Result` indicating success or failure. If successful, a message is displayed with the result;
/// otherwise, an error message is shown.
///
fn handle_remote_rename(builder: &gtk::Builder) -> io::Result<()> {
    let builder_clone = builder.clone();
    let result = create_text_entry_window2(
        "Enter old repo name",
        "Enter new repo name",
        move |old_name, new_name| {
            let resultado = obtain_text_from_remote_rename(&old_name, &new_name);
            match resultado {
                Ok(_texto) => match handle_remote(&builder_clone) {
                    Ok(_) => {}
                    Err(_e) => {
                        show_message_dialog("Error", "Couldn't update the view!");
                    }
                },
                Err(_err) => {
                    let error_mesage = _err.to_string();
                    show_message_dialog("Error", &error_mesage)
                }
            }
        },
    );
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
    Ok(())
}

/// Creates a new Git tag with a specified name based on an existing Git tag.
///
/// This function utilizes the `git_tag` operation to create a new Git tag with the provided name (`new_name`)
/// based on an existing Git tag with the name (`old_name`). If successful, it updates the Git tag view
/// by calling `handle_list_tags`. Displays error messages using GTK message dialogs and the console
/// if any issues occur.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `new_name`: The name of the new Git tag to be created.
/// - `old_name`: The name of the existing Git tag to base the new tag on.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the Git operation.
///
fn update_remote_view(builder: &gtk::Builder, output: &mut [u8]) -> io::Result<()> {
    let remote_text_view: gtk::TextView = match builder.get_object("remote-text") {
        Some(text_view) => text_view,
        None => {
            eprintln!("Couldn't get remote text view.");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Couldn't get remote text view.",
            ));
        }
    };

    let text = match str::from_utf8(output) {
        Ok(s) => s.to_string(),
        Err(_) => {
            eprintln!("Error turning result into string.");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error turning result into string",
            ));
        }
    };

    if let Some(buffer) = remote_text_view.get_buffer() {
        buffer.set_text(&text);
    } else {
        eprintln!("Error obtaining TextView.");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error obtaining TextView",
        ));
    }
    Ok(())
}

/// Handles the display of remote repositories in a TextView.
///
/// This function retrieves the list of remote repositories using Git commands and displays the result in a TextView.
///
/// # Arguments
///
/// - `builder`: A reference to the GTK builder containing the TextView widget.
///
/// # Returns
///
/// A `Result` indicating success or failure. If successful, the list of remote repositories is displayed in the TextView;
/// otherwise, an error message is shown.
///
fn handle_remote(builder: &gtk::Builder) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;

    let mut config = Config::load(&git_dir)?;

    let mut output: Vec<u8> = vec![];
    match git_remote(&mut config, vec!["remote"], &mut output) {
        Ok(_) => {}
        Err(_e) => {
            let error = _e.to_string();
            eprintln!("{}", error);
            return Err(io::Error::new(io::ErrorKind::Other, "&error"));
        }
    }

    drop(config);

    update_remote_view(builder, &mut output)?;

    Ok(())
}

/// Updates the GTK tag view with the provided output.
///
/// This function takes a GTK builder reference and a mutable slice of bytes (`output`) as input.
/// It retrieves the tag text view from the builder, converts the output bytes to a UTF-8 string,
/// and updates the text view with the obtained string. Displays error messages in the console
/// and returns an `io::Error` if any issues occur.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
/// - `output`: A mutable slice of bytes representing the output to be displayed in the tag text view.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the UI components or the conversion process.
///
fn update_tag_view(builder: &gtk::Builder, output: &mut [u8]) -> io::Result<()> {
    let tags_text_view: gtk::TextView = match builder.get_object("tag-text") {
        Some(view) => view,
        None => {
            eprintln!("obtainig text view failed");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error obtaining text view",
            ));
        }
    };

    let text = match str::from_utf8(output) {
        Ok(s) => s.to_string(),
        Err(_) => {
            eprintln!("Error turning result into string.");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Error turning result into string",
            ));
        }
    };

    if let Some(buffer) = tags_text_view.get_buffer() {
        buffer.set_text(&text);
    } else {
        eprintln!("Error obtaining TextView.");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error obtaining TextView",
        ));
    }

    Ok(())
}

/// Handles the action triggered by the "List Tags" button in a GTK application.
///
/// This function retrieves the Git directory path, performs a Git tag operation to list available tags,
/// and updates the tag view by calling `update_tag_view`. Displays error messages using GTK message dialogs
/// and the console if any issues occur.
///
/// # Arguments
///
/// - `builder`: A reference to a GTK builder containing the necessary UI components.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `io::Error` if there are issues with the Git operation or updating the view.
///
fn handle_list_tags(builder: &gtk::Builder) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;

    let line = vec![String::from("git"), String::from("tag"), String::from("-l")];

    let mut output: Vec<u8> = vec![];
    match git_tag(&git_dir, line, &mut output) {
        Ok(_config) => {}
        Err(error) => {
            eprintln!("{:?}", error);
            return Err(error);
        }
    }

    update_tag_view(builder, &mut output)?;

    Ok(())
}

// let result = show_current_branch_on_merge_window(&merge_text_view);
//                     if result.is_err() {
//                         eprintln!("No se pudo actualizar la rama actual en la ventana merge.");
//                     }
//                     let result = handle_show_branches_button(&builder_clone);
//                     if result.is_err() {
//                         eprintln!("No se pudo actualizar la rama actual en la ventana branch.");
//                     }

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
fn handle_checkout_branch_window(builder: &Builder) {
    let builder_clone = builder.clone();
    let result = create_text_entry_window("Enter the path of the file", move |text| {
        if text.is_empty() {
            show_message_dialog("Error", "The branch does not exists.");
        } else {
            let resultado = obtain_text_from_checkout_branch(&text);
            match resultado {
                Ok(_) => {
                    update_views_since_checkout_was_called(&builder_clone);
                }
                Err(err) => {
                    let error_message = err.to_string();
                    show_message_dialog("Error", &error_message);
                }
            }
        }
    });
    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
}

fn update_views_since_checkout_was_called(builder: &Builder) {
    let merge_text_view = match get_text_view(builder, "merge-text-view") {
        Some(view) => view,
        None => {
            eprintln!("No se pudo obtener el text view de merge");
            return;
        }
    };

    match show_current_branch_on_merge_window(&merge_text_view) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("No se pudo actualizar la rama actual en la ventana merge.");
        }
    }

    match handle_show_branches_button(builder) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("No se pudo actualizar la view");
        }
    }
}

/// Obtains the ScrolledWindow widget for displaying log text.
///
/// This function retrieves the ScrolledWindow widget from the GTK builder based on its identifier.
///
/// # Arguments
///
/// - `builder`: A reference to the GTK builder containing the ScrolledWindow widget.
///
/// # Returns
///
/// An `Option` containing the ScrolledWindow widget if found; otherwise, `None`.
///
fn obtain_log_text_scrolled_window(builder: &gtk::Builder) -> Option<gtk::ScrolledWindow> {
    builder.get_object("scroll-log")
}

/// Handle the "Show Log" button's click event. This function retrieves a text view widget from the GTK builder
/// and populates it with the Git log data. If the operation is successful, it displays the log data in the text view.
/// If there is an error, it prints an error message to the standard error.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK builder used to create UI elements.
fn handle_show_log_button_click(builder: &gtk::Builder) {
    let log_text_view_result: Option<gtk::TextView> = builder.get_object("log-text");

    if let Some(log_text_view) = log_text_view_result {
        let log_text_scrolled_window = match obtain_log_text_scrolled_window(builder) {
            Some(sw) => sw,
            None => {
                eprintln!("ScrolledWindow not obtained.");
                return;
            }
        };

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

                log_text_scrolled_window.add(&log_text_view);
                log_text_scrolled_window.show_all();
            }
            Err(err) => {
                eprintln!("Error obtaining text: {}", err);
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
    let git_dir = obtain_git_dir()?;
    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Working dir not found\n",
            ))
        }
    };
    let git_ignore_path = format!("{}/{}", working_dir, GIT_IGNORE);

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
        Ok(_) => {}
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Error in 'add': {:?}", err),
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
    let git_dir = find_git_directory(&mut current_dir, GIT_DIR)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Git directory not found\n"))?;
    let index_path = format!("{}/{}", git_dir, INDEX);
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_ignore_path = format!("{}/{}", git_dir_parent.to_string_lossy(), GIT_IGNORE);

    git_rm(texto, &index_path, &git_dir, &git_ignore_path)?;

    Ok(" 'rm' not working correctly".to_string())
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
    let git_dir = obtain_git_dir()?;
    let git_dir = Path::new(&git_dir);

    match force_checkout(git_dir, texto) {
        Ok(_) => Ok("Correctly changed".to_string()),
        Err(error) => Err(error),
    }
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
    let git_dir = obtain_git_dir()?;
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_dir_path = Path::new(&git_dir);
    match checkout_commit_detached(
        git_dir_path,
        git_dir_parent.to_string_lossy().as_ref(),
        texto,
    ) {
        Ok(_) => Ok("'checkout branch' works correctly.".to_string()),
        Err(err) => Err(err),
    }
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
    let git_dir = obtain_git_dir()?;
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_dir_path = Path::new(&git_dir);
    match create_or_reset_branch(
        git_dir_path,
        git_dir_parent.to_string_lossy().as_ref(),
        texto,
    ) {
        Ok(_) => Ok("'checkout branch' works correctly.".to_string()),
        Err(err) => Err(err),
    }
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
    let git_dir = obtain_git_dir()?;
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_dir_path = Path::new(&git_dir);

    match create_and_checkout_branch(
        git_dir_path,
        git_dir_parent.to_string_lossy().as_ref(),
        texto,
    ) {
        Ok(_) => Ok("'checkout branch' works correctly.".to_string()),
        Err(err) => Err(err),
    }
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
    let git_dir = obtain_git_dir()?;
    let git_dir_parent = Path::new(&git_dir)
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Gitignore file not found\n"))?;
    let git_dir_path = Path::new(&git_dir);

    match checkout_branch(
        git_dir_path,
        git_dir_parent.to_string_lossy().as_ref(),
        text,
    ) {
        Ok(_) => Ok("The 'checkout branch' function executed successfully.".to_string()),
        Err(err) => Err(err),
    }
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
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(_) => {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Not a git dir\n"));
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
    let git_dir = obtain_git_dir()?;
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
    let result = merge::git_merge_for_ui(
        &our_branch,
        their_branch,
        &git_dir,
        root_dir.to_string_lossy().as_ref(),
    )?;
    Ok(result)
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
    entry: &gtk::Entry,
    text_buffer: &gtk::TextBuffer,
    git_directory: String,
    conflicts: Rc<RefCell<Vec<String>>>,
) -> io::Result<Vec<String>> {
    let branch = entry.get_text().to_string();
    if branch.is_empty() {
        show_message_dialog("Error", "Please, enter a branch.");
    } else if !branch::is_an_existing_branch(&branch, git_directory.as_str()) {
        show_message_dialog("Error", "branch not found.");
    } else {
        let result = call_git_merge(&branch);
        match result {
            Ok(conflicts_list) => {
                if conflicts_list.is_empty() {
                    text_buffer.set_text("Succesfull Merge .");
                } else {
                    let mut conflicts_text = String::new();
                    conflicts_text.push_str("Conflict(s) found:\n");
                    for conflict in &conflicts_list {
                        conflicts_text.push_str(conflict);
                        conflicts_text.push('\n');
                    }
                    text_buffer.set_text(&conflicts_text);

                    let mut conflicts_ref = conflicts.borrow_mut();
                    *conflicts_ref = conflicts_list;
                }
            }
            Err(e) => {
                let error_message = e.to_string();
                show_message_dialog("Error", &error_message);
            }
        }
    }
    let conflict_clone = conflicts.borrow();
    Ok(conflict_clone.clone())
}

/// Turns off the abort, update and done buttons when the merge button is clicked.
fn turn_off_buttons_on_button_click(
    button: &Button,
    ok_button: &Button,
    abort_button: &Button,
    update_button: &Button,
    merge_button: &Button,
) {
    let ok_button_clone = ok_button.clone();
    let abort_button_clone = abort_button.clone();
    let update_button_clone = update_button.clone();
    let merge_button_clone = merge_button.clone();
    button.connect_clicked(move |_| {
        ok_button_clone.set_sensitive(false);
        abort_button_clone.set_sensitive(false);
        update_button_clone.set_sensitive(false);
        merge_button_clone.set_sensitive(true);
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
    ok_button: &Button,
    abort_button: &Button,
    update_button: &Button,
    merge_combo_box_text: &ComboBoxText,
) -> io::Result<Vec<String>> {
    let git_dir = obtain_git_dir()?;
    let conflicts = Rc::new(RefCell::new(Vec::<String>::new()));
    let conflicts_clone = conflicts.clone();
    let text_buffer = match text_view.get_buffer() {
        Some(buff) => buff,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Text view buffer can't be accessed.\n",
            ));
        }
    };
    let entry_clone = entry.clone();
    let merge_combo_box_text_clone = merge_combo_box_text.clone();
    let ok_button_clone = ok_button.clone();
    let abort_button_clone = abort_button.clone();
    let update_button_clone = update_button.clone();
    let text_view_clone = text_view.clone();

    button.connect_clicked(move |button: &Button| {
        let result = merge_button_connect_clicked(
            &entry_clone,
            &text_buffer,
            git_dir.clone(),
            conflicts_clone.clone(),
        );

        if !conflicts_clone.borrow().is_empty() && result.is_ok() {
            update_combo_box(
                &merge_combo_box_text_clone,
                conflicts_clone.borrow().clone(),
            );
            set_abort_button_behavior(&abort_button_clone, button, &git_dir);
            set_done_button_behavior(&ok_button_clone, button, conflicts_clone.borrow().clone());
            set_combo_box_on_changed_behavior(&merge_combo_box_text_clone, &text_view_clone);
            set_update_button_behavior(
                &update_button_clone,
                &merge_combo_box_text_clone,
                &text_view_clone,
            );
            ok_button_clone.set_sensitive(true);
            abort_button_clone.set_sensitive(true);
            update_button_clone.set_sensitive(true);
            button.set_sensitive(false);
            turn_off_buttons_on_button_click(
                &ok_button_clone,
                &ok_button_clone,
                &abort_button_clone,
                &update_button_clone,
                button,
            );
            turn_off_buttons_on_button_click(
                &abort_button_clone,
                &ok_button_clone,
                &abort_button_clone,
                &update_button_clone,
                button,
            );
        }
    });

    let conflict_clone = conflicts.borrow();
    Ok(conflict_clone.clone())
}

/// Shows the current Git branch on a merge window.
///
/// This function retrieves the current Git branch name and displays it in
/// the provided `TextView` within a merge window. The user is prompted to
/// enter the branch they want to merge with the current branch.
///
/// # Arguments
///
/// * `merge_text_view` - The GTK `TextView` where the merge information is displayed.
///
/// # Errors
///
/// Returns an `io::Result` indicating whether the operation was successful
/// or resulted in an error.
///
fn show_current_branch_on_merge_window(merge_text_view: &TextView) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;

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
        &("Actual branch: ".to_string()
            + &current_branch
            + ".\nEnter the branch you want to merge with the current branch..\n"),
    );

    Ok(())
}

/// Handles the "List Modified" button click event.
///
/// This function retrieves the list of modified files using Git and displays
/// them in the provided GTK `TextView`.
///
/// # Arguments
///
/// * `button` - The GTK button that triggers the action when clicked.
/// * `text_view` - The GTK `TextView` where the list of modified files will be displayed.
///
pub fn list_modified_button_on_clicked(button: &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let mut current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("Failed to obtain the current directory.");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let git_dir = match find_git_directory(&mut current_dir, GIT_DIR) {
            Some(dir) => dir,
            None => {
                eprintln!("Failed to obtain the Git directory..");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data.(",
                );

                return;
            }
        };
        let working_dir = match Path::new(&git_dir).parent() {
            Some(dir) => dir.to_string_lossy().to_string(),
            None => {
                eprintln!("Failed to obtain the Git directory.");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data.(",
                );

                return;
            }
        };
        let current_dir = &current_dir.to_string_lossy().to_string();
        let line = vec!["git".to_string(), "ls-files".to_string(), "-m".to_string()];
        let index_path = format!("{}/{}", git_dir, INDEX);
        let gitignore_path = format!("{}/{}", git_dir, GIT_IGNORE);
        let index = match Index::load(&index_path, &git_dir, &gitignore_path) {
            Ok(index) => index,
            Err(_e) => {
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data:(",
                );

                return;
            }
        };
        let mut output: Vec<u8> = vec![];
        let result = git_ls_files(
            &working_dir,
            &git_dir,
            current_dir,
            line,
            &index,
            &mut output,
        );
        if result.is_err() {
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data :(",
            );
            return;
        }
        let buffer = match cloned_text_view.get_buffer() {
            Some(buf) => buf,
            None => {
                eprintln!("Failed to obtain the Git directory");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data :(",
                );
                return;
            }
        };

        let string = match String::from_utf8(output) {
            Ok(str) => str,
            Err(_e) => {
                eprintln!("Failed to convert the result to a string..");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data :(",
                );
                return;
            }
        };
        buffer.set_text(string.as_str());
    });
}

/// Handles the "List Index" button click event.
///
/// This function retrieves the list of files in the Git index and displays
/// them in the provided GTK `TextView`.
///
/// # Arguments
///
/// * `button` - The GTK button that triggers the action when clicked.
/// * `text_view` - The GTK `TextView` where the list of index files will be displayed.
///
pub fn list_index_button_on_clicked(button: &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let mut current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("Failed to obtain the current directory.");
                show_message_dialog(
                    "Fatal error",
                    "An error occurred while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let git_dir = match find_git_directory(&mut current_dir, GIT_DIR) {
            Some(dir) => dir,
            None => {
                eprintln!("Failed to obtain the Git directory..");
                show_message_dialog(
                    "Fatal error",
                    "An error occurred while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let working_dir = match Path::new(&git_dir).parent() {
            Some(dir) => dir.to_string_lossy().to_string(),
            None => {
                eprintln!("Failed to obtain the working directory.");
                show_message_dialog(
                    "Fatal error",
                    "An error occurred while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let current_dir = &current_dir.to_string_lossy().to_string();
        let line = vec!["git".to_string(), "ls-files".to_string()];
        let index_path = format!("{}/{}", git_dir, INDEX);
        let gitignore_path = format!("{}/{}", git_dir, GIT_IGNORE);
        let index = match Index::load(&index_path, &git_dir, &gitignore_path) {
            Ok(index) => index,
            Err(_e) => {
                show_message_dialog(
                    "Fatal error",
                    "An error occurred while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let mut output: Vec<u8> = vec![];
        let result = git_ls_files(
            &working_dir,
            &git_dir,
            current_dir,
            line,
            &index,
            &mut output,
        );
        if result.is_err() {
            show_message_dialog(
                "Fatal error",
                "An error occurred while trying to retrieve the data. :(",
            );
            return;
        }
        let buffer = match cloned_text_view.get_buffer() {
            Some(buf) => buf,
            None => {
                eprintln!("Failed to obtain the text buffer.");
                show_message_dialog(
                    "Fatal error",
                    "An error occurred while trying to retrieve the data. :(",
                );
                return;
            }
        };

        let string = match String::from_utf8(output) {
            Ok(str) => str,
            Err(_e) => {
                eprintln!("Failed to convert the result to a string.");
                show_message_dialog(
                    "Fatal error",
                    "An error occurred while trying to retrieve the data. :(",
                );
                return;
            }
        };
        buffer.set_text(string.as_str());
    });
}

/// Handles the "List Untracked" button click event.
///
/// This function retrieves the list of untracked files using Git and displays
/// them in the provided GTK `TextView`.
///
/// # Arguments
///
/// * `button` - The GTK button that triggers the action when clicked.
/// * `text_view` - The GTK `TextView` where the list of untracked files will be displayed.
///
pub fn list_untracked_button_on_clicked(button: &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let mut current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("Failed to obtain the current directory.");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let git_dir = match find_git_directory(&mut current_dir, GIT_DIR) {
            Some(dir) => dir,
            None => {
                eprintln!("Failed to obtain the Git directory..");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let working_dir = match Path::new(&git_dir).parent() {
            Some(dir) => dir.to_string_lossy().to_string(),
            None => {
                eprintln!("Failed to obtain the working directory..");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let current_dir = &current_dir.to_string_lossy().to_string();
        let line = vec!["git".to_string(), "ls-files".to_string(), "-o".to_string()];
        let index_path = format!("{}/{}", git_dir, INDEX);
        let gitignore_path = format!("{}/{}", git_dir, GIT_IGNORE);
        let index = match Index::load(&index_path, &git_dir, &gitignore_path) {
            Ok(index) => index,
            Err(_e) => {
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let mut output: Vec<u8> = vec![];
        let result = git_ls_files(
            &working_dir,
            &git_dir,
            current_dir,
            line,
            &index,
            &mut output,
        );
        if result.is_err() {
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data. :(",
            );
            return;
        }
        let buffer = match cloned_text_view.get_buffer() {
            Some(buf) => buf,
            None => {
                eprintln!("Failed to obtain the text buffer.");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );
                return;
            }
        };

        let string = match String::from_utf8(output) {
            Ok(str) => str,
            Err(_e) => {
                eprintln!("Failed to convert the result to a string..");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );
                return;
            }
        };
        buffer.set_text(string.as_str());
    });
}

/// Opens a window that lists different types of Git-tracked files.
///
/// This function initializes and displays a GTK window with buttons to list
/// untracked files, files in the Git index, and modified files. The file
/// lists are displayed in a GTK `TextView`.
///
/// # Arguments
///
/// * `builder` - The GTK `Builder` used to construct the window.
///
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

    let scrolled_window: gtk::ScrolledWindow = builder.get_object("scroll-files").unwrap();
    scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scrolled_window.add(&text_view);

    apply_button_style(&list_untracked_button)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    apply_button_style(&list_index_button)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    apply_button_style(&list_modified_button)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    list_untracked_button_on_clicked(&list_untracked_button, &text_view);
    list_index_button_on_clicked(&list_index_button, &text_view);
    list_modified_button_on_clicked(&list_modified_button, &text_view);
    Ok(())
}

/// Checks if a given path is ignored based on the rules specified in a gitignore file.
///
/// This function calls the `git check-ignore` command with the provided path and gitignore file.
/// The result is then displayed in a GTK text view.
///
/// # Arguments
/// * `gitignore_path` - The path to the gitignore file.
/// * `line` - A vector representing the command line for `git check-ignore`.
/// * `cloned_text_view` - A reference to the GTK text view for displaying the command output.
///
fn check_ignore(gitignore_path: &str, line: Vec<String>, cloned_text_view: &TextView) {
    let mut output: Vec<u8> = vec![];
    match git_check_ignore(".mgitignore", gitignore_path, line, &mut output) {
        Ok(_) => {
            let buffer = match cloned_text_view.get_buffer() {
                Some(buf) => buf,
                None => {
                    eprintln!("Failed to obtain the text buffer.");
                    show_message_dialog(
                        "Fatal error",
                        "Something went wrong while trying to retrieve the data. :(",
                    );
                    return;
                }
            };

            let string = match String::from_utf8(output) {
                Ok(str) => str,
                Err(_e) => {
                    eprintln!("Failed to convert the result to a string..");
                    show_message_dialog(
                        "Fatal error",
                        "Something went wrong while trying to retrieve the data. :(",
                    );
                    return;
                }
            };
            buffer.set_text(string.as_str());
        }
        Err(e) => {
            eprintln!("{}", e);
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data. :(",
            );
        }
    }
}

/// Constructs a command line for the `git check-ignore` command based on the state of a switch.
///
/// If the switch is active, the command line includes the "-v" (verbose) option.
///
/// # Arguments
/// * `switch_is_active` - A boolean indicating whether the switch is active.
/// * `path` - The path for which the check-ignore command is being constructed.
///
/// # Returns
/// A vector of strings representing the command line for the `git check-ignore` command.
///
fn get_line_for_check_ignore(switch_is_active: bool, path: String) -> Vec<String> {
    let line: Vec<String> = if switch_is_active {
        vec![
            "git".to_string(),
            "check-ignore".to_string(),
            "-v".to_string(),
            path,
        ]
    } else {
        vec!["git".to_string(), "check-ignore".to_string(), path]
    };
    line
}

/// Handles the "clicked" signal for the ignore button.
///
/// This function is connected to the click event of a GTK button. It obtains the Git directory
/// (assumed to be in a folder named ".mgit"), determines the working directory, and constructs
/// the path to the ".mgitignore" file. It then retrieves the input path from a GTK entry widget,
/// checks if it is empty, and displays an error message if so. Otherwise, it generates a line
/// based on the path and the state of a GTK switch, and checks if this line is ignored in the
/// ".mgitignore" file. The result is displayed in a GTK text view.
///
/// # Arguments
/// * `button` - A reference to the GTK button triggering the event.
/// * `text_view` - A reference to the GTK text view for displaying results.
/// * `entry` - A reference to the GTK entry for obtaining the input path.
/// * `switch` - A reference to the GTK switch indicating whether to check for inclusion or exclusion.
///
pub fn check_ignore_button_on_clicked(
    button: &Button,
    text_view: &gtk::TextView,
    entry: &gtk::Entry,
    switch: &gtk::Switch,
) {
    let cloned_text_view = text_view.clone();
    let cloned_entry = entry.clone();
    let cloned_switch = switch.clone();
    button.connect_clicked(move |_| {
        let git_dir = match obtain_git_dir() {
            Ok(dir) => dir,
            Err(_error) => {
                eprintln!("Failed to obtain the Git directory.");
                return;
            }
        };
        let working_dir = match Path::new(&git_dir).parent() {
            Some(dir) => dir.to_string_lossy().to_string(),
            None => {
                eprintln!("Failed to obtain the working directory.");
                return;
            }
        };

        let gitignore_path = format!("{}/{}", working_dir, GIT_IGNORE);

        let path = cloned_entry.get_text();
        if path.is_empty() {
            show_message_dialog("Error", "You must enter a path.");
        } else {
            let line = get_line_for_check_ignore(cloned_switch.get_active(), path.to_string());

            check_ignore(&gitignore_path, line, &cloned_text_view);
        }
    });
}

/// Sets up and displays the "Check Ignore" window.
///
/// This function initializes and displays a GTK window with UI elements for
/// checking whether a specified path is ignored by Git based on the contents
/// of the `.mgitignore` file. The user can input the path in an entry, and
/// choose whether to display more detailed information using a switch.
/// The result of the check is displayed in a GTK `TextView`.
///
/// # Arguments
///
/// * `builder` - The GTK `Builder` used to construct the window.
///
pub fn check_ignore_window(builder: &Builder) {
    let check_ignore_button = get_button(builder, "check-ignore-button");
    let check_ignore_entry = match get_entry(builder, "check-ignore-entry") {
        Some(entry) => entry,
        None => {
            eprintln!("Failed to obtain the entry..");
            return;
        }
    };
    let check_ignore_view = match get_text_view(builder, "check-ignore-view") {
        Some(view) => view,
        None => {
            eprintln!("Failed to obtain the text view..");
            return;
        }
    };

    let check_ignore_switch = match get_switch(builder, "check-ignore-switch") {
        Some(view) => view,
        None => {
            eprintln!("Failed to obtain the switch..");
            return;
        }
    };

    let scrolled_window: gtk::ScrolledWindow = builder.get_object("scroll-ig").unwrap();

    match apply_button_style(&check_ignore_button) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    apply_entry_style(&check_ignore_entry);

    scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scrolled_window.add(&check_ignore_view);

    check_ignore_button_on_clicked(
        &check_ignore_button,
        &check_ignore_view,
        &check_ignore_entry,
        &check_ignore_switch,
    );
}

/// Applies a custom style to a GTK button.
///
/// This function applies a custom style to a specified GTK button. If successful,
/// the button will be visually updated to reflect the applied style.
///
/// # Arguments
///
/// * `button` - The GTK `Button` to which the style will be applied.
///
pub fn handle_apply_button_style(button: &Button) {
    match apply_button_style(button) {
        Ok(_) => {}
        Err(_e) => {
            eprintln!("Failed to apply the style to the button.");
        }
    }
}

/// Updates a GTK text view with the provided output.
///
/// This function takes a reference to a GTK text view and a vector of bytes representing the
/// output of a command. It attempts to convert the byte vector to a UTF-8 string and sets the
/// content of the text view to the resulting string.
///
/// # Arguments
/// * `cloned_text_view` - A reference to the GTK text view to be updated.
/// * `output` - A vector of bytes representing the output content.
///
fn update_show_ref_view(cloned_text_view: &gtk::TextView, output: Vec<u8>) {
    let buffer = match cloned_text_view.get_buffer() {
        Some(buf) => buf,
        None => {
            eprintln!("Failed to obtain the text buffer.");
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data. :(",
            );
            return;
        }
    };

    let string = match String::from_utf8(output) {
        Ok(str) => str,
        Err(_e) => {
            eprintln!("Failed to convert the result to a string..");
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data. :(",
            );
            return;
        }
    };
    buffer.set_text(string.as_str());
}

/// Handles the "clicked" signal for the show-ref button.
///
/// This function is connected to the click event of a GTK button. It obtains the Git directory
/// (assumed to be in a folder named ".mgit") and constructs the command line for the `git show-ref`
/// command. The result of the command is then displayed in a GTK text view.
///
/// # Arguments
/// * `button` - A reference to the GTK button triggering the event.
/// * `text_view` - A reference to the GTK text view for displaying results.
///
pub fn show_ref_button_on_clicked(button: &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let git_dir = match obtain_git_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("Failed to obtain the Git directory..");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );

                return;
            }
        };

        let line = vec!["git".to_string(), "show-ref".to_string()];

        let mut output: Vec<u8> = vec![];
        let result = git_show_ref(&git_dir, line, &mut output);
        if result.is_err() {
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data. :(",
            );
            return;
        }
        update_show_ref_view(&cloned_text_view, output);
    });
}

/// Handles the click event of the "Show Heads" button.
///
/// This function is connected to the click event of a GTK button. When the button is clicked,
/// it retrieves and displays the references in the Git repository that are heads (branches)
/// using the `git show-ref --heads` command. The output is presented in a GTK `TextView`.
///
/// # Arguments
///
/// * `button` - The GTK `Button` triggering the click event.
/// * `text_view` - The GTK `TextView` where the output will be displayed.
///
pub fn show_heads_button_on_clicked(button: &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let git_dir = match obtain_git_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("Failed to obtain the Git directory..");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--heads".to_string(),
        ];

        let mut output: Vec<u8> = vec![];
        let result = git_show_ref(&git_dir, line, &mut output);
        if result.is_err() {
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data. :(",
            );
            return;
        }
        update_show_ref_view(&cloned_text_view, output);
    });
}

/// Handles the click event of the "Show Tags" button.
///
/// This function is connected to the click event of a GTK button. When the button is clicked,
/// it retrieves and displays the references in the Git repository that are tags
/// using the `git show-ref --tags` command. The output is presented in a GTK `TextView`.
///
/// # Arguments
///
/// * `button` - The GTK `Button` triggering the click event.
/// * `text_view` - The GTK `TextView` where the output will be displayed.
///
pub fn show_tags_button_on_clicked(button: &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let git_dir = match obtain_git_dir() {
            Ok(dir) => dir,
            Err(_) => {
                eprintln!("Git dir not found.");
                return;
            }
        };

        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--tags".to_string(),
        ];

        let mut output: Vec<u8> = vec![];
        let result = git_show_ref(&git_dir, line, &mut output);
        if result.is_err() {
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data. :(",
            );
            return;
        }

        update_show_ref_view(&cloned_text_view, output);
    });
}

/// Handles the click event of the "Show Hash" button.
///
/// This function is connected to the click event of a GTK button. When the button is clicked,
/// it retrieves and displays the references in the Git repository along with their hashes
/// using the `git show-ref --hash` command. The output is presented in a GTK `TextView`.
///
/// # Arguments
///
/// * `button` - The GTK `Button` triggering the click event.
/// * `text_view` - The GTK `TextView` where the output will be displayed.
///
pub fn show_hash_button_on_clicked(button: &Button, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    button.connect_clicked(move |_| {
        let git_dir = match obtain_git_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("Failed to obtain the Git directory..");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let line = vec![
            "git".to_string(),
            "show-ref".to_string(),
            "--hash".to_string(),
        ];

        let mut output: Vec<u8> = vec![];
        let result = git_show_ref(&git_dir, line, &mut output);
        if result.is_err() {
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data. :(",
            );
            return;
        }

        update_show_ref_view(&cloned_text_view, output);
    });
}

/// Handles the click event of the "Verify Ref" button.
///
/// This function is connected to the click event of a GTK button. When the button is clicked,
/// it verifies the reference pointed to by the provided path using the `git show-ref --verify`
/// command. The result is displayed in a GTK `TextView`.
///
/// # Arguments
///
/// * `button` - The GTK `Button` triggering the click event.
/// * `text_view` - The GTK `TextView` where the output will be displayed.
/// * `entry` - The GTK `Entry` containing the path to the reference to be verified.
///
pub fn verify_ref_button_on_clicked(button: &Button, text_view: &gtk::TextView, entry: &Entry) {
    let cloned_text_view = text_view.clone();
    let cloned_entry = entry.clone();
    button.connect_clicked(move |_| {
        let git_dir = match obtain_git_dir() {
            Ok(dir) => dir,
            Err(_e) => {
                eprintln!("Failed to obtain the Git directory..");
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );

                return;
            }
        };
        let path = cloned_entry.get_text();
        if path.is_empty() {
            show_message_dialog("Error", "You must enter a path.");
        } else {
            let line = vec![
                "git".to_string(),
                "show-ref".to_string(),
                "--verify".to_string(),
                path.to_string(),
            ];

            let mut output: Vec<u8> = vec![];
            let result = git_show_ref(&git_dir, line, &mut output);
            if result.is_err() {
                show_message_dialog(
                    "Fatal error",
                    "Something went wrong while trying to retrieve the data. :(",
                );
                return;
            }

            update_show_ref_view(&cloned_text_view, output);
        }
    });
}

/// Sets up the "Show Ref" window with various buttons and their corresponding actions.
///
/// This function initializes the components of the "Show Ref" window, such as text views,
/// buttons, and entry fields. It also connects the buttons to their respective click
/// event handlers to perform specific Git operations and display the results in a GTK `TextView`.
///
/// # Arguments
///
/// * `builder` - The GTK `Builder` containing the UI elements for the "Show Ref" window.
///
pub fn show_ref_window(builder: &Builder) {
    let show_ref_view = match get_text_view(builder, "show-ref-view") {
        Some(view) => view,
        None => {
            eprintln!("Failed to obtain the text view..");
            return;
        }
    };

    let show_ref_scrolled_window: gtk::ScrolledWindow = builder.get_object("scroll-ref").unwrap();
    show_ref_scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    show_ref_scrolled_window.add(&show_ref_view);

    let show_ref_entry = match get_entry(builder, "show-ref-entry") {
        Some(entry) => entry,
        None => {
            eprintln!("Failed to obtain the entry.");
            return;
        }
    };

    apply_entry_style(&show_ref_entry);

    let verify_ref_button = get_button(builder, "verify-ref-button");
    let show_ref_button = get_button(builder, "show-ref-button");
    let show_heads_button = get_button(builder, "show-heads-button");
    let show_tags_button = get_button(builder, "show-tags-button");
    let show_hash_button = get_button(builder, "show-hash-button");

    handle_apply_button_style(&verify_ref_button);
    handle_apply_button_style(&show_ref_button);
    handle_apply_button_style(&show_heads_button);
    handle_apply_button_style(&show_tags_button);
    handle_apply_button_style(&show_hash_button);

    show_ref_button_on_clicked(&show_ref_button, &show_ref_view);
    show_heads_button_on_clicked(&show_heads_button, &show_ref_view);
    show_tags_button_on_clicked(&show_tags_button, &show_ref_view);
    show_hash_button_on_clicked(&show_hash_button, &show_ref_view);
    verify_ref_button_on_clicked(&verify_ref_button, &show_ref_view, &show_ref_entry);
}

/// Calls the `git config set-user-info` command to update the user's name and email in the Git configuration.
///
/// This function obtains the Git directory (assumed to be in a folder named ".mgit") and constructs
/// the command line for the `git config set-user-info` command with the provided name and email.
/// It then executes the command and displays success or error messages accordingly.
///
/// # Arguments
/// * `name` - The new user name to be set in the Git configuration.
/// * `email` - The new user email to be set in the Git configuration.
///
fn call_git_config(name: String, email: String) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(_e) => {
            eprintln!("Failed to obtain the Git directory..");
            show_message_dialog(
                "Fatal error",
                "Something went wrong while trying to retrieve the data. :(",
            );

            return;
        }
    };

    let line = vec![
        "git".to_string(),
        "config".to_string(),
        "set-user-info".to_string(),
        name,
        email,
    ];

    match git_config(&git_dir, line) {
        Ok(_) => {
            show_message_dialog("Success", "Successfully updated information.");
        }
        Err(_e) => {
            show_message_dialog("Error", &_e.to_string());
        }
    }
}

/// Handles the "clicked" signal for the configuration button.
///
/// This function is connected to the click event of a GTK button. It retrieves the user's name
/// and email from the provided GTK entry widgets, checks if both fields are filled, and calls
/// the `call_git_config` function to update the Git configuration accordingly.
///
/// # Arguments
/// * `button` - A reference to the GTK button triggering the event.
/// * `name_entry` - A reference to the GTK entry widget for the user's name.
/// * `email_entry` - A reference to the GTK entry widget for the user's email.
///
fn config_button_on_clicked(
    button: &Button,
    name_entry: &gtk::Entry,
    email_entry: &gtk::Entry,
    builder: &Builder,
) {
    let cloned_name_entry = name_entry.clone();
    let cloned_email_entry = email_entry.clone();
    let builder_clone = builder.clone();
    button.connect_clicked(move |_| {
        let name = cloned_name_entry.get_text().to_string();
        let email = cloned_email_entry.get_text().to_string();

        if name.is_empty() || email.is_empty() {
            show_message_dialog("Warning", "You must fill in both fields to proceed.");
        } else {
            call_git_config(name, email);
        }
        match update_config_window(&builder_clone) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("{:?}", error.to_string());
            }
        }
    });
}

/// Configures the elements of the configuration window and connects relevant signals.
///
/// This function takes a GTK builder and initializes various elements of the configuration window,
/// such as buttons, entries, and labels. It applies styles to these elements and connects the
/// "clicked" signal of the configuration button to the `config_button_on_clicked` function.
///
/// # Arguments
/// * `builder` - A reference to the GTK builder containing the configuration window elements.
///
fn config_window(builder: &gtk::Builder) {
    let config_button = get_button(builder, "config-button");
    let name_entry = match get_entry(builder, "set-user-name-entry") {
        Some(name_entry) => name_entry,
        None => {
            eprintln!("Entry not found");
            return;
        }
    };
    let email_entry = match get_entry(builder, "set-user-email-entry") {
        Some(email_entry) => email_entry,
        None => {
            eprintln!("Entry not found");
            return;
        }
    };
    let name_label = match get_label(builder, "set-user-name-label", 13.0) {
        Some(name_label) => name_label,
        None => {
            eprintln!("Label not found");
            return;
        }
    };
    let email_label = match get_label(builder, "set-user-email-label", 13.0) {
        Some(email_label) => email_label,
        None => {
            eprintln!("Label not found");
            return;
        }
    };
    let config_title = match get_label(builder, "config-title-label", 17.0) {
        Some(name_entry) => name_entry,
        None => {
            eprintln!("Label not found");
            return;
        }
    };

    match apply_button_style(&config_button) {
        Ok(_) => {}
        Err(_e) => {
            eprintln!("Couldn't apply button style");
        }
    }
    apply_entry_style(&name_entry);
    apply_entry_style(&email_entry);
    apply_label_style(&name_label);
    apply_label_style(&email_label);
    apply_label_style(&config_title);

    config_button_on_clicked(&config_button, &name_entry, &email_entry, builder);
}

/// Sets up the functionality of the merge abort button.
/// Will leave the repository as it was before the merge.
fn set_abort_button_behavior(button: &Button, merge_button: &Button, git_dir: &str) {
    merge_button.set_sensitive(true);
    let git_dir = git_dir.to_string();
    button.connect_clicked(move |_| {
        let branch = branch::get_current_branch_path(&git_dir);
        let branch = match branch {
            Ok(branch) => branch,
            Err(_e) => {
                eprintln!("Failed to obtain the name of the current branch..");
                return;
            }
        };
        let branch_name = match branch.split('/').last() {
            Some(name) => name,
            None => {
                eprintln!("Failed to obtain the name of the current branch..");
                return;
            }
        };
        let root_dir = match Path::new(&git_dir).parent() {
            Some(dir) => dir.to_string_lossy().to_string(),
            None => {
                eprintln!("Failed to obtain the current directory..");
                return;
            }
        };
        let git_dir_path = Path::new(&git_dir);
        let result = checkout::checkout_branch(git_dir_path, &root_dir, branch_name);
        if result.is_err() {
            eprintln!("Checkout failed to branch {}.", branch_name);
        }
    });
}

/// Sets up the functionality of the merge button. Will merge the current branch with the one that was selected.
fn set_done_button_behavior(button: &Button, merge_button: &gtk::Button, conflicts: Vec<String>) {
    let git_dir = match obtain_git_dir() {
        Ok(dir) => dir,
        Err(_e) => {
            eprintln!("Failed to obtain the Git directory..");
            return;
        }
    };
    let index_path = format!("{}/{}", git_dir, INDEX);

    let parent_hash = match branch::get_current_branch_commit(&git_dir) {
        Ok(hash) => hash,
        Err(_e) => {
            eprintln!("Failed to obtain the hash of the current commit..");
            return;
        }
    };
    let merge_head_path = format!("{}/MERGE_HEAD", git_dir);
    let mut merge_head_file = match File::open(merge_head_path) {
        Ok(file) => file,
        Err(_e) => {
            eprintln!("Can't open MERGE_HEAD.");
            return;
        }
    };
    let mut parent_hash2 = String::new();
    match merge_head_file.read_to_string(&mut parent_hash2) {
        Ok(_) => {}
        Err(_e) => {
            eprintln!("Can't read MERGE_HEAD.");
            return;
        }
    };
    let merge_button_cloned = merge_button.clone();
    button.connect_clicked(move |_| {
        for conflict in &conflicts {
            let result = add::add(conflict, &index_path, &git_dir, "", None);
            if result.is_err() {
                eprintln!("Can not add {} to index.", conflict);
            }
        }
        let commit_message = "Merge commit".to_string();
        let result =
            commit::new_merge_commit(&git_dir, &commit_message, &parent_hash, &parent_hash2, "");
        println!("{:?}", result);
        merge_button_cloned.set_sensitive(true);
    });
}

/// Updates the box with the paths of the files that have conflicts.
fn update_combo_box(combo_box: &gtk::ComboBoxText, conflicts: Vec<String>) {
    for conflict in &conflicts {
        combo_box.append_text(conflict);
    }
}

/// Sets the behavior for the `gtk::ComboBoxText` when its active item changes.
///
/// This function connects a callback to the `changed` signal of the provided `gtk::ComboBoxText`.
/// When the active item in the combo box changes, the callback reads the content of the file
/// associated with the selected item and sets the text of the provided `gtk::TextView` accordingly.
///
/// # Arguments
///
/// - `combo_box`: A reference to the `gtk::ComboBoxText` for which the behavior is being set.
/// - `text_view`: A reference to the `gtk::TextView` whose content will be updated based on the
///   selected item in the combo box.
///
fn set_combo_box_on_changed_behavior(combo_box: &gtk::ComboBoxText, text_view: &gtk::TextView) {
    let cloned_text_view = text_view.clone();
    let combo_box_cloned = combo_box.clone();
    combo_box.connect_changed(move |_| {
        let conflict_path = match combo_box_cloned.get_active_text() {
            Some(path) => path.to_string(),
            None => return,
        };

        let buff = match cloned_text_view.get_buffer() {
            Some(buff) => buff,
            None => {
                eprintln!("Error getting the buffer");
                return;
            }
        };

        let content = match fs::read_to_string(conflict_path) {
            Ok(content) => content,
            Err(error) => {
                eprintln!("{:?}", error);
                return;
            }
        };

        buff.set_text(&content)
    });
}

/// Sets the behavior for an update button.
///
/// This function connects a callback to the `clicked` signal of the provided `Button`.
/// When the button is clicked, the callback reads the content of the associated `gtk::TextView`
/// and writes it into the file specified by the selected item in the associated `gtk::ComboBoxText`.
///
/// # Arguments
///
/// - `button`: A reference to the `Button` for which the behavior is being set.
/// - `combo_box`: A reference to the `gtk::ComboBoxText` containing file paths.
/// - `merge_text_view`: A reference to the `gtk::TextView` containing the content to be written into a file.
// ///
// fn set_update_button_behavior(
//     button: &Button,
//     combo_box: &gtk::ComboBoxText,
//     merge_text_view: &TextView,
// ) {
//     let cloned_text_view = merge_text_view.clone();
//     let cloned_combo_box = combo_box.clone();
//     button.connect_clicked(move |_| {
//         let text_buffer = match cloned_text_view.get_buffer() {
//             Some(buff) => buff,
//             None => {
//                 eprintln!("No se encontró el buffer");
//                 return;
//             }
//         };

//         let path = match cloned_combo_box.get_active_text() {
//             Some(path) => path.to_string(),
//             None => return,
//         };

//         let content = match text_buffer.get_text(
//             &text_buffer.get_start_iter(),
//             &text_buffer.get_end_iter(),
//             false,
//         ) {
//             Some(content) => content.to_string(),
//             None => return,
//         };

//         let cloned_path = path.clone();
//         let mut file = match File::create(cloned_path) {
//             Ok(file) => file,
//             Err(error) => {
//                 eprintln!("{:?}", error);
//                 return;
//             }
//         };

//         match file.write_all(content.as_bytes()) {
//             Ok(_) => {}
//             Err(error) => {
//                 eprintln!("{:?}", error);
//             }
//         }
//     });
// }

// /// Sets up the functionality of the merge abort button.
// /// Will leave the repository as it was before the merge.
// fn set_abort_button_behavior(button: &Button, merge_button: &Button, git_dir: &str) {
//     merge_button.set_sensitive(true);
//     let git_dir = git_dir.to_string();
//     button.connect_clicked(move |_| {
//         let branch = branch::get_current_branch_path(&git_dir);
//         let branch = match branch {
//             Ok(branch) => branch,
//             Err(_e) => {
//                 eprintln!("No se pudo obtener el nombre de la rama actual.");
//                 return;
//             }
//         };
//         let branch_name = match branch.split('/').last() {
//             Some(name) => name,
//             None => {
//                 eprintln!("No se pudo obtener el nombre de la rama actual.");
//                 return;
//             }
//         };
//         let root_dir = match Path::new(&git_dir).parent() {
//             Some(dir) => dir.to_string_lossy().to_string(),
//             None => {
//                 eprintln!("No se pudo obtener el directorio actual.");
//                 return;
//             }
//         };
//         let git_dir_path = Path::new(&git_dir);
//         let result = checkout::checkout_branch(git_dir_path, &root_dir, branch_name);
//         if result.is_err() {
//             eprintln!("No se pudo hacer checkout de la rama {}.", branch_name);
//         }
//     });
// }

/// Sets up the functionality of the merge button. Will merge the current branch with the one that was selected.
// fn set_done_button_behavior(button: &Button, merge_button: &gtk::Button, conflicts: Vec<String>) {
//     let git_dir = match obtain_git_dir() {
//         Ok(dir) => dir,
//         Err(_e) => {
//             eprintln!("No se pudo obtener el git dir.");
//             return;
//         }
//     };
//     let index_path = format!("{}/{}", git_dir, INDEX);

//     let parent_hash = match branch::get_current_branch_commit(&git_dir) {
//         Ok(hash) => hash,
//         Err(_e) => {
//             eprintln!("No se pudo obtener el hash del commit actual.");
//             return;
//         }
//     };
//     let merge_head_path = format!("{}/MERGE_HEAD", git_dir);
//     let mut merge_head_file = match File::open(merge_head_path) {
//         Ok(file) => file,
//         Err(_e) => {
//             eprintln!("No se pudo abrir el archivo MERGE_HEAD.");
//             return;
//         }
//     };
//     let mut parent_hash2 = String::new();
//     match merge_head_file.read_to_string(&mut parent_hash2) {
//         Ok(_) => {}
//         Err(_e) => {
//             eprintln!("No se pudo leer el archivo MERGE_HEAD.");
//             return;
//         }
//     };
//     let merge_button_cloned = merge_button.clone();
//     button.connect_clicked(move |_| {
//         for conflict in &conflicts {
//             let result = add::add(conflict, &index_path, &git_dir, "", None);
//             if result.is_err() {
//                 eprintln!("No se pudo agregar el archivo {} al index.", conflict);
//             }
//         }
//         let commit_message = "Merge commit".to_string();
//         let result =
//             commit::new_merge_commit(&git_dir, &commit_message, &parent_hash, &parent_hash2, "");
//         println!("{:?}", result);
//         merge_button_cloned.set_sensitive(true);
//     });
// }

/// Updates the box with the paths of the files that have conflicts.
// fn update_combo_box(combo_box: &gtk::ComboBoxText, conflicts: Vec<String>) {
//     for conflict in &conflicts {
//         combo_box.append_text(conflict);
//     }
// }

/// Sets the behavior for the `gtk::ComboBoxText` when its active item changes.
///
/// This function connects a callback to the `changed` signal of the provided `gtk::ComboBoxText`.
/// When the active item in the combo box changes, the callback reads the content of the file
/// associated with the selected item and sets the text of the provided `gtk::TextView` accordingly.
///
/// # Arguments
///
/// - `combo_box`: A reference to the `gtk::ComboBoxText` for which the behavior is being set.
/// - `text_view`: A reference to the `gtk::TextView` whose content will be updated based on the
///   selected item in the combo box.
// ///
// fn set_combo_box_on_changed_behavior(combo_box: &gtk::ComboBoxText, text_view: &gtk::TextView) {
//     let cloned_text_view = text_view.clone();
//     let combo_box_cloned = combo_box.clone();
//     combo_box.connect_changed(move |_| {
//         let conflict_path = match combo_box_cloned.get_active_text() {
//             Some(path) => path.to_string(),
//             None => return,
//         };

//         let buff = match cloned_text_view.get_buffer() {
//             Some(buff) => buff,
//             None => {
//                 eprintln!("Error getting the buffer");
//                 return;
//             }
//         };

//         let content = match fs::read_to_string(conflict_path) {
//             Ok(content) => content,
//             Err(error) => {
//                 eprintln!("{:?}", error);
//                 return;
//             }
//         };

//         buff.set_text(&content)
//     });
// }

/// Sets the behavior for an update button.
///
/// This function connects a callback to the `clicked` signal of the provided `Button`.
/// When the button is clicked, the callback reads the content of the associated `gtk::TextView`
/// and writes it into the file specified by the selected item in the associated `gtk::ComboBoxText`.
///
/// # Arguments
///
/// - `button`: A reference to the `Button` for which the behavior is being set.
/// - `combo_box`: A reference to the `gtk::ComboBoxText` containing file paths.
/// - `merge_text_view`: A reference to the `gtk::TextView` containing the content to be written into a file.
///
fn set_update_button_behavior(
    button: &Button,
    combo_box: &gtk::ComboBoxText,
    merge_text_view: &TextView,
) {
    let cloned_text_view = merge_text_view.clone();
    let cloned_combo_box = combo_box.clone();
    button.connect_clicked(move |_| {
        let text_buffer = match cloned_text_view.get_buffer() {
            Some(buff) => buff,
            None => {
                eprintln!("buffer not found");
                return;
            }
        };

        let path = match cloned_combo_box.get_active_text() {
            Some(path) => path.to_string(),
            None => return,
        };

        let content = match text_buffer.get_text(
            &text_buffer.get_start_iter(),
            &text_buffer.get_end_iter(),
            false,
        ) {
            Some(content) => content.to_string(),
            None => return,
        };

        let cloned_path = path.clone();
        let mut file = match File::create(cloned_path) {
            Ok(file) => file,
            Err(error) => {
                eprintln!("{:?}", error);
                return;
            }
        };

        match file.write_all(content.as_bytes()) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("{:?}", error);
            }
        }
    });
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
    let ok_button = get_button(builder, "merge-ok-button");
    let abort_button = get_button(builder, "merge-abort-button");
    let update_button = get_button(builder, "merge-update-button");
    apply_style_to_button(&merge_button);
    apply_style_to_button(&ok_button);
    apply_style_to_button(&update_button);
    apply_style_to_button(&abort_button);
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
    let merge_combo_box_text = get_combo_box(builder, "merge-paths")?;

    ok_button.set_sensitive(false);
    abort_button.set_sensitive(false);
    update_button.set_sensitive(false);

    show_current_branch_on_merge_window(&merge_text_view)?;
    set_merge_button_behavior(
        &merge_button,
        &merge_input_branch_entry,
        &merge_text_view,
        &ok_button,
        &abort_button,
        &update_button,
        &merge_combo_box_text,
    )?;

    Ok(())
}

/// Applies a custom style to a GTK button.
///
/// This function attempts to apply a custom style to the provided `gtk::Button` instance.
/// If the style application is successful, no action is taken; otherwise, an error message is printed to the standard error.
///
/// # Arguments
///
/// - `button`: A reference to the `gtk::Button` instance to which the style is being applied.
///
fn apply_style_to_button(button: &gtk::Button) {
    match apply_button_style(button) {
        Ok(_) => {}
        Err(_e) => {
            eprintln!("Couldn't apply button style");
        }
    }
}

/// Sets up the functionality of merge
fn rebase_window(builder: &gtk::Builder) -> io::Result<()> {
    let rebase_button = get_button(builder, "make-rebase-button");
    let ok_button = get_button(builder, "rebase-ok-all-button");
    let abort_button = get_button(builder, "abort-rebase-button");
    let update_button = get_button(builder, "rebase-button");

    apply_style_to_button(&rebase_button);
    apply_style_to_button(&ok_button);
    apply_style_to_button(&update_button);
    apply_style_to_button(&abort_button);

    let builder_clone = builder.clone();
    let rebase_button_clone = rebase_button.clone();

    let branch_entry = match get_entry(builder, "rebase-branch-entry") {
        Some(branch) => branch,
        None => {
            eprintln!("Couldn't get rebase branch entry,");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Couldn't get rebase branch entry.\n",
            ));
        }
    };

    rebase_button.connect_clicked(move |_| {
        let their_branch = branch_entry.get_text().to_string();
        if their_branch.is_empty() {
            show_message_dialog("Error", "Specify branch.");
        } else {
            let git_dir = match obtain_git_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    eprintln!("{:?}", error.to_string());
                    return;
                }
            };
            if !is_an_existing_branch(&their_branch, &git_dir) {
                show_message_dialog(
                    "Invalid branch",
                    &format!("{:?} not an existing branch", &their_branch),
                );
            } else {
                let current_branch = match get_branch_name(&git_dir) {
                    Ok(branch) => branch,
                    Err(error) => {
                        eprintln!("{:?}", error);
                        return;
                    }
                };
                let rebase_object =
                    match rebase::start_rebase_gui(&git_dir, &current_branch, &their_branch) {
                        Ok(rebase) => rebase,
                        Err(e) => {
                            eprintln!("Error starting rebase: {}", e);
                            return;
                        }
                    };
                rebase_button_clone.set_sensitive(false);

                match rebase::write_rebase_step_into_gui(&builder_clone, rebase_object, &git_dir) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error writing rebase step into GUI: {}", e);
                    }
                }
            }
        }
    });
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
        Ok(text) => {
            update_text_view(builder, "staged-view", &text)?;
            update_text_view(builder, "changes-to-be-commited-view", &text)?;
        }
        Err(err) => {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Error getting staged text: {}", err),
            ))?;
        }
    }
    Ok(())
}

fn handle_reload_staging_view_button(builder: &gtk::Builder) -> io::Result<()> {
    match set_staging_area_texts(builder) {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Error setting staging area texts: {}", err),
        )),
    }
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

    let git_dir = find_git_directory(&mut current_dir.clone(), GIT_DIR).ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to find git directory",
    ))?;

    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to convert current directory to string",
            ))
        }
    };

    let index_file = format!("{}/{}", git_dir, INDEX);
    let gitignore_path = format!("{}/{}", working_dir, GIT_IGNORE);
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
    let git_dir = obtain_git_dir()?;
    let working_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to convert current directory to string",
            ))
        }
    };
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
    let gitignore_path = format!("{}/{}", working_dir, GIT_IGNORE);
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
    let git_dir_path_result = utils::find_git_directory(&mut current_dir, GIT_DIR);
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
    current_dir.to_str().map(String::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "Failed to convert current directory to string",
        )
    })
}

/// Retrieves the path to the Git directory in the given current directory.
///
/// This function searches for the Git directory starting from the provided `current_dir` path.
/// If the Git directory is found, the absolute path is returned as a `String`.
/// If the Git directory is not found, an error is returned with a description.
///
/// # Arguments
///
/// - `current_dir`: A reference to the current directory path from which the search for the Git directory begins.
///
/// # Returns
///
/// - `Ok(String)`: The absolute path to the Git directory.
/// - `Err(io::Error)`: An error indicating that the Git directory was not found.
///
fn get_git_directory_path(current_dir: &Path) -> io::Result<String> {
    match utils::find_git_directory(&mut current_dir.to_path_buf(), GIT_DIR) {
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
            "Enter commit message.",
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
    match result {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}", e);
        }
    }
    Ok(())
}

/// Perform the commit operation.
fn perform_commit(builder: &gtk::Builder, message: String) -> io::Result<()> {
    let current_dir_str = get_current_dir_string()?;
    let git_dir_path = get_git_directory_path(&PathBuf::from(&current_dir_str))?;
    let git_ignore_path = format!("{}/{}", current_dir_str, GIT_IGNORE);

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

/// Handles the button click event to show visual branches tree in the GUI.
///
/// This function is associated with a button in the GTK application. When the button is clicked,
/// it triggers the visualization of the branches tree in the GUI using the provided `builder`.
///
/// # Arguments
///
/// - `builder`: A reference to the GTK builder containing the GUI components.
///
/// # Returns
///
/// - `Ok(())`: The operation was successful, and the visual branches tree is displayed in the GUI.
/// - `Err(io::Error)`: An error occurred during the visualization process.
///
fn handle_visual_branches_button(builder: &gtk::Builder) -> io::Result<()> {
    visual_branches::handle_show_visual_branches_tree(builder)?;
    Ok(())
}
