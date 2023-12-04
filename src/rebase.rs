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

/// Obtains a ComboBoxText object from a GTK builder.
///
/// # Arguments
///
/// * `builder` - A gtk::Builder object containing the graphical interface.
///
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

/// Loads and returns the differences between files in two specified commits.
///
/// # Arguments
///
/// * `commit_to_rebase` - The commit hash to which the rebase is being performed.
/// * `active_commit` - The commit hash representing the currently active commit.
/// * `git_dir` - The path to the Git directory.
///
/// # Returns
///
/// Returns a Result containing a HashMap of file paths to their respective differences
/// between the active commit and the commit to rebase. An empty HashMap is returned
/// if there are no differences or an error occurred.
///
/// # Errors
///
/// Returns an error if there are issues loading the tree from the specified commits
/// or if obtaining differences for a file fails.
///
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

/// Retrieves the root directory of a Git repository based on the specified Git directory.
///
/// # Arguments
///
/// * `git_dir` - The path to the Git directory.
///
/// # Returns
///
/// Returns a Result containing the root directory path as a String if successful.
/// An error is returned if the working directory cannot be determined.
///
/// # Errors
///
/// Returns an error if the parent directory of the specified Git directory cannot be obtained.
///
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

/// Retrieves the content of a GTK TextView as a String.
///
/// # Arguments
///
/// * `builder` - A gtk::Builder object containing the graphical interface.
///
/// # Returns
///
/// Returns a Result containing the text content of the TextView if successful.
/// An error is returned if the TextView or its associated buffer cannot be obtained.
///
/// # Errors
///
/// Returns an error if the TextView or its associated buffer cannot be obtained,
/// or if the text content cannot be retrieved from the buffer.
///
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

/// Handles the click event for the "Abort Rebase" button in the GTK application.
///
/// This function performs the necessary actions to abort the ongoing rebase process.
///
/// # Arguments
///
/// * `builder` - A gtk::Builder object containing the graphical interface.
/// * `original_our_branch_hash` - The original hash of the commit in our branch before rebase.
///
/// # Returns
///
/// Returns a Result with a unit value if the operation is successful.
/// An error is returned if there are issues writing to files or performing Git operations.
///
/// # Errors
///
/// Returns an error if there are issues writing the original commit hash into the branch file,
/// checking out to the original branch, or writing a message to the TextView.
///
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

/// Handles the change event for the ComboBox in the GTK application.
///
/// This function retrieves the selected file from the ComboBox and displays its diff
/// in the associated TextView within the graphical interface.
///
/// # Arguments
///
/// * `builder` - A gtk::Builder object containing the graphical interface.
/// * `diff` - A HashMap containing file paths as keys and their corresponding diffs as values.
///
/// # Returns
///
/// Returns a Result with a unit value if the operation is successful.
/// An error is returned if the selected file or its corresponding diff cannot be obtained.
///
/// # Errors
///
/// Returns an error if the selected file or its corresponding diff cannot be obtained.
///
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

/// Handles the click event for the "Update" button in the GTK application.
///
/// This function retrieves the content of the TextView and the selected file from the ComboBox,
/// then updates the associated diff in the Rebase structure.
///
/// # Arguments
///
/// * `builder` - A gtk::Builder object containing the graphical interface.
/// * `rebase` - A shared reference to a Rebase structure stored in a RefCell and wrapped in Rc.
///
/// # Returns
///
/// Returns a Result with a unit value if the operation is successful.
/// An error is returned if the text content from the TextView or the selected file cannot be obtained.
///
/// # Errors
///
/// Returns an error if the text content from the TextView or the selected file cannot be obtained.
///
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

/// Loads and writes the file differences (diffs) into the GTK application's ComboBox and TextView.
///
/// This function retrieves the file differences (diffs) between the commit to rebase and the active commit
/// from the specified Git directory. It then updates the ComboBox with the file paths and displays
/// the content of the selected file in the TextView within the graphical interface.
///
/// # Arguments
///
/// * `builder` - A gtk::Builder object containing the graphical interface.
/// * `rebase` - A shared reference to a Rebase structure stored in a RefCell and wrapped in Rc.
/// * `git_dir` - A string representing the path to the Git directory.
///
/// # Returns
///
/// Returns a Result with a unit value if the operation is successful.
/// An error is returned if the file differences cannot be loaded or if updating the ComboBox and TextView fails.
///
/// # Errors
///
/// Returns an error if the file differences cannot be loaded or if updating the ComboBox and TextView fails.
///
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

