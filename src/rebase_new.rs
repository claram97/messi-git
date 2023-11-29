use std::{io::{self, Write}, hash, collections::HashMap, rc::Rc, cell::RefCell};
use gtk::{prelude::{BuilderExtManual, ComboBoxExtManual}, TextView, TextViewExt, TextBufferExt, ComboBoxTextExt, ComboBoxExt, TreeModelExt, ButtonExt};

use crate::{commit, branch, merge, utils, tree_handler, rebase, hash_object, cat_file, diff};

fn files_that_changed_between_commits(commit1: &str, commit2: &str, git_dir: &str) -> io::Result<Vec<(String, String)>> {
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

impl Rebase {
    // this function will save the Rebase in its current state to the disk
    pub fn save_rebase(&self, git_dir: &str) -> io::Result<()> {
        let rebase_path = format!("{}/rebase_status", git_dir);
        let mut rebase_file = std::fs::File::create(rebase_path)?;
        let mut rebase_content = String::new();
        rebase_content.push_str(&self.active_commit);
        rebase_content.push_str("\n");
        rebase_content.push_str(&self.commit_to_rebase);
        rebase_content.push_str("\n");
        rebase_content.push_str(&self.original_our_branch_hash);
        rebase_content.push_str("\n");
        // Print our commits vector
        for commit in &self.our_commits {
            rebase_content.push_str(&commit);
            rebase_content.push_str("\n");
        }
        // Print a separator to separate the data from the diffs
        rebase_content.push_str("====================================\n");

        for (file, diff) in &self.rebase_step.diffs {
            rebase_content.push_str(&file);
            rebase_content.push_str("\n");
            rebase_content.push_str(&diff);
            rebase_content.push_str("\n");
        }
        rebase_file.write_all(rebase_content.as_bytes())?;
        Ok(())
    }

    pub fn load_rebase(git_dir: &str) -> io::Result<Rebase> {
        let rebase_path = format!("{}/rebase_status", git_dir);
        let rebase_content = std::fs::read_to_string(rebase_path)?;
        let mut lines = rebase_content.lines();
        let active_commit = lines.next().unwrap().to_string();
        let commit_to_rebase = lines.next().unwrap().to_string();
        let original_our_branch_hash = lines.next().unwrap().to_string();
        let mut our_commits: Vec<String> = Vec::new();
        let mut commit = lines.next().unwrap().to_string();
        while commit != "====================================" {
            our_commits.push(commit);
            commit = lines.next().unwrap().to_string();
        }

        let mut commit = lines.next().unwrap().to_string();
        let mut diffs: HashMap<String, String> = HashMap::new();
        //The format is file\ncontent\nfile\ncontent\n
        while commit != "" {
            println!("commit: {}", commit);
            let file = commit.clone();
            let diff = match lines.next() {
                Some(diff) => diff.to_string(),
                None => {
                    println!("Error al leer el archivo de rebase");
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Error al leer el archivo de rebase",
                    ));
                }
            };
            diffs.insert(file, diff);
            commit = match lines.next() {
                Some(commit) => commit.to_string(),
                None => {
                    break;
                }
            };
        }

        let rebase_step = RebaseStep {
            diffs,
        };
        let rebase = Rebase {
            our_commits,
            active_commit,
            commit_to_rebase,
            original_our_branch_hash,
            rebase_step,
        };
        Ok(rebase)
    }
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

