use crate::gui::main_window::add_to_open_windows;
use gtk::prelude::*;
use gtk::BinExt;
use gtk::Builder;
use gtk::CssProviderExt;
use gtk::StyleContextExt;
use gtk::WidgetExt;
use std::io;

/// Retrieves a GTK button from a `gtk::Builder` by its ID and applies a specific style.
///
/// This function looks for a button in the provided `builder` using the given `button_id`.
/// If the button is found, it retrieves the child widget and attempts to downcast it to a
/// `gtk::Label`. If successful, it applies a custom font style and shows the button.
///
/// # Arguments
///
/// - `builder`: A reference to a `gtk::Builder` containing the button.
/// - `button_id`: The ID of the button to retrieve.
/// - `label_text`: The label text for the button.
///
/// # Returns
///
/// A `gtk::Button` widget if it was successfully retrieved, otherwise, it returns an
/// empty `gtk::Button`.
///
pub fn get_button(builder: &Builder, button_id: &str) -> gtk::Button {
    if let Some(button) = builder.get_object::<gtk::Button>(button_id) {
        if let Some(child) = button.get_child() {
            if let Ok(label) = child.downcast::<gtk::Label>() {
                let pango_desc = pango::FontDescription::from_string("Sans 20");
                label.override_font(&pango_desc);
                button.show();
            }
        }
        return button;
    }

    gtk::Button::new()
}

/// Retrieve a GTK TextView widget from a GTK Builder.
///
/// This function takes a reference to a GTK Builder and an ID string, and attempts to find a GTK TextView widget
/// in the builder's objects. If the TextView widget is found, it is returned as an `Option<gtk::TextView>`. If it is
/// not found, `None` is returned.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK Builder containing the widgets.
/// * `text_view_id` - A string representing the ID of the TextView widget to retrieve.
///
/// # Returns
///
/// * `Some(gtk::TextView)` if the TextView is found in the builder.
/// * `None` if the TextView is not found.
///
pub fn get_text_view(builder: &Builder, text_view_id: &str) -> Option<gtk::TextView> {
    if let Some(text_view) = builder.get_object::<gtk::TextView>(text_view_id) {
        return Some(text_view);
    }

    None
}

pub fn get_switch(builder: &gtk::Builder, switch_id: &str) -> Option<gtk::Switch> {
    if let Some(switch) = builder.get_object::<gtk::Switch>(switch_id) {
        return Some(switch);
    }

    None
}

/// Applies a custom button style using CSS to the provided `gtk::Button`.
///
/// This function sets a custom CSS style for the provided `gtk::Button` widget to change its appearance.
///
/// # Arguments
///
/// * `button` - A reference to the `gtk::Button` to which the style will be applied.
///
/// # Returns
///
/// Returns a `Result<(), String>` where `Ok(())` indicates success, and `Err` contains an error message if the CSS loading fails.
///
pub fn apply_button_style(button: &gtk::Button) -> Result<(), String> {
    let css_provider = gtk::CssProvider::new();
    if let Err(err) = css_provider.load_from_data(
        "button {
        background-color: #87CEEB; /* Sky Blue */
        color: #1e3799; /* Dark Blue Text Color */
        border: 5px solid #1e3799; /* Dark Blue Border */
    }"
        .as_bytes(),
    ) {
        return Err(format!("Failed to load CSS: {}", err));
    }

    let style_context = button.get_style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    Ok(())
}

/// Retrieve a GTK label widget from a GTK builder and apply a custom font size.
///
/// This function attempts to retrieve a GTK label widget using its ID from the provided GTK builder.
/// If successful, it overrides the font size for the label with the specified `font_size` and makes
/// the label visible. If the label is not found, it logs an error message and returns `None`.
///
/// # Arguments
///
/// * `builder` - A reference to the `gtk::Builder` containing the UI definition.
/// * `label_id` - The ID of the label widget to retrieve from the builder.
/// * `font_size` - The font size to apply to the label.
///
/// # Returns
///
/// An `Option<gtk::Label>`:
/// - Some(label) if the label is found and styled successfully.
/// - None if the label with the specified ID is not found in the builder.
pub fn get_label(builder: &gtk::Builder, label_id: &str, font_size: f64) -> Option<gtk::Label> {
    if let Some(label) = builder.get_object::<gtk::Label>(label_id) {
        let pango_desc = pango::FontDescription::from_string(&format!("Sans {:.1}", font_size));
        label.override_font(&pango_desc);
        label.show();
        Some(label)
    } else {
        eprintln!("Failed to get the label with ID: {}", label_id);
        None
    }
}

