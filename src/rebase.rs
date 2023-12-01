use gtk::{
    prelude::{BuilderExtManual, ComboBoxExtManual},
    ButtonExt, ComboBoxExt, ComboBoxTextExt, TextBufferExt, TextViewExt, WidgetExt,
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
    gui::style,
    hash_object, merge, tree_handler,
    utils::{self, obtain_git_dir},
};

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

// This function will return the intersection between the files that changed between commit1 and commit2 and the files that changed between commit1 and commit3
fn intersection_files_that_changed_between_commits(
    commit1: &str,
    commit2: &str,
    commit3: &str,
    git_dir: &str,
) -> io::Result<Vec<String>> {
    let commit1_tree = tree_handler::load_tree_from_commit(commit1, git_dir)?;
    let commit2_tree = tree_handler::load_tree_from_commit(commit2, git_dir)?;
    let commit3_tree = tree_handler::load_tree_from_commit(commit3, git_dir)?;
    let changed_files1 = tree_handler::get_files_with_changes(&commit2_tree, &commit1_tree);
    let changed_files2 = tree_handler::get_files_with_changes(&commit3_tree, &commit1_tree);
    let mut intersection: Vec<String> = Vec::new();
    for (file1, _) in &changed_files1 {
        for (file2, _) in &changed_files2 {
            if file1 == file2 {
                intersection.push(file1.clone());
            }
        }
    }
    Ok(intersection)
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
        let hash_active_commit = match tree_active_commit.get_hash_from_path(file) {
            Some(hash) => hash,
            None => {
                eprintln!("Couldn't obtain hash for file {}", file);
                continue;
            }
        };
        let hash_to_rebase = match tree_to_rebase.get_hash_from_path(file) {
            Some(hash) => hash,
            None => {
                eprintln!("Couldn't obtain hash for file {}", file);
                continue;
            }
        };
        let diff = diff::return_object_diff_string(&hash_active_commit, &hash_to_rebase, git_dir);
        match diff {
            Ok(diff) => {
                diffs.insert(file.clone(), diff);
            }
            Err(_) => {
                eprintln!("No se pudo obtener el diff para el archivo {}", file);
            }
        }
    }
    Ok(diffs)
}

fn get_root_dir(git_dir: &str) -> io::Result<String> {
    let root_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el working dir",
            ));
        }
    };
    Ok(root_dir)
}

// Write the given hash into the refs/heads/branch_name file pointed by the HEAD file
fn write_hash_into_branch_file(hash: &str, git_dir: &str) -> io::Result<()> {
    let branch_path = get_current_branch_path(git_dir)?;
    let branch_path = format!("{}/{}", git_dir, branch_path);
    let mut file = match File::create(branch_path) {
        Ok(file) => file,
        Err(_error) => {
            eprintln!("Error creating file");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo crear el archivo",
            ));
        }
    };
    match file.write_all(hash.as_bytes()) {
        Ok(_) => {}
        Err(_error) => {
            println!("Error writing to file");
        }
    }
    Ok(())
}

// Given a message, write it into the text view
fn write_message_into_text_view(builder: &gtk::Builder, message: &str) -> io::Result<()> {
    let text_view = match style::get_text_view(builder, "rebase-view") {
        Some(text_view) => text_view,
        None => {
            println!("No se pudo obtener el TextView");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo obtener el TextView",
            ));
        }
    };
    match text_view.get_buffer() {
        Some(buffer) => {
            buffer.set_text(message);
        }
        None => {
            println!("No se pudo obtener el buffer del TextView");
        }
    };
    Ok(())
}

fn get_text_view_content(builder: &gtk::Builder) -> io::Result<String> {
    let text_view = match style::get_text_view(builder, "rebase-view") {
        Some(text_view) => text_view,
        None => {
            println!("No se pudo obtener el TextView");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo obtener el TextView",
            ));
        }
    };
    let text_buffer = match text_view.get_buffer() {
        Some(buffer) => buffer,
        None => {
            println!("No se pudo obtener el buffer del TextView");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo obtener el buffer del TextView",
            ));
        }
    };
    let text = match text_buffer.get_text(
        &text_buffer.get_start_iter(),
        &text_buffer.get_end_iter(),
        false,
    ) {
        Some(text) => text.to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo obtener el texto del TextView",
            ));
        }
    };
    Ok(text)
}