/// Sets up the event handlers for the buttons in the GTK application.
///
/// This function initializes and connects event handlers for the "Update," "Abort Rebase," and "OK All" buttons
/// in the GTK interface. It enables these buttons and associates their respective actions with the corresponding functions.
///
/// # Arguments
///
/// * `builder` - A gtk::Builder object containing the graphical interface.
/// * `rebase` - A shared reference to a Rebase structure stored in a RefCell and wrapped in Rc.
/// * `git_dir` - A string representing the path to the Git directory.
///
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

/// Writes the current state of the rebase step into the GTK graphical interface.
///
/// This function updates the GTK interface to reflect the current state of the rebase step.
/// It disables the "Make Rebase" button, sets the combo box to be sensitive, removes all items from the combo box,
/// clears the text view, loads and writes the file differences, and sets up the event handlers for buttons.
///
/// # Arguments
///
/// * `builder` - A gtk::Builder object containing the graphical interface.
/// * `rebase` - A shared reference to a Rebase structure stored in a RefCell and wrapped in Rc.
/// * `git_dir` - A string representing the path to the Git directory.
///
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

/// Creates a new commit during the rebase process and returns its hash.
///
/// This function takes a Rebase structure and the path to the Git directory as input.
/// It loads the tree of the commit to be rebased, applies the changes specified in the rebase step,
/// creates a new commit with the updated tree, and returns the hash of the new commit.
///
/// # Arguments
///
/// * `rebase` - A reference to the Rebase structure representing the current state of the rebase.
/// * `git_dir` - A string representing the path to the Git directory.
///
/// # Returns
///
/// A Result containing the hash of the newly created commit or an IO error.
///
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

/// Finalizes the rebase process, updating the branch and displaying a completion message.
///
/// This function is responsible for completing the rebase process. It checks out the branch
/// that was rebased, sets the completion message in the TextView, and adjusts the sensitivity
/// of relevant UI elements in the GTK builder. Finally, it returns an error indicating that
/// there are no more commits for rebase, as this function is typically called after the last rebase step.
///
/// # Arguments
///
/// * `builder` - A reference to the GTK builder containing the UI elements.
/// * `git_dir` - A string representing the path to the Git directory.
///
/// # Returns
///
/// An IO Result indicating the completion of the rebase with an error indicating
/// that there are no more commits for rebase.
///
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