/// Retrieve a GTK entry widget from a GTK builder.
///
/// This function attempts to retrieve a GTK entry widget using its ID from the provided GTK builder.
/// If successful, it returns the entry widget. If the entry is not found, it logs an error message and
/// returns `None`.
///
/// # Arguments
///
/// * `builder` - A reference to the `gtk::Builder` containing the UI definition.
/// * `entry_id` - The ID of the entry widget to retrieve from the builder.
///
/// # Returns
///
/// An `Option<gtk::Entry>`:
/// - Some(entry) if the entry is found in the builder.
/// - None if the entry with the specified ID is not found in the builder.
pub fn get_entry(builder: &gtk::Builder, entry_id: &str) -> Option<gtk::Entry> {
    if let Some(entry) = builder.get_object::<gtk::Entry>(entry_id) {
        Some(entry)
    } else {
        eprintln!("Failed to get the entry with ID: {}", entry_id);
        None
    }
}

/// Apply a custom CSS style to a GTK window.
///
/// This function takes a reference to a `gtk::Window` and applies a custom CSS style to it
/// to change its background color.
///
/// # Arguments
///
/// * `window` - A reference to the `gtk::Window` to which the style will be applied.
///
pub fn apply_window_style(window: &gtk::Window) -> Result<(), Box<dyn std::error::Error>> {
    let css_data = "window {
        background-color: #87CEEB; /* Sky Blue */
    }";

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(css_data.as_bytes())?;

    let style_context = window.get_style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    Ok(())
}

/// Load a GTK window from a UI file and retrieve it from a GTK builder.
///
/// This function loads a GTK window from a UI file and retrieves it from a GTK builder using
/// the specified window name.
///
/// # Arguments
///
/// * `builder` - A reference to the `gtk::Builder` used to load the window.
/// * `ui_path` - A string specifying the path to the UI file.
/// * `window_name` - A string specifying the name of the window to retrieve.
///
/// # Returns
///
/// An `Option<gtk::Window>` containing the loaded window if successful, or `None` on failure.
///
pub fn load_and_get_window(
    builder: &gtk::Builder,
    ui_path: &str,
    window_name: &str,
) -> Option<gtk::Window> {
    match builder.add_from_file(ui_path) {
        Ok(_) => builder.get_object(window_name),
        Err(err) => {
            eprintln!("Error loading the UI file: {}", err);
            None
        }
    }
}

/// Applies a custom CSS style to a GTK label.
///
/// This function sets the text color to blue for the label.
/// It uses a CSS provider to load the styles and applies them to the label's style context.
///
/// # Arguments
///
/// * `label` - A reference to the `gtk::Label` widget to style.
pub fn apply_label_style(label: &gtk::Label) {
    let css_provider = gtk::CssProvider::new();
    if let Err(err) = css_provider.load_from_data(
        "label {
        color: #1e3799; /* Texto azul */
    }"
        .as_bytes(),
    ) {
        eprintln!("Failed to load CSS for label: {}", err);
    }

    let style_context = label.get_style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}

/// Applies a custom CSS style to a GTK entry.
///
/// This function sets the background color to white, text color to black, adds a blue border,
/// and sets padding for the entry.
/// It uses a CSS provider to load the styles and applies them to the entry's style context.
///
/// # Arguments
///
/// * `entry` - A reference to the `gtk::Entry` widget to style.
pub fn apply_entry_style(entry: &gtk::Entry) {
    let css_provider = gtk::CssProvider::new();
    if let Err(err) = css_provider.load_from_data(
        "entry {
            background-color: #FFFFFF; /* Fondo blanco */
            color: #000000; /* Texto negro */
            border: 2px solid #1e3799; /* Borde azul */
            padding: 7px; /* Espacio interno */
    }"
        .as_bytes(),
    ) {
        eprintln!("Failed to load CSS for entry: {}", err);
    }

    let style_context = entry.get_style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
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
pub fn filter_color_code(input: &str) -> String {
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

pub fn get_combo_box(builder: &gtk::Builder, id: &str) -> io::Result<gtk::ComboBoxText> {
    let combo_box = match builder.get_object::<gtk::ComboBoxText>(id) {
        Some(combo_box) => combo_box,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("No se pudo encontrar el ComboBoxText con ID {id}"),
            ));
        }
    };
    Ok(combo_box)
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
pub fn create_text_entry_window(
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

    let entry_window_clone = entry_window.clone();
    ok_button.connect_clicked(move |_| {
        let text = entry.get_text().to_string();
        entry_window.close();
        on_text_entered(text);
    });

    entry_window_clone.show_all();
    Ok(())
}

