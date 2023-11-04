use crate::add::add;
use crate::branch;
use crate::branch::git_branch_for_ui;
use crate::checkout::checkout_branch;
use crate::checkout::checkout_commit_detached;
use crate::checkout::create_and_checkout_branch;
use crate::checkout::create_or_reset_branch;
use crate::checkout::force_checkout;
use crate::commit;
use crate::gui::style::filter_color_code;
use crate::gui::style::{apply_button_style, apply_window_style, get_button, load_and_get_window};
use crate::index;
use crate::init::git_init;
use crate::log::log;
use crate::log::Log;
use crate::merge;
use crate::rm::git_rm;
use crate::status;
use crate::tree_handler;
use crate::utils;
use crate::utils::find_git_directory;
use gtk::prelude::*;
use gtk::Builder;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use crate::gui::style::create_text_entry_window;
use crate::gui::style::show_message_dialog;
use crate::gui::style::configure_repository_window;

use super::clone_window::configure_clone_window;
use super::init_window::configure_init_window;
use super::style::get_entry;
use super::style::get_text_view;

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
pub fn add_to_open_windows(window: &gtk::Window) {
    unsafe {
        if let Some(ref mutex) = OPEN_WINDOWS {
            let mut open_windows = mutex.lock().expect("Mutex lock failed");
            open_windows.push(window.clone());
        }
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

pub fn obtain_text_from_remove(texto: &str) -> Result<String, io::Error> {
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
    println!("INDEX PATH {}.", index_path);

    let result = match git_rm(texto, &index_path, &git_dir, &git_ignore_path) {
        Ok(_) => Ok("La función 'rm' se ejecutó correctamente.".to_string()),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Error al llamar a la función 'rm': {:?}\n", err),
        )),
    };
    if result.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Error al llamar a la función 'rm'\n",
        ));
    }
    Ok("Ok".to_string())
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
pub fn obtain_text_from_add(texto: &str) -> Result<String, io::Error> {
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
                "Gitignore file not found\n",
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
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Error al llamar a la función 'add': {:?}\n", err),
                ))
            }
        };
    }

    match add(texto, &index_path, &git_dir, &git_ignore_path, None) {
        Ok(_) => {
            println!("La función 'add' se ejecutó correctamente.");
        }
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Error al llamar a la función 'add': {:?}\n", err),
            ))
        }
    };
    Ok("Ok".to_string())
}


