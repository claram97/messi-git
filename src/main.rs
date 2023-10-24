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
    window.show_all();

    gtk::main();
}
