use std::{collections::HashMap, io::{self, Write}, fs::{File, self}, thread, time::Duration};

use gtk::{
    prelude::BuilderExtManual, Button, ButtonExt, ComboBoxExt, ComboBoxText, ComboBoxTextExt,
    TextBufferExt, TextView, TextViewExt, WidgetExt,
};

use crate::{
    branch, commit, diff, merge,
    tree_handler::{self, Tree},
    utils::{self, obtain_git_dir}, hash_object, gui::style::{self, apply_button_style},
};

fn update_files_to_change(
    hash1: &str,
    hash2: &str,
    path: &str,
    files_to_change: &mut HashMap<String, String>,
) -> io::Result<()> {
    let git_dir = obtain_git_dir(".mgit")?;
    let diff = match diff::return_object_diff_string(hash1, hash2, &git_dir) {
        Ok(diff) => diff,
        Err(_e) => return Ok(()),
    };
    files_to_change.insert(path.to_string(), diff);
    Ok(())
}

fn update_combo_box_text(
    combo_box: &gtk::ComboBoxText,
    options: &std::collections::HashMap<String, String>,
) {
    println!("Updating como box text");
    
    combo_box.remove_all();

    for key in options.keys() {
        println!("Key is {:?}", key);
        combo_box.append_text(key);
    }
}

/*
let current_text = match buffer.get_text(&buffer.get_start_iter(), &buffer.get_end_iter(), false) {
            Some(text) => text,
            None => {
                return; //Acá no debería return pero no me deja compilar xd
            },
        };

        // Limpiar el contenido actual del TextView
        buffer.set_text("");

        // Insertar el contenido de diff en el TextView
        buffer.insert_at_cursor(diff);
         */

fn combo_box_connect_changed(
    combo_box: &gtk::ComboBoxText,
    text_view: &gtk::TextView,
    _files_to_change: &HashMap<String, String>,
) {
    let combo_box_cloned = combo_box.clone();
    let text_view_cloned = text_view.clone();
    combo_box.connect_changed(move |_| {
        if let Some(active_text) = combo_box_cloned.get_active_text() {
            let path = active_text.to_string();
            let path = format!("{}_temp",path);
            let diff = match fs::read_to_string(path) {
                Ok(content) => content,
                Err(_e) => {eprintln!("No se pudo obtener el diff"); return}
            };
            if let Some(buffer) = text_view_cloned.get_buffer() {
                buffer.set_text("");
                buffer.insert_at_cursor(&diff);
            } 
        }
    });
}

fn rebase_button_on_clicked(
    button: &Button,
    combo_box: &ComboBoxText,
    text_view: &TextView,
) {
    let combo_box_cloned = combo_box.clone();
    let text_view_cloned = text_view.clone();
    button.connect_clicked(move |_| {
        println!("Se tocó el botón");

        let path = match combo_box_cloned.get_active_text() {
            Some(path) => path,
            None => {return}
        };
        let path = format!("{}_temp", path);
        let text_buffer = match text_view_cloned.get_buffer() {
            Some(buff) => buff,
            None => {return}
        };
        let text = match text_buffer.get_text(&text_buffer.get_start_iter(), &text_buffer.get_end_iter(), false) {
            Some(text) => text.to_string(),
            None => {return}
        };
        println!("Path que se actualizará: {:?}", path);
        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(_e) => {eprintln!("No se pudo abrir el archivo."); return}
        };
        match file.write_all(text.as_bytes()) {
            Ok(_) => {},
            Err(_e) => {eprintln!("No se pudo actualizar el archivo"); return}
        }   
        match file.flush() {
            Ok(_) => {},
            Err(_e) => {eprintln!("No se pudo flushear el archivo");}
        } 
        println!("Se escribió el archivo");
    });
}

fn write_diffs_in_files(files_to_change : &HashMap<String, String>) -> io::Result<()> {
    for (path, diff) in files_to_change {
        println!("path is {:?}", path);
        let file_name = format!("{}_temp", path);
        println!("filename is {:?}", file_name);
        let mut file = File::create(file_name)?;
        file.write_all(diff.as_bytes())?;
        file.flush()?;
        println!("File written");
    }
    Ok(())
}

fn update_files_to_change_from_files(files_to_change : &HashMap<String, String>) -> io::Result<HashMap<String,String>> {
    let mut new_hash_map : HashMap<String, String> = HashMap::new();
    for (path, _) in files_to_change {
        let temp_path = format!("{}_temp", path);
        let content = fs::read_to_string(&temp_path)?;
        new_hash_map.insert(path.to_string(), content);
        fs::remove_file(temp_path)?;
    }
    Ok(new_hash_map)
}