/// Initiates the next iteration of the rebase process.
///
/// This function performs the next iteration of the rebase process, creating a new commit
/// with the changes from the current rebase step. If there are more commits to rebase,
/// it updates the rebase information and writes the new rebase step into the GUI. Otherwise,
/// it finalizes the rebase by checking out the rebased branch and displaying a completion message.
///
/// # Arguments
///
/// * `builder` - A reference to the GTK builder containing the UI elements.
/// * `rebase` - The current state of the rebase process.
/// * `git_dir` - A string representing the path to the Git directory.
///
/// # Returns
///
/// An IO Result indicating the success of the next rebase iteration.
///
pub fn next_rebase_iteration(
    builder: &gtk::Builder,
    rebase: Rebase,
    git_dir: &str,
) -> io::Result<()> {
    let new_commit_hash = create_rebase_commit(&rebase, git_dir)?;
    let mut new_rebase = rebase;

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

/// Performs a fast-forward rebase by updating the rebase commit with changes from the new commit.
///
/// This function updates the tree of the rebase commit with changes from the new commit.
/// It calculates the differences between the new commit and the common ancestor commit.
/// The resulting tree is used to create a new rebase commit, effectively fast-forwarding
/// the rebase process.
///
/// # Arguments
///
/// * `our_commit` - The hash of the commit in the current branch before rebase.
/// * `rebased_commit` - The hash of the commit in the rebased branch.
/// * `common_ancestor` - The hash of the common ancestor commit of `our_commit` and `rebased_commit`.
/// * `git_dir` - A string representing the path to the Git directory.
///
/// # Returns
///
/// A string representing the hash of the new rebase commit created through the fast-forward process.
///
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

/// Initializes and starts the interactive rebase process in a graphical user interface (GUI).
///
/// This function prepares the necessary data for rebase, including identifying the common ancestor
/// of the source and target branches, determining the commits to rebase, and checking for conflicts.
/// It then creates and returns a `Rebase` struct wrapped in a `RefCell` and `Rc` for GUI interaction.
///
/// # Arguments
///
/// * `git_dir` - A string representing the path to the Git directory.
/// * `our_branch` - The name of the branch from which the rebase originates.
/// * `branch_to_rebase` - The name of the branch to rebase onto the `our_branch`.
///
/// # Returns
///
/// An `io::Result` containing a reference-counted, mutable `Rebase` wrapped in a `RefCell`.
///
/// # Errors
///
/// Returns an error if there are no commits available for rebase or if a fast-forward rebase is performed.
///
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
        println!("El rebase es fast forward");
        fast_forward_rebase(&our_branch_hash, &their_branch_hash, git_dir)?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "El rebase no es fast forward, por favor use la interfaz gráfica",
        ))
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::add;
    use std::fs;

    const NAME_OF_GIT_DIRECTORY_1: &str = "tests/rebase_tests/test1/.mgit";

    fn create_mock_git_dir(git_dir: &str) {
        fs::create_dir_all(&git_dir).unwrap();
        let objects_dir = format!("{}/objects", git_dir);
        fs::create_dir_all(&objects_dir).unwrap();
        let refs_dir = format!("{}/refs/heads", git_dir);
        fs::create_dir_all(&refs_dir).unwrap();
        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file.write_all(b"ref: refs/heads/master").unwrap();
    }

    #[test]
    fn test_rebase_fast_forward() {
        let git_dir = NAME_OF_GIT_DIRECTORY_1;
        let test_dir = "tests/rebase_tests/test1";
        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir).unwrap();
        }
        create_mock_git_dir(git_dir);
        let index_file_path = format!("{}/index", NAME_OF_GIT_DIRECTORY_1);
        let _ = fs::File::create(&index_file_path).unwrap();
        let src_dir = format!("{}/src", test_dir);
        fs::create_dir_all(&src_dir).unwrap();

        let file_path = "src/main.c";
        let add_path = format!("{}/{}", test_dir, file_path);
        let file_content = "int main() { return 0; }";
        let _ = fs::write(format!("{}/{}", test_dir, file_path), file_content).unwrap();
        let _ = add::add(&add_path, &index_file_path, git_dir, "", None);

        let index_file_path = format!("{}/index", NAME_OF_GIT_DIRECTORY_1);
        let index_file_content = fs::read_to_string(&index_file_path).unwrap();
        println!("Index file content: {}", index_file_content);

        let commit_message = "Initial commit".to_string();
        let commit_1_hash = commit::new_commit(git_dir, &commit_message, "").unwrap();

        let branch_name = "test";
        let _ = branch::create_new_branch(&git_dir, "test", None, &mut io::stdout());

        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/{}", branch_name).as_bytes())
            .unwrap();

        let file_path = "src/hello.c";
        let add_path = format!("{}/{}", test_dir, file_path);
        let file_content = "int hello() { return 0; }";
        let _ = fs::write(format!("{}/{}", test_dir, file_path), file_content).unwrap();
        let _ = add::add(&add_path, &index_file_path, git_dir, "", None);

        let commit_message = "Second commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let file_path = "src/bye.c";
        let add_path = format!("{}/{}", test_dir, file_path);
        let file_content = "int bye() { return 0; }";
        let _ = fs::write(format!("{}/{}", test_dir, file_path), file_content).unwrap();
        let _ = add::add(&add_path, &index_file_path, git_dir, "", None);

        let commit_message = "Third commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file.write_all(b"ref: refs/heads/master").unwrap();

        let file_path = "src/pizza.c";
        let add_path = format!("{}/{}", test_dir, file_path);
        let file_content = "int pizza() { return 0; }";
        let _ = fs::write(format!("{}/{}", test_dir, file_path), file_content).unwrap();
        let _ = add::add(&add_path, &index_file_path, git_dir, "", None);

        let commit_message = "Fourth commit";
        let _ = commit::new_commit(&git_dir, commit_message, "").unwrap();

        let head_file_path = format!("{}/HEAD", git_dir);
        let mut head_file = fs::File::create(&head_file_path).unwrap();
        head_file
            .write_all(format!("ref: refs/heads/test").as_bytes())
            .unwrap();

        let our_branch = "test";
        let their_branch = "master";
        let result = rebase(our_branch, their_branch, git_dir);
        assert!(result.is_ok());

        // Check if the amount of commits is correct
        let test_hash = branch::get_branch_commit_hash("test", git_dir).unwrap();
        let our_branch_commits = utils::get_branch_commit_history(&test_hash, git_dir).unwrap();

        // The amount of commits in the vector should be 5, the 4 commits plus the 000000000... commit
        assert_eq!(our_branch_commits.len(), 5);

        // our_branch_commits[3] should be commit_hash_1
        assert_eq!(our_branch_commits[3], commit_1_hash);

        fs::remove_dir_all(test_dir).unwrap();
    }
}
