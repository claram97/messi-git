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

    // Modifica las propiedades del botón
    button.set_label("Nuevo Texto"); // Cambia el texto del botón
   
    
    window.show_all();

    gtk::main();
}