fn update_rebase_tree(rebased_tree : &Tree, updated : &HashMap<String, String>) -> Tree {
    let mut rebased_new = rebased_tree.clone();
    for (path, hash) in updated {
        rebased_new.update_tree(path, hash);    
    }
    rebased_new
}

fn rebase_ok_all_button_on_clicked(button : &gtk::Button, our_commit: String, _parent_hash: &str, rebased_tree : &mut Tree, files_to_change : &mut HashMap<String, String>) -> io::Result<String> {
    //let mut new_commit_hash = String::new();
    let files_to_change_cloned: HashMap<String, String> = files_to_change.clone();
    let cloned_git_dir = obtain_git_dir(".mgit")?;
    let rebased_tree_cloned = rebased_tree.clone();
    button.connect_clicked(move |_| {
        let files_to_change_updated : HashMap<String,String> = match update_files_to_change_from_files(&files_to_change_cloned) {
            Ok(hash_map) => hash_map,
            Err(_e) => {eprintln!("Update error"); return}
        };
        let mut updated : HashMap<String, String> = HashMap::new();
        for (path, diff) in files_to_change_updated {
            let hash = match hash_object::store_string_to_file(&diff, &cloned_git_dir, "blob") {
                Ok(hash) => hash,
                Err(_e) => {eprintln!("No se pudo hashear"); return}
        
            };
            updated.insert(path, hash);            
        }

        let updated_tree = update_rebase_tree(&rebased_tree_cloned, &updated);
        let message = format!("Rebasing commit: {}", our_commit);
        let new_commit_hash = match commit::new_rebase_commit(&cloned_git_dir, &message, &our_commit, &updated_tree) {
            Ok(commit) => commit,
            Err(_e) => {eprintln!("No se pudo commitear"); return}
        };
        let temp_file_path = format!("{}/rebase_temp_file", &cloned_git_dir);
        let mut file = match File::create(temp_file_path) {
            Ok(file) => file,
            Err(_e) => {eprintln!("No se pudo guardar el new commit hash"); return}
        };
        match file.write_all(new_commit_hash.as_bytes()) {
            Ok(file) => file,
            Err(_e) => {eprintln!("No se pudo guardar el new commit hash"); return}
        };
        match file.flush() {
            Ok(file) => file,
            Err(_e) => {eprintln!("No se pudo guardar el new commit hash");}
        }
        
    });
    let cloned_git_dir = obtain_git_dir(".mgit")?;
    let temp_file_path = format!("{}/rebase_temp_file", &cloned_git_dir);
    let new_commit_hash = match fs::read_to_string(temp_file_path) {
        Ok(commit) => commit,
        Err(_) => {eprintln!("Error leyendo el commit.");  return Ok("ok".to_string())}
    };

    Ok(new_commit_hash)
}

