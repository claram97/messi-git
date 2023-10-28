use gtk::prelude::*;
use gtk::Builder;
use std::sync::Mutex;
use gtk::CssProvider;
use std::rc::Rc;
use core::cell::RefCell;
use gtk::Label;

pub static mut OPEN_WINDOWS: Option<Mutex<Vec<gtk::Window>>> = None;

pub fn run_main_window() {
    unsafe {
        OPEN_WINDOWS = Some(Mutex::new(Vec::new()));
    }

    let builder = Builder::new();
    if let Some(window) = load_and_get_window(&builder,"src/gui/part3.ui", "window") {
        
        window.set_default_size(800, 600);
        add_to_open_windows(&window);
        apply_window_style(&window);
    
        let button_clone: gtk::Button = get_button(&builder, "buttonclone", "Clone");
        let button_init: gtk::Button = get_button(&builder, "buttoninit", "Init");
        apply_button_style(&button_clone);
        apply_button_style(&button_init);
    
        connect_button_clicked_main_window(&button_clone, "Clone");
        connect_button_clicked_main_window(&button_init, "Init");
        
        window.show_all();
    }
   
}

pub fn close_all_windows() {
    unsafe {
        if let Some(ref mutex) = OPEN_WINDOWS {
            let mut open_windows = mutex.lock().expect("Mutex lock failed");
            for window in open_windows.iter() {
                window.close(); 
            }
            open_windows.clear(); 
        }
    }
}

fn add_to_open_windows(window: &gtk::Window) {
    unsafe {
        if let Some(ref mutex) = OPEN_WINDOWS {
            let mut open_windows = mutex.lock().expect("Mutex lock failed");
            open_windows.push(window.clone());
        }
    }
}
fn obtener_texto_desde_funcion() -> String {
    // Aquí puedes colocar el código de tu función para obtener el texto
    "Hola, mundo".to_string() // Ejemplo: Retorna un texto fijo
}

 
fn show_repository_window() {
    let builder = gtk::Builder::new();
    if let Some(new_window) = load_and_get_window(&builder,"src/gui/new_window2.ui", "window") {
        let new_window_clone = new_window.clone(); // Clonamos la ventana
        configure_repository_window(new_window);
        let button1 = get_button(&builder, "button1", "Add");
        let button2 = get_button(&builder, "button2", "Commit");
        let button3 = get_button(&builder, "button3", "Push");
        let button4 = get_button(&builder, "button4", "Push");
        let button5 = get_button(&builder, "button5", "Push");
        let button6 = get_button(&builder, "button6", "Push");
        let button7 = get_button(&builder, "button7", "Push");
        let button8 = get_button(&builder, "button8", "Push");


        apply_button_style(&button1);
        apply_button_style(&button2);
        apply_button_style(&button3);
        apply_button_style(&button4);
        apply_button_style(&button5);
        apply_button_style(&button6);
        apply_button_style(&button7);
        apply_button_style(&button8);

        button1.connect_clicked(move |_| {
            println!("Button 1 (Add) clicked.");
        });

        button2.connect_clicked(move |_| {
            println!("Button 2 (Commit) clicked.");
        });

        button3.connect_clicked(move |_| {
            println!("Button 3 (Push) clicked.");
        });

        button4.connect_clicked(move |_| {
            println!("Button 4 clicked.");
        });

        button5.connect_clicked(move |_| {
            println!("Button 5 clicked.");
        });

        button6.connect_clicked(move |_| {
            println!("Button 6 clicked.");
        });

        button7.connect_clicked(move |_| {
            let label: Label = builder.get_object("label").unwrap();
            let texto_desde_funcion = obtener_texto_desde_funcion();
            label.set_text(&texto_desde_funcion);
        });

        button8.connect_clicked(move |_| {
            new_window_clone.close();
            run_main_window();
        });
    }
    
   
}

fn configure_repository_window(new_window: gtk::Window) {
    new_window.set_default_size(800, 600);
    apply_window_style(&new_window);
    new_window.show_all();
}