pub fn write_rebase_step_into_gui(builder: &gtk::Builder, rebase: Rc<RefCell<Rebase>>, git_dir: &str) -> io::Result<()>{
    let text_view = obtain_text_view_from_builder(builder).unwrap();
    let combo_box = obtain_combo_box_from_builder(builder).unwrap();
    let text_buffer = text_view.get_buffer().unwrap();
    let update_button = obtain_update_button_from_builder(builder).unwrap();

    text_buffer.set_text("");
    combo_box.remove_all();
    
    let commit_to_rebase: &str = &rebase.borrow().commit_to_rebase.clone();
    let tree_to_rebase = tree_handler::load_tree_from_commit(commit_to_rebase, git_dir)?;
    let active_commit = &rebase.borrow().active_commit.clone();
    let tree_active_commit = tree_handler::load_tree_from_commit(active_commit, git_dir)?;

    let changed_files = tree_handler::get_files_with_changes(&tree_to_rebase, &tree_active_commit);
    // For every file that changed, we need its diff. We will store the diff in a vector of tuples (path, diff)
    let mut diffs: HashMap<String, String> = HashMap::new();
    for (file, hash) in &changed_files {
        let hash_active_commit = tree_active_commit.get_hash_from_path(&file).unwrap();
        let hash_to_rebase = tree_to_rebase.get_hash_from_path(&file).unwrap();
        let diff = diff::return_object_diff_string(&hash_active_commit, &hash_to_rebase, &git_dir);
        match diff {
            Ok(diff) => {
                diffs.insert(file.clone(), diff);
            },
            Err(_) => {
                println!("No se pudo obtener el diff para el archivo {}", file);
            }
        }
    }

    if changed_files.len() == 0 {
        // Print a message in the text view
        text_buffer.set_text("No hay problemas con los archivos\n Presione Ok para continuar al siguiente commit");
        return Ok(());
    } else

    {
        let mut rebase_step = rebase.borrow_mut();
        rebase_step.rebase_step.diffs = diffs;
    }

    for (file, _) in changed_files {
        combo_box.append_text(&file);
    }

    combo_box.set_active(Some(0));
    let file = combo_box.get_active_text().unwrap().to_string();
    let diff = rebase.borrow().rebase_step.diffs.get(&file).unwrap().clone();
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
        let diff = rebase_step_clone.borrow().rebase_step.diffs.get(&file).unwrap().clone();
        text_buffer_clone.set_text(&diff);
    });
    
    let git_dir_clone = git_dir.to_string().clone();
    let text_buffer_clone = text_buffer.clone();
    let rebase_step_clone = Rc::clone(&rebase);
    update_button.connect_clicked(move |_| {
        let text = text_buffer_clone.get_text(&text_buffer_clone.get_start_iter(), &text_buffer_clone.get_end_iter(), false).unwrap().to_string();
        let file = combo_box.get_active_text().unwrap().to_string();
        
        {
            let mut rebase_step = rebase_step_clone.borrow_mut();
            rebase_step.rebase_step.diffs.insert(file, text);
        }
        rebase_step_clone.borrow().save_rebase(&git_dir_clone).unwrap();
    });
    Ok(())
}

pub fn next_rebase_iteration(builder: &gtk::Builder, git_dir: &str) -> io::Result<()> {
    let rebase = match Rebase::load_rebase(git_dir) {
        Ok(rebase) => rebase,
        Err(_) => {
            println!("No hay rebase en progreso");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No hay rebase en progreso",
            ));
        }
    };
    let commit_to_rebase = &rebase.commit_to_rebase;
    let tree_to_rebase = tree_handler::load_tree_from_commit(commit_to_rebase, git_dir).unwrap();

    let mut tree_with_changes = tree_to_rebase.clone();
    let rebase_step = &rebase.rebase_step;
    for (file, diff) in &rebase_step.diffs {
        let hash = hash_object::store_string_to_file(&diff, &git_dir, "blob")?;
        tree_with_changes.update_tree(&file, &hash)
    }

    let commit_message = format!("Rebase commit {}", rebase.active_commit);
    let new_commit_hash = commit::new_rebase_commit(git_dir, &commit_message, &rebase.commit_to_rebase, &tree_with_changes)?;

    let mut rebase = rebase;
    let next_commit = match rebase.our_commits.pop() {
        Some(commit) => commit,
        None => {
            // Delete the rebase file
            let rebase_path = format!("{}/rebase_status", git_dir);
            std::fs::remove_file(rebase_path)?;
            return Ok(());
        }
    };
    println!("Next commit: {}", next_commit);
    println!("New commit: {}", new_commit_hash);
    rebase.active_commit = next_commit;
    rebase.commit_to_rebase = new_commit_hash;
    let commit_to_rebase = &rebase.commit_to_rebase;
    let tree_to_rebase = tree_handler::load_tree_from_commit(commit_to_rebase, git_dir).unwrap();
    let active_commit = &rebase.active_commit.clone();
    let tree_active_commit = tree_handler::load_tree_from_commit(active_commit, git_dir).unwrap();
    let changed_files = tree_handler::get_files_with_changes(&tree_to_rebase, &tree_active_commit);
    rebase.rebase_step.diffs = HashMap::new();
    rebase.save_rebase(git_dir)?;
    println!("{:#?}", rebase);
    write_rebase_step_into_gui(builder, Rc::new(RefCell::new(rebase)), git_dir)?;
    Ok(())
}



pub fn start_rebase_gui(git_dir: &str, our_branch: &str, branch_to_rebase: &str) -> io::Result<Rc<RefCell<Rebase>>> {
    let our_branch_hash = branch::get_branch_commit_hash(&our_branch, git_dir)?;
    let their_branch_hash = branch::get_branch_commit_hash(&branch_to_rebase, git_dir)?;
    let common_ancestor = merge::find_common_ancestor(&our_branch_hash, &their_branch_hash, git_dir)?;

    let mut our_commits = utils::get_branch_commit_history_until(&our_branch_hash, git_dir, &common_ancestor)?;
    let active_commit = our_commits.pop().unwrap();

    let files_with_changes = files_that_changed_between_commits(&our_branch_hash, &their_branch_hash, git_dir)?;
    if files_with_changes.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No hay cambios entre los commits",
        ));
    }
        // We need to do a rebase
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