fn abort_rebase_button_on_click(
    builder: &gtk::Builder,
    original_our_branch_hash: String,
) -> io::Result<()> {
    let git_dir = obtain_git_dir()?;
    let branch_name = get_branch_name(&git_dir)?;
    let root_dir = get_root_dir(&git_dir)?;
    match write_hash_into_branch_file(&original_our_branch_hash, &git_dir) {
        Ok(_) => {}
        Err(_e) => {
            println!("Error writing to branch file");
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

    write_message_into_text_view(builder, "Rebase abortado")?;

    let rebase_button = style::get_button(builder, "make-rebase-button");
    rebase_button.set_sensitive(true);
    let ok_button = style::get_button(builder, "rebase-ok-all-button");
    ok_button.set_sensitive(false);
    let abort_button = style::get_button(builder, "abort-rebase-button");
    abort_button.set_sensitive(false);
    let combo_box = obtain_combo_box_from_builder(builder)?;
    combo_box.set_sensitive(false);
    let update_button = style::get_button(builder, "rebase-button");
    update_button.set_sensitive(false);
    Ok(())
}

// Recieves a builder and a diff and writes the combo box and text view with the diff
fn write_combo_box_and_view(
    builder: &gtk::Builder,
    diff: HashMap<String, String>,
) -> io::Result<()> {
    let changed_files = diff.keys().cloned().collect::<Vec<String>>();
    let combo_box = obtain_combo_box_from_builder(builder)?;
    if changed_files.is_empty() {
        write_message_into_text_view(
            builder,
            "No hay problemas con los archivos\n Presione Ok para continuar al siguiente commit",
        )?;
        return Ok(());
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

    let file_text = match diff.get(&file) {
        Some(diff) => diff.clone(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el diff para el archivo",
            ));
        }
    };
    write_message_into_text_view(builder, &file_text)?;
    Ok(())
}

fn combo_box_on_change(builder: &gtk::Builder, diff: HashMap<String, String>) -> io::Result<()> {
    let combo_box = obtain_combo_box_from_builder(builder)?;
    let file = match combo_box.get_active_text() {
        Some(file) => file.to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el archivo seleccionado",
            ));
        }
    };

    let file_text = match diff.get(&file) {
        Some(diff) => diff.clone(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el diff para el archivo",
            ));
        }
    };
    write_message_into_text_view(builder, &file_text)?;
    Ok(())
}

fn update_button_on_click(builder: &gtk::Builder, rebase: Rc<RefCell<Rebase>>) -> io::Result<()> {
    let text = match get_text_view_content(builder) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error getting text from TextView: {}", e);
            return Err(e);
        }
    };
    let file = match obtain_combo_box_from_builder(builder)
        .unwrap()
        .get_active_text()
    {
        Some(file) => file.to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el archivo seleccionado",
            ));
        }
    };

    {
        let mut rebase_step = rebase.borrow_mut();
        rebase_step.rebase_step.diffs.insert(file, text);
    }
    Ok(())
}

fn load_and_write_diffs(
    builder: &gtk::Builder,
    rebase: Rc<RefCell<Rebase>>,
    git_dir: &str,
) -> io::Result<()> {
    let commit_to_rebase = &rebase.borrow().commit_to_rebase.clone();
    let active_commit = &rebase.borrow().active_commit.clone();
    let diffs = load_file_diffs(commit_to_rebase, active_commit, git_dir)?;
    write_combo_box_and_view(builder, diffs.clone())?;
    let mut rebase_step = rebase.borrow_mut();
    rebase_step.rebase_step.diffs = diffs;
    Ok(())
}

