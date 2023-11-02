use crate::add::add;
use crate::branch;
use crate::branch::git_branch_for_ui;
use crate::commit;
use crate::gui::style::{apply_button_style, apply_window_style, get_button, load_and_get_window};
use crate::index;
use crate::init::git_init;
use crate::log::log;
use crate::log::Log;
use crate::status;
use crate::tree_handler;
use crate::utils;
use crate::utils::find_git_directory;
use gtk::prelude::*;
use gtk::Builder;
use gtk::FileChooserAction;
use gtk::FileChooserDialog;
use std::io;
use std::path::Path;
use std::sync::Mutex;

use super::style::apply_clone_button_style;
use super::style::apply_entry_style;
use super::style::apply_label_style;
use super::style::get_entry;
use super::style::get_label;

pub static mut OPEN_WINDOWS: Option<Mutex<Vec<gtk::Window>>> = None;

/// Runs the main window of a GTK application.
///
/// This function initializes and displays the main window of the application using a UI builder. It configures the window, adds buttons for actions such as "Clone" and "Init," and connects these buttons to their respective event handlers.
///
pub fn run_main_window() -> io::Result<()> {
    unsafe {
        OPEN_WINDOWS = Some(Mutex::new(Vec::new()));
    }

    let builder = Builder::new();
    if let Some(window) = load_and_get_window(&builder, "src/gui/part3.ui", "window") {
        window.set_default_size(800, 600);
        add_to_open_windows(&window);
        apply_window_style(&window).map_err(|_err| {
            io::Error::new(io::ErrorKind::Other, "Error applying window stlye.\n")
        })?;

        let button_clone: gtk::Button = get_button(&builder, "buttonclone");
        let button_init: gtk::Button = get_button(&builder, "buttoninit");
        apply_button_style(&button_clone).map_err(|_err| {
            io::Error::new(io::ErrorKind::Other, "Error applying button stlye.\n")
        })?;
        apply_button_style(&button_init).map_err(|_err| {
            io::Error::new(io::ErrorKind::Other, "Error applying button stlye.\n")
        })?;

        connect_button_clicked_main_window(&button_clone, "Clone")?;
        connect_button_clicked_main_window(&button_init, "Init")?;

        window.show_all();
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to run main window.",
        ))
    }
}

/// Closes all open GTK windows in a GTK application.
///
/// This function iterates through the list of open windows maintained by the application and closes each window. It ensures that all open windows are properly closed and their references are removed from the list.
///
pub fn close_all_windows() {
    unsafe {
        if let Some(ref mutex) = OPEN_WINDOWS {
            let mut open_windows = mutex.lock().expect("Mutex lock failed");
            for window in open_windows.iter() {
                window.close();
            }
            open_windows.clear();
        }
    }
}

/// Adds a GTK window to the list of open windows in a GTK application.
///
/// This function takes a reference to a GTK window (`window`) and adds it to the list of open windows maintained by the application. The list of open windows is managed using a mutex to ensure thread safety.
///
/// # Arguments
///
/// - `window`: A reference to the GTK window to be added to the list of open windows.
///
fn add_to_open_windows(window: &gtk::Window) {
    unsafe {
        if let Some(ref mutex) = OPEN_WINDOWS {
            let mut open_windows = mutex.lock().expect("Mutex lock failed");
            open_windows.push(window.clone());
        }
    }
}

