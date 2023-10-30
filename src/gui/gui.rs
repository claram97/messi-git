use gtk::FileChooserAction;
use gtk::FileChooserDialog;
use gtk::prelude::*;
use gtk::Builder;
use std::sync::Mutex;
use gtk::CssProvider;
use std::rc::Rc;
use core::cell::RefCell;
use gtk::Label;
use crate::branch::list_branches;
use crate::utils::find_git_directory;
use crate::branch::git_branch_for_ui;
use crate::branch::create_new_branch;
use crate::gui::style::{apply_button_style, get_button, apply_window_style, load_and_get_window};
use crate::log::log;
use std::io;
use crate::log::Log;
use crate::log::accumulate_logs;
use crate::log::print_logs;

use super::style::apply_clone_button_style;
use super::style::apply_entry_style;
use super::style::apply_label_style;
use super::style::get_entry;
use super::style::get_label;


pub static mut OPEN_WINDOWS: Option<Mutex<Vec<gtk::Window>>> = None;

/// Runs the main window of a GTK application.
///
/// This function initializes and displays the main window of the application using a UI builder. It configures the window, adds buttons for actions such as "Clone" and "Init," and connects these buttons to their respective event handlers.
///
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

/// Closes all open GTK windows in a GTK application.
///
/// This function iterates through the list of open windows maintained by the application and closes each window. It ensures that all open windows are properly closed and their references are removed from the list.
///
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

/// Adds a GTK window to the list of open windows in a GTK application.
///
/// This function takes a reference to a GTK window (`window`) and adds it to the list of open windows maintained by the application. The list of open windows is managed using a mutex to ensure thread safety.
///
/// # Arguments
///
/// - `window`: A reference to the GTK window to be added to the list of open windows.
///
fn add_to_open_windows(window: &gtk::Window) {
    unsafe {
        if let Some(ref mutex) = OPEN_WINDOWS {
            let mut open_windows = mutex.lock().expect("Mutex lock failed");
            open_windows.push(window.clone());
        }
    }
}

/// Obtains text data from a function and returns it as a result.
///
/// This function invokes another function, `git_branch_for_ui`, to retrieve text data and returns it as a `Result`. If the data is obtained successfully, it is wrapped in a `Result::Ok`, and if an error occurs during the data retrieval, it is wrapped in a `Result::Err`.
///
/// # Returns
///
/// - `Ok(String)`: If the text data is successfully obtained, it contains the text.
/// - `Err(std::io::Error)`: If an error occurs during data retrieval, it contains the error information.
///
fn obtener_texto_desde_funcion() -> Result<String, std::io::Error> {
    match git_branch_for_ui(None) {
        Ok(result) => Ok(result),
        Err(err) => Err(err),
    }
}

fn obtener_texto_desde_log() -> Result<String, std::io::Error> {
    let mut current_dir = std::env::current_dir()?;
    let git_dir = match find_git_directory(&mut current_dir, ".git") {
        Some(git_dir) => git_dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Git directory not found\n",
            ))
        }
    };

    let log_iter=  log(None, &git_dir, 10, 0, true) ;
    let log_iter = log_iter.unwrap();
    let log_text = get_logs_as_string(log_iter);

        
    Ok(log_text)

}
pub fn get_logs_as_string(log_iter: impl Iterator<Item = Log>) -> String {
    let mut log_text = String::new();

    for log in log_iter {
        log_text.push_str(&log.to_string()); 
        log_text.push('\n'); 
    }

    log_text
}

