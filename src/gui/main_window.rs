use super::clone_window::configure_clone_window;
use super::init_window::configure_init_window;
use crate::configuration::GIT_DIR;
use crate::gui::repository_window::show_repository_window;
use crate::gui::style::{
    apply_button_style, apply_window_style, get_button, load_and_get_window, show_message_dialog,
};
use gtk::prelude::*;
use gtk::Builder;
use gtk::{self, FileChooserAction, FileChooserDialog, FileChooserExt};
use std::io;
use std::path::Path;
use std::sync::Mutex;

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
        let button_open_repo: gtk::Button = get_button(&builder, "button-open-repo");
        apply_button_style(&button_clone).map_err(|_err| {
            io::Error::new(io::ErrorKind::Other, "Error applying button stlye.\n")
        })?;
        apply_button_style(&button_init).map_err(|_err| {
            io::Error::new(io::ErrorKind::Other, "Error applying button stlye.\n")
        })?;
        apply_button_style(&button_open_repo).map_err(|_err| {
            io::Error::new(io::ErrorKind::Other, "Error applying button stlye.\n")
        })?;

        connect_button_clicked_main_window(&button_clone, "Clone")?;
        connect_button_clicked_main_window(&button_init, "Init")?;
        connect_button_clicked_open_new_repository(&button_open_repo)?;
        window.show_all();
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to run main window.",
        ))
    }
}

/// Connects the "clicked" signal of a GTK button to open a new repository.
///
/// This function is designed to be connected to the "clicked" signal of a GTK button.
/// When the button is clicked, it opens a file chooser dialog for selecting a directory.
/// If a valid directory is selected, it checks if it contains an ".mgit" folder, indicating
/// a valid repository. If so, it closes all windows, shows the repository window for the
/// selected directory, and prints diagnostic information. If the directory is not a valid
/// repository, it displays an error message.
///
/// # Arguments
///
/// * `button` - A reference to the GTK button triggering the action.
///
/// # Errors
///
/// Returns a `std::io::Result<()>` to handle potential I/O errors that may occur during
/// file system operations or window management.
///
fn connect_button_clicked_open_new_repository(button: &gtk::Button) -> std::io::Result<()> {
    button.connect_clicked(move |_| {
        let dialog = FileChooserDialog::new(
            Some("Select Directory"),
            Some(&gtk::Window::new(gtk::WindowType::Popup)),
            FileChooserAction::SelectFolder,
        );
        dialog.add_button("Open", gtk::ResponseType::Ok);
        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        let result = dialog.run();
        if result == gtk::ResponseType::Ok {
            if let Some(selected_directory) = dialog.get_filename() {
                let mgit_folder_path = selected_directory.join(GIT_DIR);
                if mgit_folder_path.is_dir() {
                    let code_dir = match std::env::current_dir() {
                        Ok(dir) => dir,
                        Err(_) => Path::new("").to_path_buf(),
                    };
                    close_all_windows();
                    let result = show_repository_window(&code_dir, &selected_directory);
                    if result.is_err() {
                        eprintln!("Couldn't show repository window");
                    }
                    println!("Selected directory: {:?}", selected_directory);
                } else {
                    show_message_dialog("Error", "The folder is not a directory");
                    println!("Not an mgit directory");
                }
            }
        }
        dialog.hide();
    });
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
