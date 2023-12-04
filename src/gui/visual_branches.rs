use crate::commit::{self, is_merge_commit};
use crate::{branch, utils};
use gtk::{prelude::BuilderExtManual, WidgetExt};
use std::f64::consts;
use std::{
    collections::{HashMap, HashSet},
    io,
};

const COMMIT_RADIUS: f64 = 15.0;
const BRANCHES_DISTANCE: i32 = 70;

#[derive(Debug, Clone)]
struct Commit {
    hash: String,
    message: String,
    time: String,
    branch: String,
    parents: Vec<String>,
}

impl Commit {
    fn new(
        hash: String,
        message: String,
        time: String,
        branch: String,
        parents: Vec<String>,
    ) -> Commit {
        Commit {
            hash,
            message,
            time,
            branch,
            parents,
        }
    }
}

/// Sets up the canvas for drawing the graph.
/// Sets the size of the canvas to fit the graph.
/// Sets the background color of the canvas.
/// Returns the canvas.
fn setup_canvas(builder: &gtk::Builder) -> io::Result<gtk::DrawingArea> {
    let drawing_area: gtk::DrawingArea = builder
        .get_object("visual-branches-area")
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get drawing area"))?;
    drawing_area.connect_draw(move |_, context| {
        context.set_source_rgb(0.529, 0.808, 0.922);
        context.paint();
        gtk::Inhibit(false)
    });
    Ok(drawing_area)
}

/// Defines the x positions of the branches.
/// Returns a HashMap with the branch name as key and the x position as value.
fn define_branch_positions(branches: &Vec<String>) -> HashMap<String, i32> {
    let mut branch_positions: HashMap<String, i32> = HashMap::new();
    let mut x_pos = 20;
    if branches.contains(&"master".to_string()) {
        branch_positions.insert("master".to_string(), x_pos);
        x_pos += BRANCHES_DISTANCE;
    }
    for branch in branches {
        if branch != "master" {
            branch_positions.insert(branch.clone(), x_pos);
            x_pos += BRANCHES_DISTANCE;
        }
    }
    branch_positions
}

/// Defines the colors of the branches.
/// Returns a HashMap with the x position as key and the color as value.
fn define_branch_colors(
    branches: &Vec<String>,
    branches_pos: &HashMap<String, i32>,
) -> HashMap<i32, (f64, f64, f64)> {
    let mut branch_colors: HashMap<i32, (f64, f64, f64)> = HashMap::new();
    let mut color_index = 0;
    let colors = [
        (0.2, 0.22, 0.95),
        (0.23, 0.65, 0.62),
        (0.43, 0.12, 0.7),
        (0.9, 0.2, 0.3),
        (0.96, 0.24, 0.82),
    ];
    for branch in branches {
        let x_pos = *branches_pos.get(branch).unwrap_or(&20);
        branch_colors.insert(x_pos, colors[color_index]);
        color_index += 1;
        if color_index == colors.len() {
            color_index = 0;
        }
    }
    branch_colors
}

/// Draws the branch names on the canvas.
/// The branch names are drawn on the top of the canvas.
fn draw_branch_names(
    drawing_area: &gtk::DrawingArea,
    branches: &Vec<String>,
    x_positions: &HashMap<String, i32>,
) {
    let mut y_pos = 10;
    for branch in branches {
        let name = branch.clone();
        let x_pos = *x_positions.get(&name).unwrap_or(&20);
        y_pos += 20;
        drawing_area.connect_draw(move |_, context| {
            context.set_source_rgb(0.0, 0.0, 0.0);
            context.move_to(x_pos as f64 - COMMIT_RADIUS, y_pos as f64);
            context.set_font_size(12.0);
            context.show_text(&name);
            gtk::Inhibit(false)
        });
    }
}