/// Remove ANSI color codes from a given string.
///
/// This function takes a string containing ANSI color codes and removes them, resulting in a
/// plain text string without color formatting.
///
/// # Arguments
///
/// * `input` - A reference to the input string containing ANSI color codes.
///
/// # Returns
///
/// A new string with ANSI color codes removed.
///
fn filter_color_code(input: &str) -> String {
    let mut result = String::new();
    let mut in_escape_code = false;

    for char in input.chars() {
        if char == '\u{001b}' {
            in_escape_code = true;
        } else if in_escape_code {
            if char == 'm' {
                in_escape_code = false;
            }
        } else {
            result.push(char);
        }
    }

    result
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
fn obtain_text_from_log() -> Result<String, std::io::Error> {
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

/// Displays a repository window with various buttons and actions in a GTK application.
///
/// This function initializes and displays a GTK repository window using a UI builder. It configures the window, adds buttons with specific actions, and sets their styles and click event handlers. The repository window provides buttons for actions like "Add," "Commit," "Push," and more.
///
fn show_repository_window() -> io::Result<()> {
    let builder = gtk::Builder::new();
    if let Some(new_window) = load_and_get_window(&builder, "src/gui/new_window2.ui", "window") {
        let _new_window_clone = new_window.clone();
        let builder_clone = builder.clone();
        let builder_clone1 = builder.clone();
        set_staging_area_texts(&builder_clone)?;
        set_commit_history_view(&builder_clone1)?;

        add_to_open_windows(&new_window);
        configure_repository_window(new_window)?;
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
                        eprintln!("Fail");
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
                    let resultado = obtain_text_from_add(&text);
                    match resultado {
                        Ok(texto) => {
                            println!("Texto: {}", texto);
                        }
                        Err(err) => {
                            eprintln!("Error al obtener el texto: {}", err);
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
            println!("Button 5 clicked.");
        });

        remove_all_button.connect_clicked(move |_| {
            println!("Button 6 clicked.");
        });

        commit_changes_button.connect_clicked(move |_| {
            let _ = make_commit(&builder_clone2);
            println!("Commit button clicked");
        });
        Ok(())
        // button7.connect_clicked(move |_| {
        //     button9.set_visible(true);
        //     create_branch_button.set_visible(true);
        //     let builder_clone = builder.clone();

        //     button9.connect_clicked(move |_| {
        //         let label: Label = builder_clone.get_object("label").unwrap();
        //         let texto_desde_funcion = git_branch_for_ui(None);
        //         match texto_desde_funcion {
        //             Ok(texto) => {
        //                 label.set_text(&texto);
        //             }
        //             Err(err) => {
        //                 eprintln!("Error al obtener el texto: {}", err);
        //             }
        //         }
        //     });
        // });

        // button8.connect_clicked(move |_| {
        //     new_window_clone.close();
        //     run_main_window();
        // });

        // button11.connect_clicked(move |_| {
        //     let label: Label = builder_clone1.get_object("label").unwrap();
        //     create_text_entry_window("Enter the path of the file", move |text| {
        //         let resultado = obtain_text_from_add(&text);

        //         match resultado {
        //             Ok(texto) => {
        //                 label.set_text(&texto);
        //             }
        //             Err(err) => {
        //                 eprintln!("Error al obtener el texto: {}", err);
        //             }
        //         }
        //     });
        // });
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to show repository window.",
        ))
    }
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
fn set_staging_area_texts(builder: &gtk::Builder) -> io::Result<()> {
    let staging_area_text_view: gtk::TextView = builder.get_object("not-staged-view").ok_or(
        io::Error::new(io::ErrorKind::Other, "Failed to get not-staged-view object"),
    )?;
    let buffer = staging_area_text_view.get_buffer().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to get buffer for not-staged-view",
    ))?;

    let current_dir =
        std::env::current_dir().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let _binding = current_dir.clone();
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
fn format_branch_history(history_vec: Vec<(String, String)>) -> String {
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
fn set_commit_history_view(builder: &gtk::Builder) -> io::Result<()> {
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
                "Git directory not found",
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
fn make_commit(builder: &gtk::Builder) -> io::Result<()> {
    let mut current_dir = std::env::current_dir()?;
    let binding = current_dir.clone();
    let current_dir_str = match binding.to_str() {
        Some(str) => str.to_owned(), // Asignar el valor
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to convert current directory to string",
            ))
        }
    };

    let git_dir_path = match utils::find_git_directory(&mut current_dir, ".mgit") {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Git directory not found",
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
                    "Failed to get commit message text view",
                )
            })?;

    let message = message_view.get_text().to_string();

    let result = commit::new_commit(&git_dir_path, &message, &git_ignore_path);
    println!("{:?}", result);
    set_commit_history_view(builder)?;
    Ok(())
}

