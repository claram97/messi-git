use gtk::{
    prelude::{BuilderExtManual, ComboBoxExtManual},
    ButtonExt, ComboBoxExt, ComboBoxTextExt, TextBufferExt, TextView, TextViewExt, WidgetExt,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{self, Write},
    path::Path,
    rc::Rc,
};

use crate::{
    branch::{self, get_current_branch_path},
    checkout,
    commit::{self, get_branch_name},
    diff,
    gui::style::{get_button, show_message_dialog},
    hash_object, merge, tree_handler,
    utils::{self, obtain_git_dir},
};

fn files_that_changed_between_commits(
    commit1: &str,
    commit2: &str,
    git_dir: &str,
) -> io::Result<Vec<(String, String)>> {
    let commit1_tree = tree_handler::load_tree_from_commit(commit1, git_dir)?;
    let commit2_tree = tree_handler::load_tree_from_commit(commit2, git_dir)?;
    let changed_files = tree_handler::get_files_with_changes(&commit1_tree, &commit2_tree);
    Ok(changed_files)
}

pub enum RebaseState {
    RebaseStepInProgress,
    RebaseStepFinished,
    RebaseFinished,
}

#[derive(Debug, Clone)]
pub struct Rebase {
    our_commits: Vec<String>,
    active_commit: String,
    commit_to_rebase: String,
    original_our_branch_hash: String,
    rebase_step: RebaseStep,
}

#[derive(Debug, Clone)]
struct RebaseStep {
    diffs: HashMap<String, String>,
}

fn obtain_text_view_from_builder(builder: &gtk::Builder) -> io::Result<gtk::TextView> {
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
    Ok(text_view)
}

fn obtain_combo_box_from_builder(builder: &gtk::Builder) -> io::Result<gtk::ComboBoxText> {
    let combo_box = match builder.get_object::<gtk::ComboBoxText>("rebase-text-list") {
        Some(combo_box) => combo_box,
        None => {
            println!("No se pudo encontrar el ComboBoxText con ID rebase-text-list");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el ComboBoxText con ID rebase-text-list",
            ));
        }
    };
    Ok(combo_box)
}

fn obtain_update_button_from_builder(builder: &gtk::Builder) -> io::Result<gtk::Button> {
    let button = match builder.get_object::<gtk::Button>("rebase-button") {
        Some(button) => button,
        None => {
            println!("No se pudo encontrar el Button con ID rebase-button");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el Button con ID rebase-button",
            ));
        }
    };
    Ok(button)
}

fn obtain_rebase_button_from_builder(builder: &gtk::Builder) -> io::Result<gtk::Button> {
    let button = match builder.get_object::<gtk::Button>("make-rebase-button") {
        Some(button) => button,
        None => {
            println!("No se pudo encontrar el Button con ID make-rebase-button");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el Button con ID make-rebase-button",
            ));
        }
    };
    Ok(button)
}

fn obtain_ok_all_button_from_builder(builder: &gtk::Builder) -> io::Result<gtk::Button> {
    let button = match builder.get_object::<gtk::Button>("rebase-ok-all-button") {
        Some(button) => button,
        None => {
            println!("No se pudo encontrar el Button con ID rebase-ok-all-button");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el Button con ID rebase-ok-all-button",
            ));
        }
    };
    Ok(button)
}

fn obtain_abort_button_from_builder(builder: &gtk::Builder) -> io::Result<gtk::Button> {
    let button = match builder.get_object::<gtk::Button>("abort-rebase-button") {
        Some(button) => button,
        None => {
            println!("No se pudo encontrar el Button con ID abort-rebase-button");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el Button con ID abort-rebase-button",
            ));
        }
    };
    Ok(button)
}

fn load_file_diffs(
    commit_to_rebase: &str,
    active_commit: &str,
    git_dir: &str,
) -> io::Result<HashMap<String, String>> {
    let tree_to_rebase = tree_handler::load_tree_from_commit(commit_to_rebase, git_dir)?;
    let tree_active_commit = tree_handler::load_tree_from_commit(active_commit, git_dir)?;

    let changed_files = tree_handler::get_files_with_changes(&tree_to_rebase, &tree_active_commit);
    let mut diffs: HashMap<String, String> = HashMap::new();
    for (file, _) in &changed_files {
        let hash_active_commit = match tree_active_commit.get_hash_from_path(&file) {
            Some(hash) => hash,
            None => {
                println!("Couldn't obtain hash for file {}", file);
                continue;
            }
        };
        let hash_to_rebase = match tree_to_rebase.get_hash_from_path(&file) {
            Some(hash) => hash,
            None => {
                println!("Couldn't obtain hash for file {}", file);
                continue;
            }
        };
        let diff = diff::return_object_diff_string(&hash_active_commit, &hash_to_rebase, &git_dir);
        match diff {
            Ok(diff) => {
                diffs.insert(file.clone(), diff);
            }
            Err(_) => {
                println!("No se pudo obtener el diff para el archivo {}", file);
            }
        }
    }
    Ok(diffs)
}

