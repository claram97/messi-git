use gtk::prelude::*;
use gtk::Builder;
use gtk::Window;
use pango::Alignment;
use std::sync::Mutex;

pub static mut OPEN_WINDOWS: Option<Mutex<Vec<gtk::Window>>> = None;

pub fn run_main_window() {
    unsafe {
        OPEN_WINDOWS = Some(Mutex::new(Vec::new()));
    }
    let builder = Builder::new();
    builder.add_from_file("src/gui/part3.ui");

    let window: gtk::Window = builder.get_object("window").expect("Failed to get the window");
    window.set_default_size(800, 600);
    add_to_open_windows(&window);
    let button_clone: gtk::Button = get_button(&builder, "buttonclone", "Clone");
    let button_init: gtk::Button = get_button(&builder, "buttoninit", "Init");

    apply_common_style(&button_clone, &button_init);
    apply_window_style(&window);
    connect_button_clicked(&button_clone, "Clone");
    connect_button_clicked(&button_init, "Init");
    window.show_all();
}

pub fn close_all_windows() {
    unsafe {
        if let Some(ref mutex) = OPEN_WINDOWS {
            let mut open_windows = mutex.lock().expect("Mutex lock failed");
            for window in open_windows.iter() {
                window.close(); // Close each window
            }
            open_windows.clear(); // Clear the list
        }
    }
}

fn print_open_windows() {
    unsafe {
        if let Some(ref windows) = OPEN_WINDOWS {
            if let Ok(windows) = windows.lock() {
                for window in windows.iter() {
                    println!("Window: {:?}", window);
                }
            } else {
                println!("Failed to lock the window mutex");
            }
        } else {
            println!("OPEN_WINDOWS is not initialized");
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

fn show_new_window() {
    let builder = gtk::Builder::new();
    builder.add_from_file("src/gui/new_window.ui");

    let new_window: gtk::Window = builder.get_object("window").expect("Failed to get the window");
    new_window.set_default_size(800, 600);
    apply_window_style(&new_window);

    new_window.show_all();
}

fn create_text_entry_window(button_type: &str, message: &str) {
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
    apply_common_style(&ok_button, &ok_button);
    main_box.add(&ok_button);

    ok_button.connect_clicked(move |_| {
        let text = entry.get_text().to_string();
        println!("Entered Text: {}", text);
        close_all_windows();
        show_new_window();
    });

    entry_window.show_all();
}

fn connect_button_clicked2(button: &gtk::Button, button_type: &str) {
    let button_type = button_type.to_owned(); // Clone the label

    button.connect_clicked(move |_| {
        if button_type == "option2" {
            create_text_entry_window(&button_type, "Enter the branch");
        } else if button_type == "option3" {
            create_text_entry_window(&button_type, "Enter the template path");
        } else if button_type == "option1" {
            close_all_windows();
            show_new_window();
        }
    });
}

fn connect_button_clicked(button: &gtk::Button, button_type: &str) {
    let button_type = button_type.to_owned(); // Clone the label

    button.connect_clicked(move |_| {
        if button_type == "Init" {
            let builder_window_init = gtk::Builder::new();
            builder_window_init.add_from_file("src/gui/windowInit.ui"); // Make sure the path is correct
            let new_window_init: gtk::Window = builder_window_init.get_object("window").expect("Failed to get the window");
            add_to_open_windows(&new_window_init);
            new_window_init.set_default_size(800, 600);
            apply_window_style(&new_window_init);
            let button1: gtk::Button = get_button(&builder_window_init, "button1", "option1");
            let button2: gtk::Button = get_button(&builder_window_init, "button2", "option2");
            let button3: gtk::Button = get_button(&builder_window_init, "button3", "option3");

            apply_common_style(&button2, &button1);
            apply_common_style(&button3, &button1);

            // Show the new "Init" window
            new_window_init.show_all();
            connect_button_clicked2(&button1, "option1");
            connect_button_clicked2(&button2, "option2");
            connect_button_clicked2(&button3, "option3");
        } else if button_type == "Clone" {
            let builder_window_clone = gtk::Builder::new();
            builder_window_clone.add_from_file("src/gui/windowClone.ui"); // Make sure the path is correct
            let new_window_clone: gtk::Window = builder_window_clone.get_object("window").expect("Failed to get the window");
            add_to_open_windows(&new_window_clone);
            new_window_clone.set_default_size(800, 600);
            apply_window_style(&new_window_clone);
            new_window_clone.show_all();
        }
    });
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
    let pango_desc = pango::FontDescription::from_string("Sans 30");
    label.override_font(&pango_desc);
    button.show();
    button
}

fn apply_common_style(button_clone: &gtk::Button, button_init: &gtk::Button) {
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

    let style_context_clone = button_clone.get_style_context();
    style_context_clone.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let style_context_init = button_init.get_style_context();
    style_context_init.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}