fn setup_buttons(builder: &gtk::Builder, rebase: Rc<RefCell<Rebase>>, git_dir: &str) {
    let update_button = style::get_button(builder, "rebase-button");
    update_button.set_sensitive(true);
    let rebase_step_clone = Rc::clone(&rebase);
    let builder_clone = builder.clone();
    update_button.connect_clicked(move |_| {
        let result = update_button_on_click(&builder_clone, Rc::clone(&rebase_step_clone));
        eprintln!("{:#?}", result);
    });

    let abort_button = style::get_button(builder, "abort-rebase-button");
    abort_button.set_sensitive(true);
    let rebase_step_clone = Rc::clone(&rebase);
    let builder_clone = builder.clone();
    abort_button.connect_clicked(move |_| {
        let rebase_step = rebase_step_clone.borrow();
        let original_our_branch_hash = rebase_step.original_our_branch_hash.clone();
        abort_rebase_button_on_click(&builder_clone, original_our_branch_hash).unwrap();
    });

    let ok_button = style::get_button(builder, "rebase-ok-all-button");
    ok_button.set_sensitive(true);
    let builder_clone = builder.clone();
    let git_dir_clone = git_dir.to_string().clone();
    let rebase_rf = Rc::clone(&rebase);
    ok_button.connect_clicked(move |_| {
        let rebase_clone = rebase_rf.borrow().clone();
        let _ = next_rebase_iteration(&builder_clone, rebase_clone, &git_dir_clone);
    });
}

pub fn write_rebase_step_into_gui(
    builder: &gtk::Builder,
    rebase: Rc<RefCell<Rebase>>,
    git_dir: &str,
) -> io::Result<()> {
    let rebase_button = style::get_button(builder, "make-rebase-button");
    rebase_button.set_sensitive(false);

    let combo_box = obtain_combo_box_from_builder(builder)?;
    combo_box.set_sensitive(true);
    combo_box.remove_all();
    write_message_into_text_view(builder, "")?;
    let rebase_clone = Rc::clone(&rebase);
    load_and_write_diffs(builder, rebase_clone, git_dir)?;
    let rebase_step_clone = Rc::clone(&rebase);
    let builder_clone = builder.clone();
    combo_box.connect_changed(move |_| {
        let diff = rebase_step_clone.borrow().rebase_step.diffs.clone();
        let result = combo_box_on_change(&builder_clone, diff);
        match result {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error writing into TextView: {}", e);
            }
        }
    });

    setup_buttons(builder, Rc::clone(&rebase), git_dir);
    Ok(())
}

fn create_rebase_commit(rebase: &Rebase, git_dir: &str) -> io::Result<String> {
    let commit_to_rebase = &rebase.commit_to_rebase.clone();
    let tree_to_rebase = tree_handler::load_tree_from_commit(commit_to_rebase, git_dir)?;
    let mut tree_with_changes = tree_to_rebase.clone();
    let rebase_step = &rebase.rebase_step;
    for (file, diff) in &rebase_step.diffs {
        let hash = hash_object::store_string_to_file(diff, git_dir, "blob")?;
        tree_with_changes.update_tree(file, &hash)
    }

    let active_commit = &rebase.active_commit.clone();
    let commit_message = format!("Rebasing with commit {}", &active_commit[0..7]);
    let new_commit_hash: String = commit::new_rebase_commit(
        git_dir,
        &commit_message,
        &rebase.commit_to_rebase,
        &tree_with_changes,
    )?;

    Ok(new_commit_hash)
}

fn finalize_rebase(builder: &gtk::Builder, git_dir: &str) -> io::Result<()> {
    let text_view = match style::get_text_view(builder, "rebase-view") {
        Some(text_view) => text_view,
        None => {
            println!("No se pudo obtener el TextView");
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo obtener el TextView",
            ));
        }
    };
    match text_view.get_buffer() {
        Some(buffer) => {
            let branch_name = get_branch_name(git_dir)?;
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
    let update_button = style::get_button(builder, "rebase-button");
    let ok_button = style::get_button(builder, "rebase-ok-all-button");
    let rebase_button = style::get_button(builder, "make-rebase-button");
    combo_box.set_sensitive(false);
    update_button.set_sensitive(false);
    ok_button.set_sensitive(false);
    rebase_button.set_sensitive(true);

    Err(io::Error::new(
        io::ErrorKind::Other,
        "No hay más commits para rebase",
    ))
}

pub fn next_rebase_iteration(
    builder: &gtk::Builder,
    rebase: Rebase,
    git_dir: &str,
) -> io::Result<()> {
    let new_commit_hash = create_rebase_commit(&rebase, git_dir)?;
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
            finalize_rebase(builder, git_dir)?;
        }
    };
    Ok(())
}