/// Draws a commit node with its message on the canvas.
/// The commit node is drawn in the x corresponding to its branch.
/// The message is drawn on the right side of the canvas. In a column with all the other messages.
fn draw_commit_node_w_message_side(
    drawing_area: &gtk::DrawingArea,
    node_pos: (i32, i32),
    message_pos: (i32, i32),
    color: (f64, f64, f64),
    hash: String,
    message: String,
) {
    drawing_area.connect_draw(move |_, context| {
        context.set_source_rgb(color.0, color.1, color.2);
        context.arc(
            node_pos.0 as f64,
            node_pos.1 as f64,
            COMMIT_RADIUS,
            0.0,
            2.0 * consts::PI,
        );
        context.fill();
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.move_to(message_pos.0 as f64 + 5.0, message_pos.1 as f64 - 5.0);
        context.show_text(&hash[..7]);
        context.move_to(message_pos.0 as f64 + 5.0, message_pos.1 as f64 + 10.0);
        context.show_text(&message);
        context.set_source_rgb(color.0, color.1, color.2);
        context.move_to(message_pos.0 as f64, message_pos.1 as f64 - 15.0);
        context.line_to(message_pos.0 as f64, message_pos.1 as f64 + 15.0);
        context.stroke();

        context.set_source_rgba(color.0, color.1, color.2, 0.25);
        context.move_to(message_pos.0 as f64, message_pos.1 as f64);
        context.line_to(node_pos.0 as f64, node_pos.1 as f64);
        context.set_line_width(COMMIT_RADIUS * 2.0);
        context.stroke();

        gtk::Inhibit(false)
    });
}

/// Draws all the commit nodes on the canvas.
/// Returns a HashMap with the commit hash as key and the node position as value.
fn draw_nodes(
    drawing_area: &gtk::DrawingArea,
    commits: &Vec<Commit>,
    branches: &Vec<String>,
    branches_pos: &HashMap<String, i32>,
    branch_colors: &HashMap<i32, (f64, f64, f64)>,
) -> HashMap<String, (i32, i32)> {
    let mut node_positions: HashMap<String, (i32, i32)> = HashMap::new();
    let commits_len = commits.len() as i32;
    let branches_len = branches.len() as i32;
    let mut y = if commits_len * 50 + 20 < 600 {
        600
    } else {
        commits_len * 50 + 20
    };
    y += 20 * branches_len;
    drawing_area.set_size_request(BRANCHES_DISTANCE * branches_len, y);

    for commit in commits {
        let branch = commit.branch.clone();
        let branch_x = *branches_pos.get(&branch).unwrap_or(&20);
        let color = *branch_colors.get(&branch_x).unwrap_or(&(0.0, 1.0, 0.22));
        let hash = commit.hash.clone();
        let message = commit.message.clone();
        let message_x = BRANCHES_DISTANCE * branches_len;
        let message_y = y;
        draw_commit_node_w_message_side(
            drawing_area,
            (branch_x, y),
            (message_x, message_y),
            color,
            hash.clone(),
            message.clone(),
        );
        node_positions.insert(hash, (branch_x, y));
        y -= 50;
    }

    node_positions
}

/// Draws a line between two nodes.
/// The line is drawn from the source node to the destination node.
/// The color of the line is the color of the branch of the source node.+
/// The line is an L shape.
fn draw_line(
    drawing_area: &gtk::DrawingArea,
    source: (i32, i32),
    dest: (i32, i32),
    color: (f64, f64, f64),
) {
    if source.0 == 0 && source.1 == 0 || dest.0 == 0 && dest.1 == 0 {
        return;
    }

    let offset = match source.0.cmp(&dest.0) {
        std::cmp::Ordering::Equal => 0.0,
        std::cmp::Ordering::Greater => COMMIT_RADIUS,
        std::cmp::Ordering::Less => -COMMIT_RADIUS,
    };

    drawing_area.connect_draw(move |_, context| {
        context.set_source_rgb(color.0, color.1, color.2);
        context.move_to(source.0 as f64, source.1 as f64);
        context.line_to(source.0 as f64, dest.1 as f64);
        context.line_to(dest.0 as f64 + offset, dest.1 as f64);
        context.set_line_width(2.0);

        context.stroke();
        gtk::Inhibit(false)
    });
}

/// Draws all the connections between the commit nodes.
fn draw_commits_connections(
    drawing_area: &gtk::DrawingArea,
    branch_colors: &HashMap<i32, (f64, f64, f64)>,
    node_positions: &HashMap<String, (i32, i32)>,
    connections: &Vec<(String, String)>,
) {
    for connection in connections {
        let branch_commit = connection.1.clone();
        let ancestor_commit = connection.0.clone();
        let branch = node_positions.get(&branch_commit).unwrap_or(&(0, 0));
        let ancestor = node_positions.get(&ancestor_commit).unwrap_or(&(0, 0));
        let color = *branch_colors.get(&branch.0).unwrap_or(&(0.0, 1.0, 0.22));

        draw_line(drawing_area, *branch, *ancestor, color)
    }
}