/// Displays a repository window with various buttons and actions in a GTK application.
///
/// This function initializes and displays a GTK repository window using a UI builder. It configures the window, adds buttons with specific actions, and sets their styles and click event handlers. The repository window provides buttons for actions like "Add," "Commit," "Push," and more.
///
fn show_repository_window() {
    let builder = gtk::Builder::new();
    if let Some(new_window) = load_and_get_window(&builder,"src/gui/new_window2.ui", "window") {
        let new_window_clone = new_window.clone(); 
        let builder_clone = builder.clone();
        add_to_open_windows(&new_window);
        configure_repository_window(new_window);
        let button1 = get_button(&builder, "button1", "Add");
        let button2 = get_button(&builder, "button2", "Commit");
        let button3 = get_button(&builder, "button3", "Push");
        let button4 = get_button(&builder, "button4", "Push");
        let button5 = get_button(&builder, "button5", "Push");
        let button6 = get_button(&builder, "button6", "Push");
        let button7 = get_button(&builder, "button7", "Push");
        let button8 = get_button(&builder, "button8", "Push");
        let button9 = get_button(&builder, "button9", "Push");
        let button10 = get_button(&builder, "button10", "Push");
        let button11 = get_button(&builder, "button11", "Push");



        apply_button_style(&button1);
        apply_button_style(&button2);
        apply_button_style(&button3);
        apply_button_style(&button4);
        apply_button_style(&button5);
        apply_button_style(&button6);
        apply_button_style(&button7);
        apply_button_style(&button8);
        apply_button_style(&button9);
        apply_button_style(&button10);
        apply_button_style(&button11);

        
        button9.set_visible(false);
        button10.set_visible(false);

        button1.connect_clicked(move |_| {
            let label: Label = builder_clone.get_object("label").unwrap();
            let texto_desde_funcion = obtener_texto_desde_log();
            match texto_desde_funcion {
                Ok(texto) => {
                    let font_description = pango::FontDescription::from_string("Sans 2"); // Cambia "Serif 12" al tamaño y estilo de fuente deseado
                    label.override_font(&font_description);
                    label.set_hexpand(true);
                    label.set_halign(gtk::Align::Start);

                    label.set_ellipsize(pango::EllipsizeMode::End);
                    label.set_text(&texto);
                }
                Err(err) => {
                    eprintln!("Error al obtener el texto: {}", err);
                }
            }
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
            button9.set_visible(true);
            button10.set_visible(true);
            let builder_clone = builder.clone();


            
            button9.connect_clicked(move |_| {
                let label: Label = builder_clone.get_object("label").unwrap();
                let texto_desde_funcion = obtener_texto_desde_funcion();
                match texto_desde_funcion {
                    Ok(texto) => {
                        label.set_text(&texto);
                    }
                    Err(err) => {
                        eprintln!("Error al obtener el texto: {}", err);
                    }
                }            
            });
    
            button10.connect_clicked(move |_| {
                create_text_entry_window("Enter the name of the branch", |text| {
                    git_branch_for_ui(Some(text));                
                });
                
                
            });
    


           
        });

        button8.connect_clicked(move |_| {
            new_window_clone.close();
            run_main_window();
        });
    }
    
   
}

/// Configures the properties of a repository window in a GTK application.
///
/// This function takes a GTK window (`new_window`) as input and configures the repository window's properties, such as setting its default size and applying a specific window style, before displaying it.
///
/// # Arguments
///
/// - `new_window`: The GTK window to be configured as a repository window.
///
fn configure_repository_window(new_window: gtk::Window) {
    new_window.set_default_size(800, 600);
    apply_window_style(&new_window);
    new_window.show_all();
}

/// Creates a GTK text entry window for user input with a message and a callback function.
///
/// This function generates a new GTK window with a text entry field and an "OK" button. It allows users to input text and invokes a provided callback function when the "OK" button is clicked. The window can display a custom message as its title.
///
/// # Arguments
///
/// - `message`: A string message to be displayed as the window's title.
/// - `on_text_entered`: A callback function that takes a string parameter and is called when the user confirms the text input.
///
fn create_text_entry_window(message: &str, on_text_entered: impl Fn(String) + 'static) {
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
        close_all_windows();
        show_repository_window();
        on_text_entered(text); 
    });

    entry_window.show_all();
}

/// Connects a GTK button in an initialization window to specific actions based on its type.
///
/// This function takes a reference to a GTK button (`button`) and a button type (`button_type`) as input and connects a click event handler. The handler performs different actions based on the button's type, such as opening text entry dialogs, closing all windows, or showing a repository window.
///
/// # Arguments
///
/// - `button`: A reference to the GTK button to which the event handler will be connected.
/// - `button_type`: A string indicating the type of button, which determines the action to be taken when the button is clicked.
///
fn connect_button_clicked_init_window(button: &gtk::Button, button_type: &str) {
    let button_type = button_type.to_owned(); 

    button.connect_clicked(move |_| {
        if button_type == "option2" {
            create_text_entry_window("Enter the branch", |_| {
                
            });
        } else if button_type == "option3" {
            create_text_entry_window("Enter the template path", |_| {
                
            });
        } else if button_type == "option1" {
            close_all_windows();
            show_repository_window();
        }
    });
}

