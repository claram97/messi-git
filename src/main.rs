use messi::gui::run_main_window;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    run_main_window();

    gtk::main();
}