/// Draws the graph of the git in a visual way.
/// It uses circles to represent the commits and lines to represent the connections between the commits.
/// Each branch is represented by a different color.
///
/// It searches for the git directory in the current directory.
/// If it finds it, it builds the graph of the git.
pub fn handle_show_visual_branches_tree(builder: &gtk::Builder) -> io::Result<()> {
    let git_dir = utils::obtain_git_dir()?;

    let drawing_area = setup_canvas(builder)?;

    let branches = branch::get_all_branches(&git_dir)?;
    let branches_x_positions = define_branch_positions(&branches);
    let branch_colors = define_branch_colors(&branches, &branches_x_positions);
    draw_branch_names(&drawing_area, &branches, &branches_x_positions);

    let graph = build_git_graph(&git_dir)?;
    let mut commits: Vec<Commit> = graph.values().cloned().collect();
    commits.sort_by(|a, b| a.time.cmp(&b.time));

    let node_positions = draw_nodes(
        &drawing_area,
        &commits,
        &branches,
        &branches_x_positions,
        &branch_colors,
    );

    let mut connections: Vec<(String, String)> = Vec::new();
    for node in &commits {
        let parents = node.parents.clone();
        let hash = node.hash.clone();
        if !parents.is_empty() {
            for parent in parents {
                let connection = (parent.clone(), hash.clone());
                connections.push(connection);
            }
        }
    }

    draw_commits_connections(&drawing_area, &branch_colors, &node_positions, &connections);

    drawing_area.queue_draw();
    Ok(())
}

/// Inserts a commit in the graph.
fn process_commit(
    commit_hash: String,
    branch_name: String,
    git_dir: &str,
    graph: &mut HashMap<String, Commit>,
    queue: &mut Vec<(String, String)>,
    visited: &mut HashSet<String>,
) -> io::Result<()> {
    if visited.contains(&commit_hash) {
        return Ok(());
    }

    match commit::get_commit_message(&commit_hash, git_dir) {
        Ok(_) => {}
        Err(_e) => return Ok(()),
    }

    visited.insert(commit_hash.clone());
    let commit_msg = commit::get_commit_message(&commit_hash, git_dir)?;
    let commit_time = commit::get_commit_time(&commit_hash, git_dir)?;

    let parents = if commit::is_merge_commit(&commit_hash, git_dir)? {
        commit::get_merge_parents(&commit_hash, git_dir)?
    } else {
        let parents = vec![commit::get_parent_hash(&commit_hash, git_dir)?];
        parents
    };

    let commit = Commit::new(
        commit_hash.clone(),
        commit_msg,
        commit_time,
        branch_name.clone(),
        parents,
    );
    graph.insert(commit_hash.clone(), commit);

    queue.push((commit_hash, branch_name));

    Ok(())
}

/// Builds the graph of the git dir.
/// It returns a HashMap with the commit hash as key and the commit as value.
/// The commit contains the commit hash, the commit message, the commit time, the branch name and the parent hash.
/// The graph is built from the heads of the branches.
fn build_git_graph(git_dir: &str) -> io::Result<HashMap<String, Commit>> {
    let mut graph: HashMap<String, Commit> = HashMap::new();
    let mut queue: Vec<(String, String)> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    for branch_name in branch::get_all_branches(git_dir)? {
        let head_hash = branch::get_branch_commit_hash(&branch_name, git_dir)?;
        process_commit(
            head_hash.clone(),
            branch_name.clone(),
            git_dir,
            &mut graph,
            &mut queue,
            &mut visited,
        )?;
    }

    // I process master first so it can be clearly seen on the graph
    let master_hash = branch::get_branch_commit_hash("master", git_dir)?;
    for commit_hash in utils::get_branch_commit_history(&master_hash, git_dir)? {
        process_commit(
            commit_hash.clone(),
            "master".to_string(),
            git_dir,
            &mut graph,
            &mut queue,
            &mut visited,
        )?;
    }

    while let Some((current_hash, branch_name)) = queue.pop() {
        let parents = if is_merge_commit(&current_hash, git_dir)? {
            commit::get_merge_parents(&current_hash, git_dir)?
        } else {
            let parents = vec![commit::get_parent_hash(&current_hash, git_dir)?];
            parents
        };

        for parent in parents {
            process_commit(
                parent.clone(),
                branch_name.clone(),
                git_dir,
                &mut graph,
                &mut queue,
                &mut visited,
            )?;
        }
    }
    Ok(graph)
}
