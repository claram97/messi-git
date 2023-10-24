extern crate gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, CssProvider, Box, Orientation, Button};

pub fn run_gui() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = create_window();
    let container = create_gui_elements(&window);

    window.show_all();
    gtk::main();
}

fn create_window() -> Window {
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Bienvenido a GitMessi");
    window.set_default_size(800, 600);

    // Agregar un estilo CSS para establecer el fondo en azul
    let provider = CssProvider::new();
    provider
        .load_from_data(
            "window {
                background-color: #3498db; /* Azul */
            }
            button {
                background-color: #3498db;
                color: white;
                font-family: monospace;
                border: none;
                padding: 10px;
            }
            ".as_bytes(),
        )
        .expect("Failed to load CSS");

    let context = window.get_style_context();
    context.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    window
}

fn create_gui_elements(window: &Window) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Etiqueta de bienvenida en el centro
    let label = gtk::Label::new(Some("Bienvenido a GitMessi"));
    label.set_halign(gtk::Align::Center);
    label.set_valign(gtk::Align::Center);

    // Aplicar la clase personalizada al Label
    let label_style_context = label.get_style_context();
    label_style_context.add_class("custom-label");


    // Contenedor para los botones
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    button_box.set_spacing(100); // Agrega espaciado entre los botones

    for i in 1..=4 {
        let button = gtk::Button::with_label(&format!("Botón {}", i));
        
        // Aplicar el estilo personalizado a cada botón
        let style_context = button.get_style_context();
        style_context.add_class("custom-button");

        button_box.add(&button);
    }

    // Aplicar el estilo al contenedor de botones
    let button_box_style_context = button_box.get_style_context();
    button_box_style_context.add_class("horizontal-button-box");

    container.pack_start(&label, true, true, 0);
    container.pack_start(&button_box, false, false, 0);

    // Agrega el contenedor a la ventana
    window.add(&container);

    container
}