fn create_text_entry_window( message: &str) {
    let entry_window = gtk::Window::new(gtk::WindowType::Toplevel);
    add_to_open_windows(&entry_window);
    apply_window_style(&entry_window);
    entry_window.set_title(message);
    entry_window.set_default_size(400, 150);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    entry_window.add(&main_box);

    let entry = gtk::Entry::new();
    entry.set_text("Default Text");
    main_box.add(&entry);

    let ok_button = gtk::Button::with_label("OK");
    apply_button_style(&ok_button);
    main_box.add(&ok_button);

    ok_button.connect_clicked(move |_| {
        let text = entry.get_text().to_string();
        println!("Entered Text: {}", text);
        close_all_windows();
        show_repository_window();
    });

    entry_window.show_all();
}

fn connect_button_clicked_init_window(button: &gtk::Button, button_type: &str) {
    let button_type = button_type.to_owned(); 

    button.connect_clicked(move |_| {
        if button_type == "option2" {
            create_text_entry_window( "Enter the branch");
        } else if button_type == "option3" {
            create_text_entry_window( "Enter the template path");
        } else if button_type == "option1" {
            close_all_windows();
            show_repository_window();
        }
    });
}


fn configure_init_window(new_window_init: &gtk::Window, builder: &gtk::Builder) {
    add_to_open_windows(new_window_init);
    apply_window_style(new_window_init);
    new_window_init.set_default_size(800, 600);

    let button1 = get_button(builder, "button1", "option1");
    let button2 = get_button(builder, "button2", "option2");
    let button3 = get_button(builder, "button3", "option3");

    apply_button_style(&button1);
    apply_button_style(&button2);
    apply_button_style(&button3);

    connect_button_clicked_init_window(&button1, "option1");
    connect_button_clicked_init_window(&button2, "option2");
    connect_button_clicked_init_window(&button3, "option3");
}

fn configure_clone_window(new_window_clone: &gtk::Window, builder: &gtk::Builder) {
    add_to_open_windows(new_window_clone);
    apply_window_style(new_window_clone);
    new_window_clone.set_default_size(800, 600);

}

fn connect_button_clicked_main_window(button: &gtk::Button, button_type: &str) {
    let button_type = button_type.to_owned();
    
    button.connect_clicked(move |_| {
        let builder = gtk::Builder::new();
        match button_type.as_str() {
            "Init" => {
                if let Some(new_window_init) = load_and_get_window(&builder, "src/gui/windowInit.ui", "window") {
                    configure_init_window(&new_window_init, &builder);
                    new_window_init.show_all();
                }
            },
            "Clone" => {
                if let Some(new_window_clone) = load_and_get_window(&builder, "src/gui/windowClone.ui", "window") {
                    configure_clone_window(&new_window_clone, &builder);
                    new_window_clone.show_all();
                }
            },
            _ => eprintln!("Unknown button type: {}", button_type),
        }
    });
}

fn load_and_get_window(builder: &gtk::Builder, ui_path: &str, window_name: &str) -> Option<gtk::Window> {
    match builder.add_from_file(ui_path) {
        Ok(_) => {
            builder.get_object(window_name)
        }
        Err(err) => {
            eprintln!("Error loading the UI file: {}", err);
            None
        }
    }
}

fn apply_window_style(window: &gtk::Window) {
    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data("window {
            background-color: #87CEEB; /* Sky Blue */
        }"
        .as_bytes())
        .expect("Failed to load CSS");

    let style_context = window.get_style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}

fn get_button(builder: &Builder, button_id: &str, label_text: &str) -> gtk::Button {
    let button: gtk::Button = builder.get_object(button_id).expect(&format!("Failed to get the button {}", label_text));
    let label = button.get_child().unwrap().downcast::<gtk::Label>().unwrap();
    let pango_desc = pango::FontDescription::from_string("Sans 20");
    label.override_font(&pango_desc);
    button.show();
    button
}

fn apply_button_style(button: &gtk::Button) {
    let css_provider = gtk::CssProvider::new();
    css_provider
        .load_from_data("button {
            background-color: #87CEEB; /* Sky Blue */
            color: #1e3799; /* Dark Blue Text Color */
            border: 10px solid #1e3799; /* Dark Blue Border */
            padding: 10px; /* Padding around content */
        }"
        .as_bytes())
        .expect("Failed to load CSS");

    let style_context = button.get_style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}