/// Creates a text entry window with two input fields.
///
/// This function creates a GTK window with two text entry fields and an "OK" button.
/// It takes two messages as input to set as default text in each entry field. When the
/// user clicks "OK," the provided closure `on_text_entered` is called with the entered
/// text from both fields.
///
/// # Arguments
///
/// * `message1` - The initial text for the first entry field.
/// * `message2` - The initial text for the second entry field.
/// * `on_text_entered` - A closure that will be called with the entered text from both fields.
///
/// # Returns
///
/// An `io::Result` indicating whether the operation was successful or resulted in an error.
///
pub fn create_text_entry_window2(
    message1: &str,
    message2: &str,
    on_text_entered: impl Fn(String, String) + 'static,
) -> io::Result<()> {
    let entry_window = gtk::Window::new(gtk::WindowType::Toplevel);
    add_to_open_windows(&entry_window);
    apply_window_style(&entry_window)
        .map_err(|_err| io::Error::new(io::ErrorKind::Other, "Error applying window stlye.\n"))?;
    entry_window.set_title(&format!("{} {}", message1, message2));
    entry_window.set_default_size(400, 150);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    entry_window.add(&main_box);

    let entry1 = gtk::Entry::new();
    entry1.set_text(message1);
    main_box.add(&entry1);

    let entry2 = gtk::Entry::new();
    entry2.set_text(message2);
    main_box.add(&entry2);

    let ok_button = gtk::Button::with_label("OK");
    apply_button_style(&ok_button)
        .map_err(|_err| io::Error::new(io::ErrorKind::Other, "Error applying button stlye.\n"))?;
    main_box.add(&ok_button);

    let entry_window_clone = entry_window.clone();
    ok_button.connect_clicked(move |_| {
        let text1 = entry1.get_text().to_string();
        let text2 = entry2.get_text().to_string();
        entry_window.close();
        on_text_entered(text1, text2);
    });

    entry_window_clone.show_all();
    Ok(())
}

/// Creates a text entry window with a switch in a GTK application.
///
/// This function generates a window with two text entry fields, a switch, and an OK button.
/// It allows the user to input text in the entry fields and toggle a switch. The provided
/// closure `on_text_entered` is called when the OK button is clicked, providing the entered
/// text from both entry fields and the state of the switch.
///
/// # Arguments
///
/// - `message1`: Initial text for the first entry field.
/// - `message2`: Initial text for the second entry field.
/// - `on_text_entered`: A closure that takes three parameters: the text from the first entry field,
///   the text from the second entry field, and a boolean indicating the state of the switch.
///
/// # Returns
///
/// - `Ok(())`: The operation was successful, and the text entry window was created and displayed.
/// - `Err(io::Error)`: An error occurred during the creation or display of the text entry window.
///
pub fn create_text_entry_window_with_switch(
    message1: &str,
    message2: &str,
    on_text_entered: impl Fn(String, String, bool) + 'static,
) -> io::Result<()> {
    let entry_window = gtk::Window::new(gtk::WindowType::Toplevel);
    add_to_open_windows(&entry_window);
    apply_window_style(&entry_window)
        .map_err(|_err| io::Error::new(io::ErrorKind::Other, "Error applying window style.\n"))?;
    entry_window.set_title(&format!("{} {}", message1, message2));
    entry_window.set_default_size(400, 150);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    entry_window.add(&main_box);

    let entry1 = gtk::Entry::new();
    entry1.set_text(message1);
    main_box.add(&entry1);

    let entry2 = gtk::Entry::new();
    entry2.set_text(message2);
    main_box.add(&entry2);

    // Crear un Switch
    let switch_label = gtk::Label::new(Some("Modify current branch"));
    let switch = gtk::Switch::new();
    switch.set_size_request(10, 10); // Ajusta el ancho del switch
    main_box.add(&switch_label);
    main_box.add(&switch);

    let ok_button = gtk::Button::with_label("OK");
    apply_button_style(&ok_button)
        .map_err(|_err| io::Error::new(io::ErrorKind::Other, "Error applying button style.\n"))?;
    main_box.add(&ok_button);

    // Manejar el clic del botón OK
    let entry_window_clone = entry_window.clone();
    ok_button.connect_clicked(move |_| {
        let text1 = entry1.get_text().to_string();
        let text2 = entry2.get_text().to_string();
        let switch_value = switch.get_active();
        entry_window.close();
        on_text_entered(text1, text2, switch_value);
    });

    // Mostrar todo y devolver el resultado
    entry_window_clone.show_all();
    Ok(())
}

/// Retrieve a GTK TextView widget from a GTK Builder.
///
/// This function takes a reference to a GTK Builder and an ID string, and attempts to find a GTK TextView widget
/// in the builder's objects. If the TextView widget is found, it is returned as an `Option<gtk::TextView>`. If it is
/// not found, `None` is returned.
///
/// # Arguments
///
/// * `builder` - A reference to a GTK Builder containing the widgets.
/// * `text_view_id` - A string representing the ID of the TextView widget to retrieve.
///
/// # Returns
///
/// * `Some(gtk::TextView)` if the TextView is found in the builder.
/// * `None` if the TextView is not found.
///
pub fn show_message_dialog(title: &str, message: &str) {
    let dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Info,
        gtk::ButtonsType::Ok,
        message,
    );
    dialog.set_title(title);
    dialog.run();
    dialog.close();
}

/// Configures the properties of a repository window in a GTK application.
///
/// This function takes a GTK window (`new_window`) as input and configures the repository window's properties, such as setting its default size and applying a specific window style, before displaying it.
///
/// # Arguments
///
/// - `new_window`: The GTK window to be configured as a repository window.
///
pub fn configure_repository_window(new_window: gtk::Window) -> io::Result<()> {
    new_window.set_default_size(800, 600);
    apply_window_style(&new_window)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to apply window style"))?;
    new_window.show_all();
    Ok(())
}
