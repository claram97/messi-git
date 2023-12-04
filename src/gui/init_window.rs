use crate::configuration::GIT_DIR;
use crate::gui::main_window::add_to_open_windows;
use crate::gui::main_window::close_all_windows;
use crate::gui::main_window::run_main_window;
use crate::gui::repository_window::show_repository_window;
use crate::gui::style::apply_button_style;
use crate::gui::style::apply_window_style;
use crate::gui::style::create_text_entry_window;
use crate::gui::style::get_button;
use crate::init::git_init;
use gtk::prelude::*;
use gtk::ButtonExt;
use gtk::ContainerExt;
use gtk::FileChooserExt;
use gtk::GtkWindowExt;
use gtk::WidgetExt;
use gtk::{Box, Button, FileChooserAction, FileChooserButton, Orientation, Window, WindowType};
use std::env;
use std::io;
use std::path::Path;

/// Configures the properties of a clone window in a GTK application.
///
/// This function takes a reference to a GTK window (`new_window_clone`) and a GTK builder (`builder`) as input and configures the clone window's properties, including adding it to the list of open windows, applying a specific window style, and setting its default size.
///
/// # Arguments
///
/// - `new_window_clone`: A reference to the GTK window to be configured.
/// - `builder`: A reference to the GTK builder used for UI construction.
///
pub fn configure_init_window(
    new_window_init: &gtk::Window,
    builder: &gtk::Builder,
) -> io::Result<()> {
    add_to_open_windows(new_window_init);
    apply_window_style(new_window_init)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply window style"))?;
    new_window_init.set_default_size(800, 600);

    let button1 = get_button(builder, "button1");
    let button2 = get_button(builder, "button2");
    let button3 = get_button(builder, "button3");
    let button4 = get_button(builder, "button4");

    apply_button_style(&button1)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply button1 style"))?;
    apply_button_style(&button2)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply button2 style"))?;
    apply_button_style(&button3)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply button3 style"))?;
    apply_button_style(&button4)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply button4 style"))?;

    connect_button_clicked_init_window(&button1, "option1")?;
    connect_button_clicked_init_window(&button2, "option2")?;
    connect_button_clicked_init_window(&button3, "option3")?;
    connect_button_clicked_init_window(&button4, "option4")?;
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
pub fn connect_button_clicked_init_window(
    button: &gtk::Button,
    button_type: &str,
) -> io::Result<()> {
    let button_type = button_type.to_owned();
    button.connect_clicked(move |_| {
        let current_dir = std::env::current_dir();

        if let Ok(current_dir) = current_dir {
            let dir_str = match current_dir.to_str() {
                Some(str) => str.to_owned(),
                None => {
                    eprintln!("Failed to convert current directory to string");
                    return;
                }
            };
            if button_type == "option2" {
                let _result = show_text_entry_window("Enter the branch", move |text| {
                    if let Err(err) = handle_git_init_and_change_dir(&dir_str, &text, &current_dir)
                    {
                        eprintln!("{}", err);
                    }
                });
            } else if button_type == "option3" {
                let current_dir_clone = &current_dir.clone();
                handle_template_path_entry(&dir_str, current_dir_clone);
            } else if button_type == "option1" {
                init_git_and_handle_errors(&dir_str, &current_dir);
            } else if button_type == "option4" {
                let (window, button_ok, file_chooser, vbox) = create_selection_window();
                button_ok.connect_clicked(move |_| {
                    handle_directory_selection(&file_chooser, &current_dir);
                });
                window.add(&vbox);
                window.show_all();
            }
        } else {
            eprintln!("failed to obtain actual directory");
        }
    });
    Ok(())
}

/// Handle the selection of a directory using a file chooser button.
///
/// This function takes a GTK file chooser button and the current directory as parameters.
/// It retrieves the selected directory from the file chooser, initializes a Git repository,
/// and handles the result by changing the current directory or printing an error message.
///
/// # Arguments
///
/// * `file_chooser` - A GTK file chooser button for selecting directories.
/// * `current_dir` - The current directory as a `Path`.
///
fn handle_directory_selection(file_chooser: &FileChooserButton, current_dir: &Path) {
    if let Some(selected_directory) = file_chooser.get_filename() {
        let result = git_init(
            &selected_directory.to_string_lossy(),
            GIT_DIR,
            "master",
            None,
        );
        if result.is_err() {
            eprintln!("Error in git init .");
            return;
        }

        let result = handle_git_init_result(result, current_dir, Path::new(&selected_directory));
        if result.is_err() {
            eprintln!("Error handling git init with template");
        }
    }
}

/// Create a selection window with a file chooser and an OK button.
///
/// This function creates a GTK window containing a file chooser button and an OK button.
/// The file chooser is set to select directories, and the window is styled with a vertical box.
/// The selected directory is printed to the console when chosen.
///
/// # Returns
///
fn create_selection_window() -> (Window, Button, FileChooserButton, Box) {
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Selection Directory");
    window.set_default_size(400, 150);
    add_to_open_windows(&window);
    let file_chooser =
        FileChooserButton::new("Select a directory ", FileChooserAction::SelectFolder);

    let button_ok = Button::with_label("OK");
    if let Err(err) = apply_button_style(&button_ok) {
        eprintln!("Couldn't apply button style: {:?}", err);
    }

    let vbox = Box::new(Orientation::Vertical, 5);
    vbox.add(&file_chooser);
    vbox.add(&button_ok);

    let file_chooser_clone = file_chooser.clone();
    file_chooser.connect_file_set(move |_| {
        if let Some(directory) = file_chooser_clone.get_filename() {
            println!("Selected directory: {:?}", directory.to_string_lossy());
        }
    });
    window.add(&vbox);

    (window, button_ok, file_chooser, vbox)
}

/// Initialize a Git repository and handle potential errors.
///
/// This function initializes a Git repository in the specified directory with the default branch
/// set to "master". It also handles errors that may occur during the initialization process.
///
/// # Arguments
///
/// * `dir_str` - A string representing the directory in which the Git repository will be initialized.
/// * `current_dir` - The current directory path.
///
fn init_git_and_handle_errors(dir_str: &str, current_dir: &Path) {
    let result = git_init(dir_str, GIT_DIR, "master", None);
    if result.is_err() {
        eprintln!("Error initiating git.");
        return;
    }

    let result = handle_git_init_result(result, current_dir, Path::new(&dir_str));
    if result.is_err() {
        eprintln!("Error handling git init result");
    }
}

/// Handle the entry of a template path.
///
/// This function prompts the user to enter a template path using a text entry window. The entered
/// path is then used to initialize a Git repository with the specified template.
///
/// # Arguments
///
/// * `dir_str` - A string representing the directory in which the Git repository will be initialized.
/// * `current_dir` - The current directory path.
///
fn handle_template_path_entry(dir_str: &str, current_dir: &Path) {
    let dir_str = dir_str.to_string();
    let current_dir_clone = current_dir.to_path_buf();

    let result = create_text_entry_window("Enter the template path", move |text| {
        let result = git_init(&dir_str, GIT_DIR, "master", Some(&text));
        if result.is_err() {
            eprintln!("Error initiating git.");
            return;
        }

        let result = handle_git_init_result(result, &current_dir_clone, Path::new(&dir_str));
        if result.is_err() {
            eprintln!("Error handling git init with template");
        }
    });

    if result.is_err() {
        eprintln!("Error creating text entry window.");
    }
}

/// Show a text entry window.
///
/// This function displays a window with a text entry field and an "OK" button. The provided callback
/// function is invoked when the "OK" button is clicked, passing the entered text as an argument.
///
/// # Arguments
///
/// * `prompt` - A prompt message displayed in the text entry window.
/// * `callback` - A callback function that takes a `String` as input. This function is invoked
///               when the user clicks the "OK" button, passing the entered text to the callback.
///
/// # Errors
///
/// Returns a `Result` indicating whether the operation was successful or if an error occurred.
/// In case of an error, a `String` with a descriptive error message is returned.
///
fn show_text_entry_window<'a, F>(prompt: &str, callback: F) -> Result<(), String>
where
    F: Fn(String) + 'a + 'static,
{
    let result = create_text_entry_window(prompt, callback);
    if result.is_err() {
        return Err("Error creating text entry window.".to_string());
    }
    Ok(())
}

/// Initialize a Git repository and change the current directory.
///
/// This function performs the following steps:
/// 1. Initializes a Git repository in the specified directory (`dir_str`) with the specified branch (`branch_name`).
/// 2. Handles any errors resulting from the Git initialization.
/// 3. Changes the current directory to the newly created directory.
///
/// # Arguments
///
/// * `dir_str` - Path of the directory where the Git repository will be initialized.
/// * `branch_name` - Name of the branch to create during Git initialization.
/// * `current_dir` - Current directory before Git initialization.
///
/// # Errors
///
/// The function returns a `Result` indicating whether the operation was successful or if an error occurred.
/// In case of an error, a `String` with a descriptive error message is returned.
///
fn handle_git_init_and_change_dir(
    dir_str: &str,
    branch_name: &str,
    current_dir: &Path,
) -> Result<(), String> {
    let result = git_init(dir_str, GIT_DIR, branch_name, None);
    if result.is_err() {
        return Err("Error initiating git.".to_string());
    }

    let result = handle_git_init_result(result, current_dir, Path::new(dir_str));
    if result.is_err() {
        return Err("Error handling git init result.".to_string());
    }

    if env::set_current_dir(dir_str).is_err() {
        return Err("Error changing current directory.".to_string());
    }

    Ok(())
}

/// Handles the result of a Git initialization operation and performs window management.
///
/// This function takes the directory path `dir_str` and the result of a Git initialization operation
/// as input and manages the opening and closing of windows based on the result.
///
/// If the Git initialization is successful, it closes all windows and shows the repository window.
/// If there's an error, it closes all windows and shows the main window.
///
/// # Arguments
///
/// - `dir_str`: A string representing the directory path.
/// - `result`: A `Result` containing the outcome of the Git initialization operation.
///
/// # Returns
///
/// A `Result` with an empty `Ok(())` value to indicate success.
pub fn handle_git_init_result(
    result: Result<(), io::Error>,
    code_dir: &Path,
    work_dir: &Path,
) -> Result<(), io::Error> {
    match result {
        Ok(_) => {
            close_all_windows();
            let result = show_repository_window(code_dir, work_dir);

            if result.is_err() {
                eprintln!("Couldn't show repository window");
            }
        }
        Err(_err) => {
            close_all_windows();
            let result = run_main_window();
            if result.is_err() {
                eprintln!("Couldn't show main window");
            }
        }
    }

    Ok(())
}
