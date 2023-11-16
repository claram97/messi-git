use crate::clone;
use crate::gui::main_window::add_to_open_windows;
use crate::gui::style::apply_button_style;
use crate::gui::style::apply_entry_style;
use crate::gui::style::apply_label_style;
use crate::gui::style::apply_window_style;
use crate::gui::style::get_button;
use crate::gui::style::get_entry;
use crate::gui::style::get_label;
use gtk::ButtonExt;
use gtk::DialogExt;
use gtk::Entry;
use gtk::EntryExt;
use gtk::FileChooserAction;
use gtk::FileChooserDialog;
use gtk::FileChooserExt;
use gtk::GtkWindowExt;
use std::env;
use std::io;
use std::path::Path;
use std::result;

use super::main_window::close_all_windows;
use super::repository_window::show_repository_window;
use super::style::show_message_dialog;

/// Handles the "Browse" button click event in a GTK application.
/// When the "Browse" button is clicked, a file dialog is displayed
/// to allow the user to select a directory, and the selected directory
/// path is then displayed in a text entry field.
///
/// # Parameters
///
/// - `button`: A reference to the GTK button widget that triggers the action.
/// - `new_window`: A reference to the GTK window where the file dialog will be displayed.
/// - `dir_to_clone_entry`: A reference to the GTK entry widget where the selected
///   directory path will be displayed.
///
/// # Usage
///
/// You can use this function to connect the "clicked" signal of a GTK button to handle
/// directory selection. When the button is clicked, a file dialog will be displayed,
/// and the selected directory path will be shown in the specified entry field.
pub fn connect_button_clicked_browse(
    button: &gtk::Button,
    new_window: &gtk::Window,
    dir_to_clone_entry: &Entry,
) {
    let dir_to_clone_entry_clone = dir_to_clone_entry.clone();
    let new_window_clone = new_window.clone();
    button.connect_clicked(move |_| {
        let dialog: FileChooserDialog = FileChooserDialog::new(
            Some("Seleccionar Carpeta"),
            Some(&new_window_clone),
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
}

/// Handles the "Clone Repository" button click event in a GTK application.
/// It retrieves the URL and directory paths from the specified entry fields
/// and performs some error checking. If both fields are filled, it prints "Ok!"
/// to the console.
///
/// # Parameters
///
/// - `button`: A reference to the GTK button widget that triggers the action.
/// - `url_entry`: A reference to the GTK entry widget containing the URL.
/// - `dir_to_clone_entry`: A reference to the GTK entry widget containing the
///   directory path for cloning.
///
/// # Returns
///
/// Returns a Result indicating the success or failure of the operation.
///
/// # Usage
///
/// You can use this function to connect the "clicked" signal of a GTK button to handle
/// the cloning of a repository. It checks if both the URL and directory fields are
/// filled. If they are, it prints "Ok!" to the console.
fn connect_button_clicked_clone_repository(
    button: &gtk::Button,
    url_entry: &Entry,
    dir_to_clone_entry: &Entry,
) -> io::Result<()> {
    let url_entry_clone = url_entry.clone();
    let dir_to_clone_entry_clone = dir_to_clone_entry.clone();
    button.connect_clicked(move |_| {
        let url_text = url_entry_clone.get_text().to_string();
        let dir_text = dir_to_clone_entry_clone.get_text().to_string();
        let code_dir = match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => Path::new("").to_path_buf(),
        };

        if url_text.is_empty() || dir_text.is_empty() {
            show_message_dialog("Error", "Faltan datos: URL o directorio de clonación.");
        } else {
            let remote_repo_url = match url_text.rsplit_once('/') {
                Some((string, _)) => string,
                None => "",
            };

            let remote_repo_name = url_text.split('/').last().unwrap_or("");

            println!("URL: {}", remote_repo_url);
            println!("Remote repo URL: {}", remote_repo_url);
            let working_dir = Path::new(&dir_text);

            let result =
                clone::git_clone(remote_repo_url, remote_repo_name, "localhost", &dir_text);
            handle_clone_result(result, &code_dir, working_dir);
        }
    });
    Ok(())
}

/// Handles the result of a repository clone operation and performs corresponding actions.
///
/// # Arguments
///
/// * `result` - The result of the repository clone operation.
/// * `code_dir` - The path to the directory containing the cloned repository.
/// * `working_dir` - The working directory where the user intends to operate.
///
fn handle_clone_result(result: result::Result<(), io::Error>, code_dir: &Path, working_dir: &Path) {
    if result.is_err() {
        show_message_dialog("Error", "Error al clonar el repositorio.");
    } else {
        show_message_dialog("Ok", "Repositorio clonado correctamente.");
        let result = env::set_current_dir(working_dir);
        if result.is_err() {
            show_message_dialog("Error", "Error al cambiar de directorio.");
        } else {
            close_all_windows();
            let result = show_repository_window(code_dir, working_dir);
            if result.is_err() {
                show_message_dialog("Error", "Error al mostrar la ventana del repositorio.");
            }
        }
    }
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
pub fn configure_clone_window(
    new_window_clone: &gtk::Window,
    builder: &gtk::Builder,
) -> io::Result<()> {
    add_to_open_windows(new_window_clone);
    let (url_entry, dir_to_clone_entry) =
        apply_styles_and_get_entries(new_window_clone, builder, "url-entry", "dir-to-clone-entry")?;
    let dir_to_clone_entry_clone = dir_to_clone_entry.clone();

    apply_entry_style(&dir_to_clone_entry);
    apply_entry_style(&dir_to_clone_entry_clone);

    let browse_button = get_button(builder, "browse-button");
    let clone_button = get_button(builder, "clone-button");
    apply_button_style(&browse_button).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    apply_button_style(&clone_button).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    connect_button_clicked_clone_repository(&clone_button, &url_entry, &dir_to_clone_entry)?;
    connect_button_clicked_browse(&browse_button, new_window_clone, &dir_to_clone_entry);

    let url_label = get_label_with_error(builder, "url-label", 14.0)?;
    let clone_dir_label = get_label_with_error(builder, "clone-dir-label", 14.0)?;
    let clone_info_label = get_label_with_error(builder, "clone-info-label", 26.0)?;

    apply_label_style(&url_label);
    apply_label_style(&clone_dir_label);
    apply_label_style(&clone_info_label);

    new_window_clone.set_default_size(800, 600);
    Ok(())
}

/// Retrieves a GTK label with the specified ID and font size from a GTK builder.
///
/// # Arguments
///
/// * `builder` - A reference to the GTK builder containing the label.
/// * `label_id` - The ID of the label to retrieve from the builder.
/// * `font_size` - The font size to set for the retrieved label.
///
/// # Returns
///
/// Returns a `Result` containing the retrieved GTK label if successful, or an `io::Error`
/// if the label with the specified ID is not found in the builder.
///
fn get_label_with_error(
    builder: &gtk::Builder,
    label_id: &str,
    font_size: f64,
) -> io::Result<gtk::Label> {
    match get_label(builder, label_id, font_size) {
        Some(label) => Ok(label),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Label not found: {}\n", label_id),
        )),
    }
}

/// Applies styles to a GTK window and retrieves two GTK entries with specified IDs.
///
/// # Arguments
///
/// * `new_window_clone` - A reference to the GTK window to apply styles to.
/// * `builder` - A reference to the GTK builder containing the entries.
/// * `url_entry_id` - The ID of the URL entry to retrieve and style.
/// * `dir_to_clone_entry_id` - The ID of the directory entry to retrieve and style.
///
/// # Returns
///
/// Returns a `Result` containing a tuple with two GTK entries (URL entry and directory entry)
/// if successful, or an `io::Error` if any style application or entry retrieval fails.
///
fn apply_styles_and_get_entries(
    new_window_clone: &gtk::Window,
    builder: &gtk::Builder,
    url_entry_id: &str,
    dir_to_clone_entry_id: &str,
) -> Result<(gtk::Entry, gtk::Entry), io::Error> {
    if let Err(err) = apply_window_style(new_window_clone) {
        eprintln!("Error applying window style: {}", err);
    }

    let url_entry = get_and_apply_entry_style(builder, url_entry_id)?;
    let dir_to_clone_entry = get_and_apply_entry_style(builder, dir_to_clone_entry_id)?;

    Ok((url_entry, dir_to_clone_entry))
}

/// Retrieves a GTK entry from a builder, applies a specific style, and returns the entry.
///
/// # Arguments
///
/// * `builder` - A reference to the GTK builder containing the entry.
/// * `entry_id` - The ID of the entry to retrieve and style.
///
/// # Returns
///
/// Returns a `Result` containing the styled GTK entry if successful,
/// or an `io::Error` if the entry is not found or if any style application fails.
///
fn get_and_apply_entry_style(
    builder: &gtk::Builder,
    entry_id: &str,
) -> Result<gtk::Entry, io::Error> {
    if let Some(entry) = get_entry(builder, entry_id) {
        apply_entry_style(&entry);
        Ok(entry)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Entry not found: {}\n", entry_id),
        ))
    }
}
