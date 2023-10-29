use gtk::CssProviderExt;
use gtk::WidgetExt;
use gtk::StyleContextExt;

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