fn fast_forward_rebase_commit(
    our_commit: &str,
    rebased_commit: &str,
    common_ancestor: &str,
    git_dir: &str,
) -> io::Result<String> {
    let rebased_tree = tree_handler::load_tree_from_commit(rebased_commit, git_dir)?;
    let our_tree = tree_handler::load_tree_from_commit(our_commit, git_dir)?;
    let common_ancestor_tree = tree_handler::load_tree_from_commit(common_ancestor, git_dir)?;

    let mut new_tree = rebased_tree;

    let files_changed_this_commit =
        tree_handler::get_files_with_changes(&common_ancestor_tree, &our_tree);

    for (path, hash) in files_changed_this_commit {
        new_tree.update_tree(&path, &hash);
    }

    let commit_message = format!("Rebasing with commit {}", &our_commit[0..7]);
    let new_commit_hash: String =
        commit::new_rebase_commit(git_dir, &commit_message, rebased_commit, &new_tree)?;

    Ok(new_commit_hash)
}

// Do a fast forward merge, this means that we simply put our branch commits on top of theirs.
// For every commit in our branch since the common ancestor, we create a new commit with the updates
// and point our branch to the last commit
fn fast_forward_rebase(
    our_branch_hash: &str,
    their_branch_hash: &str,
    git_dir: &str,
) -> io::Result<()> {
    let common_ancestor = merge::find_common_ancestor(our_branch_hash, their_branch_hash, git_dir)?;
    let mut our_branch_commits =
        utils::get_branch_commit_history_until(our_branch_hash, git_dir, &common_ancestor)?;

    let mut our_new_branch_hash: String = their_branch_hash.to_string();

    while let Some(commit) = our_branch_commits.pop() {
        our_new_branch_hash =
            fast_forward_rebase_commit(&commit, &our_new_branch_hash, &common_ancestor, git_dir)?;
    }

    let branch_name = get_branch_name(git_dir)?;
    let root_dir = match Path::new(&git_dir).parent() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No se pudo encontrar el working dir",
            ));
        }
    };
    write_hash_into_branch_file(&our_new_branch_hash, git_dir)?;
    let git_dir_path = Path::new(&git_dir);
    checkout::checkout_branch(git_dir_path, &root_dir, &branch_name)?;

    Ok(())
}

pub fn start_rebase_gui(
    git_dir: &str,
    our_branch: &str,
    branch_to_rebase: &str,
) -> io::Result<Rc<RefCell<Rebase>>> {
    let our_branch_hash = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_branch_hash = branch::get_branch_commit_hash(branch_to_rebase, git_dir)?;
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

    // If there are no conflicts, we will do a fast forward rebase
    let conflicting_files = intersection_files_that_changed_between_commits(
        &common_ancestor,
        &our_branch_hash,
        &their_branch_hash,
        git_dir,
    )?;
    println!("{:#?}", conflicting_files);
    if conflicting_files.is_empty() {
        fast_forward_rebase(&our_branch_hash, &their_branch_hash, git_dir)?;
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No hay conflictos, se hizo un fast forward rebase",
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

// A function to do a rebase without gui
pub fn rebase(our_branch: &str, their_branch: &str, git_dir: &str) -> io::Result<()> {
    // We will will only do a fast forward rebase. If the rebase is not fast forward, we will tell the user to go to the gui
    let our_branch_hash = branch::get_branch_commit_hash(our_branch, git_dir)?;
    let their_branch_hash = branch::get_branch_commit_hash(their_branch, git_dir)?;
    let common_ancestor =
        merge::find_common_ancestor(&our_branch_hash, &their_branch_hash, git_dir)?;

    let conflicting_files = intersection_files_that_changed_between_commits(
        &common_ancestor,
        &our_branch_hash,
        &their_branch_hash,
        git_dir,
    )?;
    if conflicting_files.is_empty() {
        fast_forward_rebase(&our_branch_hash, &their_branch_hash, git_dir)?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "El rebase no es fast forward, por favor use la interfaz gráfica",
        ))
    }
}