/// Stage changes for Git commit in a GTK+ application.
///
/// This function stages changes for a Git commit by adding specified files or all changes in the
/// working directory. It retrieves the current working directory, Git directory, and Git ignore file
/// path. Depending on the provided `texto`, it stages specific files or all changes for the commit.
///
/// # Arguments
///
/// * `texto` - A string representing the files to be staged. Use `"."` to stage all changes.
///
/// # Returns
///
/// - `Ok("Ok".to_string())`: If the changes are successfully staged.
/// - `Err(std::io::Error)`: If an error occurs during the process, it returns an `std::io::Error`.
///
fn obtain_text_from_add(texto: &str) -> Result<String, io::Error> {
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
    let index_path = format!("{}/{}", git_dir, "index");
    let git_dir_parent = match Path::new(&git_dir).parent() {
        Some(git_dir_parent) => git_dir_parent,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Gitignore filey not found\n",
            ))
        }
    };
    let git_ignore_path = format!("{}/{}", git_dir_parent.to_string_lossy(), ".mgitignore");

    if texto == "." {
        let options = Some(vec!["-u".to_string()]);
        match add("", &index_path, &git_dir, &git_ignore_path, options) {
            Ok(_) => {
                println!("La función 'add' se ejecutó correctamente.");
            }
            Err(err) => {
                eprintln!("Error al llamar a la función 'add': {:?}", err);
            }
        };
    }

    match add(texto, &index_path, &git_dir, &git_ignore_path, None) {
        Ok(_) => {
            println!("La función 'add' se ejecutó correctamente.");
        }
        Err(err) => {
            eprintln!("Error al llamar a la función 'add': {:?}", err);
        }
    };
    Ok("Ok".to_string())
}

/// Configures the properties of a repository window in a GTK application.
///
/// This function takes a GTK window (`new_window`) as input and configures the repository window's properties, such as setting its default size and applying a specific window style, before displaying it.
///
/// # Arguments
///
/// - `new_window`: The GTK window to be configured as a repository window.
///
fn configure_repository_window(new_window: gtk::Window) -> io::Result<()> {
    new_window.set_default_size(800, 600);
    apply_window_style(&new_window)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply window style"))?;
    new_window.show_all();
    Ok(())
}

/// Creates a GTK text entry window for user input with a message and a callback function.
///
/// This function generates a new GTK window with a text entry field and an "OK" button. It allows users to input text and invokes a provided callback function when the "OK" button is clicked. The window can display a custom message as its title.
///
/// # Arguments
///
/// - `message`: A string message to be displayed as the window's title.
/// - `on_text_entered`: A callback function that takes a string parameter and is called when the user confirms the text input.
///
fn create_text_entry_window(
    message: &str,
    on_text_entered: impl Fn(String) + 'static,
) -> io::Result<()> {
    let entry_window = gtk::Window::new(gtk::WindowType::Toplevel);
    add_to_open_windows(&entry_window);
    apply_window_style(&entry_window)
        .map_err(|_err| io::Error::new(io::ErrorKind::Other, "Error applying window stlye.\n"))?;
    entry_window.set_title(message);
    entry_window.set_default_size(400, 150);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    entry_window.add(&main_box);

    let entry = gtk::Entry::new();
    entry.set_text("Default Text");
    main_box.add(&entry);

    let ok_button = gtk::Button::with_label("OK");
    apply_button_style(&ok_button)
        .map_err(|_err| io::Error::new(io::ErrorKind::Other, "Error applying button stlye.\n"))?;
    main_box.add(&ok_button);

    ok_button.connect_clicked(move |_| {
        let text = entry.get_text().to_string();
        // close_all_windows();
        // show_repository_window();
        on_text_entered(text);
    });

    entry_window.show_all();
    Ok(())
}

