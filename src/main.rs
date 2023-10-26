use gtk::prelude::*;
use gtk::Builder;
use gtk::Window;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let builder = Builder::new();
    builder.add_from_file("src/part3.ui"); // Reemplaza con el nombre de tu archivo .ui

    let window: gtk::Window = builder.get_object("window").expect("No se puede obtener la ventana");
    window.set_default_size(800, 600);
    let button_clone: gtk::Button = get_button(&builder, "buttonclone", "Clone");
    let button_init: gtk::Button = get_button(&builder, "buttoninit", "Init");

    apply_common_style(&button_clone, &button_init);
    apply_window_style(&window); 
    connect_button_clicked(&button_clone, "Clone");
    connect_button_clicked(&button_init, "Init");
    window.show_all();

    gtk::main();
}

fn connect_button_clicked(button: &gtk::Button, button_type: &str) {
    let button_type = button_type.to_owned(); // Clonar la etiqueta
    
    button.connect_clicked(move |_| {
        let new_window = gtk::Window::new(gtk::WindowType::Toplevel);
       

        match &button_type[..] {
            "Clone" => {
                // Acciones específicas para el botón "Clone"
                println!("Botón 'Clone' presionado");
                new_window.set_title("Clone Repository");
            }
            "Init" => {
                // Acciones específicas para el botón "Init"
                println!("Botón 'Init' presionado");
                new_window.set_title("Init  Repository");
            }
            _ => {
                // Manejar otros casos si es necesario
            }
        }
        new_window.set_default_size(800, 600);
        apply_window_style(&new_window); 
        new_window.show_all();
    });
}


fn apply_window_style(window: &gtk::Window) {
    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data("window {
            background-color: #87CEEB; /* Color celeste */
        }"
        .as_bytes())
        .expect("Failed to load CSS");

    let style_context = window.get_style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}


fn get_button(builder: &Builder, button_id: &str, label_text: &str) -> gtk::Button {
    let button: gtk::Button = builder.get_object(button_id).expect(&format!("No se puede obtener el botón {}", label_text));
    let label = button.get_child().unwrap().downcast::<gtk::Label>().unwrap();
    let pango_desc = pango::FontDescription::from_string("Sans 30");
    label.override_font(&pango_desc);
    button.show();
    button
}

fn apply_common_style(button_clone: &gtk::Button, button_init: &gtk::Button) {
    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data("button {
            background-color: #87CEEB; /* Color celeste */
            color: #1e3799; /* Color de texto azul oscuro */
            border: 10px solid #1e3799; /* Borde azul oscuro */
            padding: 10px; /* Espaciado alrededor del contenido */
        }"
        .as_bytes())
        .expect("Failed to load CSS");

    let style_context_clone = button_clone.get_style_context();
    style_context_clone.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let style_context_init = button_init.get_style_context();
    style_context_init.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}
