use crate::gui::gui::add_to_open_windows;
use crate::gui::style::apply_button_style;
use crate::gui::style::apply_window_style;
use crate::gui::style::get_button;
use crate::gui::run_main_window;
use crate::gui::gui::close_all_windows;
use crate::init::git_init;
use crate::gui::style::create_text_entry_window;
use crate::gui::repository_window::show_repository_window;
use gtk::GtkWindowExt;
use gtk::ButtonExt;
use std::io;

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
