use gtk::prelude::*;
use gtk::Builder;
use gtk::Window;
use pango::Alignment;

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
        if button_type == "Init" {
            let builder_window_init = gtk::Builder::new();
            builder_window_init.add_from_file("src/windowInit.ui"); // Asegúrate de que la ruta sea correcta
            let new_window_init: gtk::Window = builder_window_init.get_object("window").expect("No se puede obtener la ventana");
            new_window_init.set_default_size(800, 600);
            apply_window_style(&new_window_init);
            let button1: gtk::Button = get_button(&builder_window_init, "button1", "option1");
            let button2: gtk::Button = get_button(&builder_window_init, "button2", "option2");
            let button3: gtk::Button = get_button(&builder_window_init, "button3", "option3");

            apply_common_style(&button2, &button1);
            apply_common_style(&button3, &button1);

            // Mostrar la nueva ventana "Init"
            new_window_init.show_all();
        } else if button_type == "Clone" {
            // Código para el botón "Clone" aquí
        }
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
