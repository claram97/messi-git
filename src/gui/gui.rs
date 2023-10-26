
use gtk::prelude::*;
use gtk::Builder;
use gtk::Window;
use pango::Alignment;

pub fn run_main_window() {
    let builder = Builder::new();
    builder.add_from_file("src/gui/part3.ui"); 

    let window: gtk::Window = builder.get_object("window").expect("No se puede obtener la ventana");
    window.set_default_size(800, 600);
    let button_clone: gtk::Button = get_button(&builder, "buttonclone", "Clone");
    let button_init: gtk::Button = get_button(&builder, "buttoninit", "Init");

    apply_common_style(&button_clone, &button_init);
    apply_window_style(&window); 
    connect_button_clicked(&button_clone, "Clone");
    connect_button_clicked(&button_init, "Init");
    window.show_all();
}
fn create_text_entry_window(button_type: &str) {
    let entry_window = gtk::Window::new(gtk::WindowType::Toplevel);
    entry_window.set_title("Ingresar Texto");
    entry_window.set_default_size(400, 150);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    entry_window.add(&main_box);

    let entry = gtk::Entry::new();
    entry.set_text("Texto predeterminado (opcional)");
    main_box.add(&entry);

    let ok_button = gtk::Button::with_label("OK");
    main_box.add(&ok_button);

    ok_button.connect_clicked(move |_| {
        let text = entry.get_text().to_string();
        println!("Texto ingresado: {}", text);

        // Aquí puedes realizar la lógica con el texto ingresado, como guardarlo o procesarlo.
        // También puedes cerrar la ventana con entry_window.close() si es necesario.
    });

    entry_window.show_all();
}
fn connect_button_clicked2(button: &gtk::Button, button_type: &str) {
    let button_type = button_type.to_owned(); // Clonar la etiqueta
    
    button.connect_clicked(move |_| {
        if button_type == "option2" {
            create_text_entry_window(&button_type);
        } else if button_type == "Clone" {
            // Código para el botón "Clone" aquí
        }
    });
}
fn connect_button_clicked(button: &gtk::Button, button_type: &str) {
    let button_type = button_type.to_owned(); // Clonar la etiqueta
    
    button.connect_clicked(move |_| {
        if button_type == "Init" {
            let builder_window_init = gtk::Builder::new();
            builder_window_init.add_from_file("src/gui/windowInit.ui"); // Asegúrate de que la ruta sea correcta
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
            connect_button_clicked2(&button2, "option2");
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
