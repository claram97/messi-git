use crate::gui::gui::add_to_open_windows;
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
use std::io;

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

        if url_text.is_empty() || dir_text.is_empty() {
            // let error_dialog = gtk::MessageDialog::new(
            //     Some(&new_window_clone_clone),
            //     gtk::DialogFlags::MODAL,
            //     gtk::MessageType::Error,
            //     gtk::ButtonsType::Ok,
            //     "Faltan datos: URL o directorio de clonación.",
            // );
            // error_dialog.run();
            // error_dialog.close();
            show_message_dialog("Error", "Faltan datos: URL o directorio de clonación.");
        } else {
            println!("Ok!");
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
pub fn configure_clone_window(
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
    apply_entry_style(&dir_to_clone_entry_clone);

    let browse_button = get_button(builder, "browse-button");
    let clone_button = get_button(builder, "clone-button");
    apply_button_style(&browse_button).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    apply_button_style(&clone_button).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    connect_button_clicked_clone_repository(&clone_button, &url_entry, &dir_to_clone_entry)?;
    connect_button_clicked_browse(&browse_button, new_window_clone, &dir_to_clone_entry);

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
