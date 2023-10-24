// extern crate messi;
// use messi::gui::run_gui;

// fn main() {
//    // run_gui();
// }
use gtk::prelude::*;
use gtk::Builder;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let builder = Builder::new();
    builder.add_from_file("src/part3.ui"); // Reemplaza con el nombre de tu archivo .ui

    let window: gtk::Window = builder.get_object("window").expect("No se puede obtener la ventana");
    window.set_default_size(800, 600); 
    let button: gtk::Button = builder.get_object("buttonclone").expect("No se puede obtener el botón");
    let button_init: gtk::Button = builder.get_object("buttoninit").expect("No se puede obtener el botón Init");

    let label = button.get_child().unwrap().downcast::<gtk::Label>().unwrap();
    let pango_desc = pango::FontDescription::from_string("Sans 30"); // Cambia el tamaño de fuente a 14
    label.override_font(&pango_desc);

    let label1 = button_init.get_child().unwrap().downcast::<gtk::Label>().unwrap();
    let pango_desc = pango::FontDescription::from_string("Sans 30"); // Cambia el tamaño de fuente a 14
    label1.override_font(&pango_desc);

     // Aplicar un estilo de fondo al botón
     let css_provider = gtk::CssProvider::new();
     css_provider
         .load_from_data("button {
            background-color: #87CEEB; /* Color celeste */
             color: #1e3799; /* Color de texto blanco */
             border: 10px solid #1e3799; /* Borde azul oscuro */
             padding: 10px; /* Espaciado alrededor del contenido */
         }"
         .as_bytes())
         .expect("Failed to load CSS");
 
     let style_context = button.get_style_context();
     style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

     let style_context_init = button_init.get_style_context();
     style_context_init.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
 
 
     button.show();
     button_init.show();

    
    window.show_all();

    gtk::main();
}