fn abort_rebase_button_on_click(
    builder: &gtk::Builder,
    original_our_branch_hash: String,
) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;
    let current_branch_path = get_current_branch_path(&git_dir)?;
    let complete_branch_path = format!("{}/{}", git_dir, current_branch_path);
    println!("complete_branch_path: {}", complete_branch_path);
    let branch_name = get_branch_name(&git_dir)?;
    let root_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el working dir",
            ));
        }
    };
    println!("root_dir: {}", root_dir);
    let mut file = match File::create(&complete_branch_path) {
        Ok(file) => file,
        Err(_error) => {
            eprintln!("Error creating file");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo crear el archivo",
            ));
        }
    };
    println!("File: {:?}", file);
    match file.write_all(original_our_branch_hash.as_bytes()) {
        Ok(_) => {}
        Err(_error) => {
            println!("Error writing to file");
        }
    }

    let git_dir_path = Path::new(&git_dir);
    match checkout::checkout_branch(git_dir_path, &root_dir, &branch_name) {
        Ok(_) => {
            println!("Checkout to branch {} completed", branch_name);
        }
        Err(_e) => {
            println!("Error checking out to branch {}", branch_name);
        }
    }

    // Set the text view to notify the user that the rebase was aborted
    let text_view = obtain_text_view_from_builder(&builder)?;
    match text_view.get_buffer() {
        Some(buffer) => {
            buffer.set_text("Rebase abortado");
        }
        None => {
            println!("No se pudo obtener el buffer del TextView");
        }
    };

    let rebase_button = obtain_rebase_button_from_builder(&builder)?;
    rebase_button.set_sensitive(true);
    let ok_button = obtain_ok_all_button_from_builder(&builder)?;
    ok_button.set_sensitive(false);
    let abort_button = obtain_abort_button_from_builder(&builder)?;
    abort_button.set_sensitive(false);
    let combo_box = obtain_combo_box_from_builder(&builder)?;
    combo_box.set_sensitive(false);
    let update_button = obtain_update_button_from_builder(&builder)?;
    update_button.set_sensitive(false);

    Ok(())
}

pub fn write_rebase_step_into_gui(
    builder: &gtk::Builder,
    rebase: Rc<RefCell<Rebase>>,
    git_dir: &str,
) -> io::Result<()> {
    let text_view = obtain_text_view_from_builder(builder)?;
    let combo_box = obtain_combo_box_from_builder(builder)?;
    let text_buffer = match text_view.get_buffer() {
        Some(text_buffer) => text_buffer,
        None => {
            println!("No se pudo obtener el buffer del TextView");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo obtener el buffer del TextView",
            ));
        }
    };
    let update_button = obtain_update_button_from_builder(builder)?;
    let ok_button = obtain_ok_all_button_from_builder(builder)?;
    let abort_button = obtain_abort_button_from_builder(builder)?;

    // Set buttons
    let rebase_button = obtain_rebase_button_from_builder(builder)?;
    rebase_button.set_sensitive(false);
    ok_button.set_sensitive(true);
    abort_button.set_sensitive(true);
    combo_box.set_sensitive(true);
    update_button.set_sensitive(true);

    text_buffer.set_text("");
    combo_box.remove_all();

    let diffs = match load_file_diffs(
        &rebase.borrow().commit_to_rebase,
        &rebase.borrow().active_commit,
        git_dir,
    ) {
        Ok(diffs) => diffs,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo obtener los diffs para el rebase",
            ));
        }
    };

    let changed_files = diffs.keys().cloned().collect::<Vec<String>>();

    if changed_files.len() == 0 {
        text_buffer.set_text(
            "No hay problemas con los archivos\n Presione Ok para continuar al siguiente commit",
        );
        return Ok(());
    }

    {
        let mut rebase_step = rebase.borrow_mut();
        rebase_step.rebase_step.diffs = diffs;
    }

    for file in changed_files {
        combo_box.append_text(&file);
    }

    combo_box.set_active(Some(0));
    let file = match combo_box.get_active_text() {
        Some(file) => file.to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el archivo seleccionado",
            ));
        }
    };
    let diff = match rebase.borrow().rebase_step.diffs.get(&file) {
        Some(diff) => diff.clone(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el diff para el archivo",
            ));
        }
    };
    text_buffer.set_text(&diff);

    let rebase_step_clone = Rc::clone(&rebase);
    let text_buffer_clone = text_buffer.clone();
    combo_box.connect_changed(move |combo_box| {
        let file = match combo_box.get_active_text() {
            Some(file) => file.to_string(),
            None => {
                return;
            }
        };
        let diff = match rebase_step_clone.borrow().rebase_step.diffs.get(&file) {
            Some(diff) => diff.clone(),
            None => {
                return;
            }
        };
        text_buffer_clone.set_text(&diff);
    });

    let text_buffer_clone = text_buffer.clone();
    let rebase_step_clone = Rc::clone(&rebase);
    update_button.connect_clicked(move |_| {
        let text = match text_buffer_clone.get_text(
            &text_buffer_clone.get_start_iter(),
            &text_buffer_clone.get_end_iter(),
            false,
        ) {
            Some(text) => text.to_string(),
            None => {
                return;
            }
        };
        let file = match combo_box.get_active_text() {
            Some(file) => file.to_string(),
            None => {
                return;
            }
        };

        {
            let mut rebase_step = rebase_step_clone.borrow_mut();
            rebase_step.rebase_step.diffs.insert(file, text);
        }
    });

    let rebase_step_clone = Rc::clone(&rebase);
    let builder_clone = builder.clone();
    abort_button.connect_clicked(move |_| {
        let rebase_step = rebase_step_clone.borrow();
        let original_our_branch_hash = rebase_step.original_our_branch_hash.clone();
        abort_rebase_button_on_click(&builder_clone, original_our_branch_hash).unwrap();
    });

    let builder_clone = builder.clone();
    let git_dir_clone = git_dir.to_string().clone();
    let rebase_rf = Rc::clone(&rebase);
    ok_button.connect_clicked(move |_| {
        let rebase_clone = rebase_rf.borrow().clone();
        match next_rebase_iteration(&builder_clone, rebase_clone, &git_dir_clone) {
            Ok(_) => {}
            Err(_) => {
                return ();
            }
        }
    });

    Ok(())
}