/// Configures the properties of a clone window in a GTK application.
///
/// This function takes a reference to a GTK window (`new_window_clone`) and a GTK builder (`builder`) as input and configures the clone window's properties, including adding it to the list of open windows, applying a specific window style, and setting its default size.
///
/// # Arguments
///
/// - `new_window_clone`: A reference to the GTK window to be configured.
/// - `builder`: A reference to the GTK builder used for UI construction.
///
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

/// Configures the properties of a clone window in a GTK application.
///
/// This function takes a reference to a GTK window (`new_window_clone`) and a GTK builder (`builder`) as input and configures the clone window's properties, including adding it to the list of open windows, applying a specific window style, and setting its default size.
///
/// # Arguments
///
/// - `new_window_clone`: A reference to the GTK window to be configured.
/// - `builder`: A reference to the GTK builder used for UI construction.
///
fn configure_clone_window(new_window_clone: &gtk::Window, builder: &gtk::Builder) {
    add_to_open_windows(new_window_clone);
    apply_window_style(new_window_clone);
    let browse_button = get_button(builder,"browse-button","Browse");
    let clone_button = get_button(builder,"clone-button","Clone the repo!");
    
    apply_clone_button_style(&browse_button);
    apply_clone_button_style(&clone_button);

     // Conectar la señal "clicked" del botón "Browse" para abrir el cuadro de diálogo de selección de directorio
     let new_window_clone_clone = new_window_clone.clone();
     let dir_to_clone_entry = get_entry(builder, "dir-to-clone-entry").unwrap(); // Asume que esto es un campo de entrada para mostrar el directorio seleccionado
     let dir_to_clone_entry_clone = dir_to_clone_entry.clone(); // Clonar la entrada para pasarlo a la función de manejo de clic
     browse_button.connect_clicked(move |_| {
         let dialog = FileChooserDialog::new(
             Some("Seleccionar Carpeta"),
             Some(&new_window_clone_clone),
             FileChooserAction::SelectFolder,
         );
 
         dialog.add_button("Cancelar", gtk::ResponseType::Cancel);
         dialog.add_button("Seleccionar", gtk::ResponseType::Ok);
 
         if dialog.run() == gtk::ResponseType::Ok {
             if let Some(folder) = dialog.get_filename() {
                 println!("Carpeta seleccionada: {:?}", folder);
                 // Actualiza la entrada de directorio con la carpeta seleccionada
                 dir_to_clone_entry_clone.set_text(&folder.to_string_lossy());
                 // Aquí puedes realizar acciones adicionales con la carpeta seleccionada
             }
         }
 
         unsafe { dialog.destroy() };
     });
     
    let url_label = get_label(builder, "url-label", 12.0).unwrap();
    let clone_dir_label = get_label(builder,"clone-dir-label",12.0).unwrap();
    let clone_info_label = get_label(builder, "clone-info-label", 26.0).unwrap();
    
    apply_label_style(&url_label);
    apply_label_style(&clone_dir_label);
    apply_label_style(&clone_info_label);

    let dir_to_clone_entry = get_entry(builder, "dir-to-clone-entry").unwrap();
    let url_entry = get_entry(builder, "url-entry").unwrap();

    apply_entry_style(&dir_to_clone_entry);
    apply_entry_style(&url_entry);
    new_window_clone.set_default_size(800, 600);

}

/// Connects a GTK button to a specific action.
///
/// This function takes a GTK button and a button type as input and sets an event handler for the "clicked" event of the button.
/// When the button is clicked, it performs a specific action based on the provided button type.
///
/// # Arguments
///
/// - `button`: A reference to the GTK button to which the event handler will be connected.
/// - `button_type`: A string indicating the button type, which determines the action to be taken when the button is clicked.
///
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








