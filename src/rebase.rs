use std::{collections::HashMap, io};

use gtk::{
    prelude::BuilderExtManual, Button, ButtonExt, ComboBoxExt, ComboBoxText, ComboBoxTextExt,
    TextBufferExt, TextView, TextViewExt,
};

use crate::{
    branch, commit, diff, merge,
    tree_handler::{self, Tree},
    utils::{self, obtain_git_dir},
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
    builder: &gtk::Builder,
    combo_box: &gtk::ComboBoxText,
    options: &std::collections::HashMap<String, String>,
) {
    combo_box.remove_all();

    for (key, _value) in options {
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
    files_to_change: &HashMap<String, String>,
) {
    let combo_box_cloned = combo_box.clone();
    let text_view_cloned = text_view.clone();
    let files_to_change_clone = files_to_change.clone();
    combo_box.connect_changed(move |_| {
        // Acción a realizar cuando el usuario elige una opción
        if let Some(active_text) = combo_box_cloned.get_active_text() {
            let path = active_text.to_string();
            let diff = match files_to_change_clone.get(&path) {
                Some(diff) => diff,
                None => return,
            };
            if let Some(buffer) = text_view_cloned.get_buffer() {
                // Obtener el contenido actual del TextView y guardarlo en una variable
                // Limpiar el contenido actual del TextView
                buffer.set_text("");

                // Insertar el contenido de diff en el TextView
                buffer.insert_at_cursor(diff);

                // Hacer algo con la variable current_text
            } // Aquí puedes realizar la acción que desees con la opción seleccionada
        }
    });
}

fn rebase_button_on_clicked(
    button: &Button,
    combo_box: &ComboBoxText,
    text_view: &TextView,
    files_to_change: &mut HashMap<String, String>,
) {
    // let path = match combo_box.get_active_text() {
    //     Some(path) => &path.to_string(),
    //     None => {return}
    // };
    // let text_buffer = match text_view.get_buffer() {
    //     Some(buff) => buff,
    //     None => {return}
    // };
    // let text = match text_buffer.get_text(&text_buffer.get_start_iter(), &text_buffer.get_end_iter(), false) {
    //     Some(text) => &text.to_string(),
    //     None => {return}
    // };

    // let text_view_cloned = text_view.clone();
    // let mut files_to_change_clone = files_to_change.clone();
    // button.connect_clicked(move |_| {
    //     files_to_change_clone.remove(&(path.to_string()));
    //     files_to_change_clone.insert(path.to_string(), text.to_string());
    // });
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
    let mut files_to_change: HashMap<String, String> = HashMap::new();
    let combo_box = match builder.get_object::<ComboBoxText>("rebase-text-list") {
        Some(combo_box) => combo_box,
        None => {
            println!("No se pudo encontrar el ComboBoxText con ID rebase-text-list");
            return Ok("err".to_string()); //Devolver error acá
        }
    };
    let text_view = match builder.get_object::<TextView>("rebase-view") {
        Some(combo_box) => combo_box,
        None => {
            println!("No se pudo encontrar el TextView con ID rebase-view");
            return Ok("err".to_string()); //Devolver error acá
        }
    };

    let button = match builder.get_object::<Button>("rebase-button") {
        Some(combo_box) => combo_box,
        None => {
            println!("No se pudo encontrar el TextView con ID rebase-button");
            return Ok("err".to_string()); //Devolver error acá
        }
    };

    // For each file changed this commit, we should check if it wasn't changed between the ancestor and rebase.
    // If so, we should simply update the hash.
    for (hash, path) in files_changed_this_commit {
        if files_without_changes_in_rebased.contains_key(&path) {
            rebased_tree.update_tree(&path, &hash);
        } else {
            let hash2 = match rebased_tree.get_hash_from_path(&path) {
                Some(hash) => hash,
                None => {
                    return Ok("ok".to_string());
                    //Acá en realidad voy a devolver error
                }
            };
            update_files_to_change(&hash, &hash2, &path, &mut files_to_change)?;
            update_combo_box_text(builder, &combo_box, &files_to_change);
            combo_box_connect_changed(&combo_box, &text_view, &files_to_change);
            rebase_button_on_clicked(&button, &combo_box, &text_view, &mut files_to_change);
        }
    }

    let message = format!("Rebasing commit: {our_commit}");
    let new_commit_hash = commit::new_rebase_commit(git_dir, &message, parent_hash, &rebased_tree)?;

    Ok(new_commit_hash)
}

pub fn rebase(
    builder: &gtk::Builder,
    our_branch: &str,
    their_branch: &str,
    git_dir: &str,
) -> io::Result<()> {
    let our_branch_hash = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_branch_hash = branch::get_branch_commit_hash(their_branch, git_dir)?;

    let common_commit_ancestor =
        merge::find_common_ancestor(&our_branch_hash, &their_branch_hash, &git_dir)?;
    let mut our_branch_commits = utils::get_branch_commit_history_until(
        &our_branch_hash,
        &git_dir,
        &common_commit_ancestor,
    )?;
    our_branch_commits.reverse();

    let mut our_new_branch_hash = their_branch_hash.clone();

    while let Some(commit_hash) = our_branch_commits.pop() {
        our_new_branch_hash = create_rebasing_commit(
            builder,
            &commit_hash,
            &their_branch_hash,
            &common_commit_ancestor,
            &git_dir,
            &our_new_branch_hash,
        )?;
    }

    Ok(())
}
