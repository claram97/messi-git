use gtk::prelude::*;
use gtk::BinExt;
use gtk::Builder;
use gtk::CssProviderExt;
use gtk::StyleContextExt;
use gtk::WidgetExt;

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
        border: 10px solid #1e3799; /* Dark Blue Border */
        padding: 10px; /* Padding around content */
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

/// Applies a custom CSS style to a GTK button.
///
/// This function sets the background color to white, text color to blue, and a blue border for the button.
/// It uses a CSS provider to load the styles and applies them to the button's style context.
///
/// # Arguments
///
/// * `button` - A reference to the `gtk::Button` widget to style.
pub fn apply_clone_button_style(button: &gtk::Button) {
    let css_provider = gtk::CssProvider::new();
    if let Err(err) = css_provider.load_from_data(
        "button {
            background-color: #FFFFFF; /* Fondo blanco */
            color: #1e3799; /* Texto azul */
            border: 2px solid #1e3799; /* Borde azul */
        }"
        .as_bytes(),
    ) {
        eprintln!("Failed to load CSS for button: {}", err);
    }

    let style_context = button.get_style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
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
            border: 1px solid #1e3799; /* Borde azul */
            padding: 5px; /* Espacio interno */
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