/// Connects a GTK button in an initialization window to specific actions based on its type.
///
/// This function takes a reference to a GTK button (`button`) and a button type (`button_type`) as input and connects a click event handler. The handler performs different actions based on the button's type, such as opening text entry dialogs, closing all windows, or showing a repository window.
///
/// # Arguments
///
/// - `button`: A reference to the GTK button to which the event handler will be connected.
/// - `button_type`: A string indicating the type of button, which determines the action to be taken when the button is clicked.
fn connect_button_clicked_init_window(button: &gtk::Button, button_type: &str) -> io::Result<()> {
    let button_type = button_type.to_owned();

    button.connect_clicked(move |_| {
        let current_dir = std::env::current_dir();

        if let Ok(current_dir) = current_dir {
            let dir_str = match current_dir.to_str() {
                Some(str) => str.to_owned(), // Asignar el valor
                None => {
                    eprintln!("Failed to convert current directory to string");
                    return;
                }
            };

            if button_type == "option2" {
                let result = create_text_entry_window("Enter the branch", move |text| {
                    let result = git_init(&dir_str, &text, None);
                    match result {
                        Ok(_) => {
                            close_all_windows();
                            let result = show_repository_window();
                            if result.is_err() {
                                eprintln!("Couldn't show repository window");
                            }
                        }
                        Err(_err) => {
                            close_all_windows();
                            let result = run_main_window();
                            if result.is_err() {
                                eprintln!("Couldn't show repository window");
                            }
                        }
                    }
                });
                if result.is_err() {}
            } else if button_type == "option3" {
                let result_create =
                    create_text_entry_window("Enter the template path", move |text| {
                        let result = git_init(&dir_str, "main", Some(&text));
                        match result {
                            Ok(_) => {
                                close_all_windows();
                                let result = show_repository_window();
                                if result.is_err() {
                                    eprintln!("Couldn't show repository window");
                                }
                            }
                            Err(_err) => {
                                close_all_windows();
                                let result = run_main_window();
                                if result.is_err() {
                                    eprintln!("Couldn't show repository window");
                                }
                            }
                        }
                    });
                if result_create.is_err() {
                    eprintln!("Error trying to create text entry window.\n");
                    return;
                }
            } else if button_type == "option1" {
                let result = git_init(&dir_str, "main", None);
                match result {
                    Ok(_) => {
                        close_all_windows();
                        let result = show_repository_window();
                        if result.is_err() {
                            eprintln!("Couldn't show repository window");
                            return;
                        }
                    }
                    Err(_err) => {
                        close_all_windows();
                        let result = run_main_window();
                        if result.is_err() {
                            eprintln!("Couldn't show repository window");
                            return;
                        }
                    }
                }
            }
        } else {
            eprintln!("No se pudo obtener el directorio actual.");
        }
    });
    Ok(())
}

/// Configures the properties of a clone window in a GTK application.
///
/// This function takes a reference to a GTK window (`new_window_clone`) and a GTK builder (`builder`) as input and configures the clone window's properties, including adding it to the list of open windows, applying a specific window style, and setting its default size.
///
/// # Arguments
///
/// - `new_window_clone`: A reference to the GTK window to be configured.
/// - `builder`: A reference to the GTK builder used for UI construction.
///
fn configure_init_window(new_window_init: &gtk::Window, builder: &gtk::Builder) -> io::Result<()> {
    add_to_open_windows(new_window_init);
    apply_window_style(new_window_init)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply window style"))?;
    new_window_init.set_default_size(800, 600);

    let button1 = get_button(builder, "button1");
    let button2 = get_button(builder, "button2");
    let button3 = get_button(builder, "button3");

    apply_button_style(&button1)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply button1 style"))?;
    apply_button_style(&button2)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply button2 style"))?;
    apply_button_style(&button3)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply button3 style"))?;
    connect_button_clicked_init_window(&button1, "option1")?;
    connect_button_clicked_init_window(&button2, "option2")?;
    connect_button_clicked_init_window(&button3, "option3")?;
    Ok(())
}

