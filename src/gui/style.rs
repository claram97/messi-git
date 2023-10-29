use gtk::CssProviderExt;
use gtk::WidgetExt;
use gtk::StyleContextExt;
use gtk::BinExt;
use std::thread::Builder as OtherBuilder;
use gtk::prelude::*;
use gtk::Builder;


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
pub fn get_button(builder: &Builder, button_id: &str, label_text: &str) -> gtk::Button {
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
    
    eprintln!("Failed to get the button {}", label_text);
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
/// # Examples
///
/// ```rust
/// use gtk::Button;
///
/// let my_button = Button::new_with_label("Custom Button");
/// if let Err(err) = apply_button_style(&my_button) {
///     eprintln!("Error applying button style: {}", err);
/// }
/// ```
pub fn apply_button_style(button: &gtk::Button) -> Result<(), String> {
    let css_provider = gtk::CssProvider::new();
    if let Err(err) = css_provider.load_from_data("button {
        background-color: #87CEEB; /* Sky Blue */
        color: #1e3799; /* Dark Blue Text Color */
        border: 10px solid #1e3799; /* Dark Blue Border */
        padding: 10px; /* Padding around content */
    }".as_bytes()) {
        return Err(format!("Failed to load CSS: {}", err));
    }

    let style_context = button.get_style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    Ok(())
}