pub fn next_rebase_iteration(
    builder: &gtk::Builder,
    rebase: Rebase,
    git_dir: &str,
) -> io::Result<()> {
    let commit_to_rebase = &rebase.commit_to_rebase;
    let tree_to_rebase = tree_handler::load_tree_from_commit(commit_to_rebase, git_dir)?;

    let mut tree_with_changes = tree_to_rebase.clone();
    let rebase_step = &rebase.rebase_step;
    for (file, diff) in &rebase_step.diffs {
        let hash = hash_object::store_string_to_file(&diff, &git_dir, "blob")?;
        tree_with_changes.update_tree(&file, &hash)
    }

    let commit_message = format!("Rebase commit {}", rebase.active_commit);
    let new_commit_hash: String = commit::new_rebase_commit(
        git_dir,
        &commit_message,
        &rebase.commit_to_rebase,
        &tree_with_changes,
    )?;

    let text_view = obtain_text_view_from_builder(builder)?;

    let mut new_rebase = rebase;
    println!("{:#?}", new_rebase);
    match new_rebase.our_commits.pop() {
        Some(commit) => {
            new_rebase.active_commit = commit;
            new_rebase.commit_to_rebase = new_commit_hash;
            new_rebase.rebase_step.diffs = HashMap::new();
            write_rebase_step_into_gui(builder, Rc::new(RefCell::new(new_rebase)), git_dir)?;
        }
        None => {
            match text_view.get_buffer() {
                Some(buffer) => {
                    let branch_name = get_branch_name(&git_dir)?;
                    let root_dir = match Path::new(&git_dir).parent() {
                        Some(dir) => dir.to_string_lossy().to_string(),
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::NotFound,
                                "No se pudo encontrar el working dir",
                            ));
                        }
                    };
                    let git_dir_path = Path::new(&git_dir);
                    checkout::checkout_branch(git_dir_path, &root_dir, &branch_name)?;
                    buffer.set_text("Rebase finalizado");
                }
                None => {
                    println!("No se pudo obtener el buffer del TextView");
                }
            };
            let combo_box = obtain_combo_box_from_builder(builder)?;
            let update_button = obtain_update_button_from_builder(builder)?;
            let ok_button = obtain_ok_all_button_from_builder(builder)?;
            combo_box.set_sensitive(false);
            update_button.set_sensitive(false);
            ok_button.set_sensitive(false);

            let rebase_button = obtain_rebase_button_from_builder(builder)?;
            rebase_button.set_sensitive(true);

            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No hay más commits para rebase",
            ));
        }
    };
    Ok(())
}

pub fn start_rebase_gui(
    git_dir: &str,
    our_branch: &str,
    branch_to_rebase: &str,
) -> io::Result<Rc<RefCell<Rebase>>> {
    let our_branch_hash = branch::get_branch_commit_hash(&our_branch, git_dir)?;
    let their_branch_hash = branch::get_branch_commit_hash(&branch_to_rebase, git_dir)?;
    let common_ancestor =
        merge::find_common_ancestor(&our_branch_hash, &their_branch_hash, git_dir)?;

    let mut our_commits =
        utils::get_branch_commit_history_until(&our_branch_hash, git_dir, &common_ancestor)?;
    let active_commit = match our_commits.pop() {
        Some(commit) => commit,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No hay commits con los que hacer rebase",
            ));
        }
    };

    let files_with_changes =
        files_that_changed_between_commits(&our_branch_hash, &their_branch_hash, git_dir)?;
    if files_with_changes.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No hay cambios entre los commits",
        ));
    }

    let rebase = Rebase {
        our_commits,
        active_commit,
        commit_to_rebase: their_branch_hash.clone(),
        original_our_branch_hash: our_branch_hash,
        rebase_step: RebaseStep {
            diffs: HashMap::new(),
        },
    };
    Ok(Rc::new(RefCell::new(rebase)))
}