/// Configures the properties of a clone window in a GTK application.
///
/// This function takes a reference to a GTK window (`new_window_clone`) and a GTK builder (`builder`) as input and configures the clone window's properties, including adding it to the list of open windows, applying a specific window style, and setting its default size.
///
/// # Arguments
///
/// - `new_window_clone`: A reference to the GTK window to be configured.
/// - `builder`: A reference to the GTK builder used for UI construction.
///
fn configure_clone_window(
    new_window_clone: &gtk::Window,
    builder: &gtk::Builder,
) -> io::Result<()> {
    add_to_open_windows(new_window_clone);
    let result = apply_window_style(new_window_clone);
    if result.is_err() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Interrupted,
            "Fatal error.\n",
        ));
    }

    let url_entry = match get_entry(builder, "url-entry") {
        Some(entry) => entry,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Entry not found: url-entry\n",
            ))
        }
    };

    apply_entry_style(&url_entry);
    let dir_to_clone_entry = match get_entry(builder, "dir-to-clone-entry") {
        Some(entry) => entry,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Entry not found.\n",
            ))
        }
    };
    let dir_to_clone_entry_clone = dir_to_clone_entry.clone();

    apply_entry_style(&dir_to_clone_entry);

    let browse_button = get_button(builder, "browse-button");
    let clone_button = get_button(builder, "clone-button");

    let new_window_clone_clone = new_window_clone.clone();
    clone_button.connect_clicked(move |_| {
        let url_text = url_entry.get_text().to_string();
        let dir_text = dir_to_clone_entry.get_text().to_string();

        if url_text.is_empty() || dir_text.is_empty() {
            let error_dialog = gtk::MessageDialog::new(
                Some(&new_window_clone_clone),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Error,
                gtk::ButtonsType::Ok,
                "Faltan datos: URL o directorio de clonación.",
            );
            error_dialog.run();
            error_dialog.close();
        } else {
            println!("Ok!");
            // Si ambos campos tienen datos, llama a la función de clonación
            // (asume que ya tienes una función llamada clone_repository)
            // if let Err(err) = clone_repository(&url_text, &dir_text) {
            //     // Si hubo un error al clonar, muestra un mensaje de error
            //     let error_dialog = gtk::MessageDialog::new(
            //         Some(new_window_clone_clone),
            //         gtk::DialogFlags::MODAL,
            //         gtk::MessageType::Error,
            //         gtk::ButtonsType::Ok,
            //         &format!("Error al clonar el repositorio: {}", err));
            //     error_dialog.run();
            //     error_dialog.close();
            // }
        }
    });

    apply_clone_button_style(&browse_button);
    apply_clone_button_style(&clone_button);

    let new_window_clone_clone = new_window_clone.clone();
    browse_button.connect_clicked(move |_| {
        let dialog: FileChooserDialog = FileChooserDialog::new(
            Some("Seleccionar Carpeta"),
            Some(&new_window_clone_clone),
            FileChooserAction::SelectFolder,
        );

        dialog.set_position(gtk::WindowPosition::CenterOnParent);

        dialog.add_button("Cancelar", gtk::ResponseType::Cancel);
        dialog.add_button("Seleccionar", gtk::ResponseType::Ok);

        if dialog.run() == gtk::ResponseType::Ok {
            if let Some(folder) = dialog.get_filename() {
                dir_to_clone_entry_clone.set_text(&folder.to_string_lossy());
            }
        }

        dialog.close();
    });
    let url_label = match get_label(builder, "url-label", 14.0) {
        Some(label) => label,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Label not found: url-label\n",
            ))
        }
    };

    let clone_dir_label = match get_label(builder, "clone-dir-label", 14.0) {
        Some(label) => label,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Label not found: clone-dir-label\n",
            ))
        }
    };

    let clone_info_label = match get_label(builder, "clone-info-label", 26.0) {
        Some(label) => label,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Label not found: clone-info-label\n",
            ))
        }
    };

    apply_label_style(&url_label);
    apply_label_style(&clone_dir_label);
    apply_label_style(&clone_info_label);

    new_window_clone.set_default_size(800, 600);
    Ok(())
}

/// Connects a GTK button to a specific action.
///
/// This function takes a GTK button and a button type as input and sets an event handler for the "clicked" event of the button.
/// When the button is clicked, it performs a specific action based on the provided button type.
///
/// # Arguments
///
/// - `button`: A reference to the GTK button to which the event handler will be connected.
/// - `button_type`: A string indicating the button type, which determines the action to be taken when the button is clicked.
///
fn connect_button_clicked_main_window(button: &gtk::Button, button_type: &str) -> io::Result<()> {
    let button_type = button_type.to_owned();

    button.connect_clicked(move |_| {
        let builder = gtk::Builder::new();
        match button_type.as_str() {
            "Init" => {
                if let Some(new_window_init) =
                    load_and_get_window(&builder, "src/gui/windowInit.ui", "window")
                {
                    let init_window_result = configure_init_window(&new_window_init, &builder);
                    if init_window_result.is_err() {
                        eprintln!("Error initializing init window.\n");
                        return;
                    }
                    new_window_init.show_all();
                }
            }
            "Clone" => {
                if let Some(new_window_clone) =
                    load_and_get_window(&builder, "src/gui/windowClone.ui", "window")
                {
                    let clone_window_result = configure_clone_window(&new_window_clone, &builder);
                    if clone_window_result.is_err() {
                        eprintln!("Error initializing clone window.\n");
                        return;
                    }
                    new_window_clone.show_all();
                }
            }
            _ => eprintln!("Unknown button type: {}", button_type),
        }
    });
    Ok(())
}