/// Grabs the hashes from the common ancestor, our branch and their branch. Creates a new commit applied to the common ancestor, and then rebase the commits from our branch on top of the new commit.
pub fn create_rebasing_commit(
    builder: &gtk::Builder,
    our_commit: &str,
    rebased_commit: &str,
    common_ancestor: &str,
    git_dir: &str,
    parent_hash: &str,
) -> io::Result<String> {
    let our_tree = tree_handler::load_tree_from_commit(our_commit, git_dir)?;
    let ancestor_tree = tree_handler::load_tree_from_commit(common_ancestor, git_dir)?;
    let mut rebased_tree = tree_handler::load_tree_from_commit(rebased_commit, git_dir)?;

    // Get the paths of the files that haven't been modified between the common ancestor and the rebased commit.
    let files_without_changes_in_rebased: HashMap<String, String> =
        tree_handler::get_files_without_changes(&ancestor_tree, &rebased_tree)
            .into_iter()
            .collect();
    let files_changed_this_commit = tree_handler::get_files_with_changes(&ancestor_tree, &our_tree);
    for (a, b) in &files_changed_this_commit {
        println!("Files changed: ({},{})", a, b);
    }
    for (a, b) in &files_without_changes_in_rebased {
        println!("Files withouth changed: ({},{})", a, b);
    }
    let mut files_to_change: HashMap<String, String> = HashMap::new();
    let combo_box = match builder.get_object::<ComboBoxText>("rebase-text-list") {
        Some(combo_box) => combo_box,
        None => {
            println!("No se pudo encontrar el ComboBoxText con ID rebase-text-list");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el ComboBoxText con ID rebase-text-list",
            ));
        }
    };
    println!("Obtained combo box");
    let text_view = match builder.get_object::<TextView>("rebase-view") {
        Some(combo_box) => combo_box,
        None => {
            println!("No se pudo encontrar el TextView con ID rebase-view");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el TextView con ID rebase-view",
            ));
        }
    };
    println!("Obtained text view");

    let button = match builder.get_object::<Button>("rebase-button") {
        Some(combo_box) => combo_box,
        None => {
            println!("No se pudo encontrar el botón con ID rebase-button");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el botón con ID rebase-button",
            ));
        }
    };

    apply_button_style(&button);

    println!("Obtained update button");
    let ok_all_button: Button = match builder.get_object::<Button>("rebase-ok-all-button") {
        Some(combo_box) => combo_box,
        None => {
            println!("No se pudo encontrar el botón con ID rebase-ok-all-button");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el botón con ID rebase-ok-all-button",
            ));
        }
    };
    
    apply_button_style(&ok_all_button);

    println!("Obtained ok all button");
    // For each file changed this commit, we should check if it wasn't changed between the ancestor and rebase.
    // If so, we should simply update the hash.
    for (path, hash) in files_changed_this_commit {
        println!("Estoy en el for");
        if files_without_changes_in_rebased.contains_key(&path) {
            println!("Entré al if");
            rebased_tree.update_tree(&path, &hash);
            println!("Updated!");
        } else {
            println!("Entré al else");
            let hash2 = match rebased_tree.get_hash_from_path(&path) {
                Some(hash) => hash,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "No se pudo obtener el hash del path del rebased tree",
                    ));
                }
            };
            println!("Obtuve el hash {}", hash2);
            update_files_to_change(&hash, &hash2, &path, &mut files_to_change)?;
            println!("Updated files to change");
        }
    }
    for (path, diff) in &files_to_change {
        println!("Files to change path : {} and diff : {}", path, diff);
    }
    update_combo_box_text(&combo_box, &files_to_change);
    write_diffs_in_files(&files_to_change)?;
    combo_box_connect_changed(&combo_box, &text_view, &files_to_change);
    rebase_button_on_clicked(&button, &combo_box, &text_view);
    let mut new_commit_hash = rebase_ok_all_button_on_clicked(&ok_all_button, our_commit.to_string(), parent_hash, &mut rebased_tree, &mut files_to_change)?;
     // Esperar hasta que new_commit_hash sea distinto de "ok"
    while new_commit_hash == "ok" {
        thread::sleep(Duration::from_secs(1)); // Puedes ajustar el tiempo de espera según tus necesidades
        new_commit_hash = rebase_ok_all_button_on_clicked(
            &ok_all_button,
            our_commit.to_string(),
            parent_hash,
            &mut rebased_tree,
            &mut files_to_change,
        )?;
    }
    //let new_commit_hash = String::from("ok");
    println!("New commit hash is {:?}", new_commit_hash);
    Ok(new_commit_hash)
}

pub fn rebase(
    builder: &gtk::Builder,
    our_branch: &str,
    their_branch: &str,
    git_dir: &str,
) -> io::Result<()> {
    let our_branch_hash = branch::get_branch_commit_hash(our_branch, git_dir)?;
    println!("Our branch hash {:?}", our_branch_hash);
    let their_branch_hash = branch::get_branch_commit_hash(their_branch, git_dir)?;
    println!("Their branch hash {:?}", their_branch_hash);
    let common_commit_ancestor =
        merge::find_common_ancestor(&our_branch_hash, &their_branch_hash, git_dir)?;
    println!("Common commit ancestor {}", common_commit_ancestor);
    let mut our_branch_commits = utils::get_branch_commit_history_until(
        &our_branch_hash,
        git_dir,
        &common_commit_ancestor,
    )?;
    println!("Our branch commits:");
    println!("{:?}", our_branch_commits);
    our_branch_commits.reverse();

    let mut our_new_branch_hash = their_branch_hash.clone();
    println!("Our new branch hash {:?}", our_new_branch_hash);
    while let Some(commit_hash) = our_branch_commits.pop() {
        our_new_branch_hash = create_rebasing_commit(
            builder,
            &commit_hash,
            &their_branch_hash,
            &common_commit_ancestor,
            git_dir,
            &our_new_branch_hash,
        )?;
    }

    Ok(())
}

