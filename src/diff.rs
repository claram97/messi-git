use crate::cat_file::cat_file_return_content;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

/// Prints the difference between two text files using line-by-line comparison.
///
/// This function reads the contents of two text files specified by their file paths and compares them
/// line by line. It then prints the differences between the two files, highlighting additions, deletions,
/// and modifications. The output is displayed in a human-readable format to the console.
///
/// # Arguments
///
/// * `path_a`: A string representing the file path of the first text file for comparison.
/// * `path_b`: A string representing the file path of the second text file for comparison.
///
pub fn print_diff(path_a: &str, path_b: &str) {
    let archivo_a = match read_file_lines(path_a) {
        Ok(lines) => lines,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let archivo_b = match read_file_lines(path_b) {
        Ok(lines) => lines,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    show_diff(&archivo_a, &archivo_b, archivo_a.len(), archivo_b.len());
}

/// Reads the lines from a file and returns them as a vector of strings.
///
/// # Arguments
///
/// * `path` - A string representing the path to the file.
///
/// # Returns
///
/// Returns a `Result` containing a vector of strings if the file is successfully read,
/// otherwise returns an `Err` with an error message as a `String`.
///
fn read_file_lines(path: &str) -> Result<Vec<String>, String> {
    match File::open(path) {
        Ok(file) => {
            let lines_vector: Result<Vec<String>, std::io::Error> =
                BufReader::new(file).lines().collect();
            match lines_vector {
                Ok(lines) => Ok(lines),
                Err(_) => Err("File content cannot be read".to_string()),
            }
        }
        Err(_) => Err("File does not exist or cannot be opened".to_string()),
    }
}

/// Computes the matrix for the Longest Common Subsequence (LCS) problem.
///
/// This function calculates the matrix for the Longest Common Subsequence problem
/// given two vectors of strings. The matrix represents the length of the LCS between
/// the elements of the two vectors.
///
/// # Arguments
///
/// * `a` - A reference to a vector of strings.
/// * `b` - A reference to another vector of strings.
///
/// # Returns
///
/// Returns a 2D vector (matrix) of integers representing the LCS lengths.
///
fn compute_longest_common_subsequence_matrix(a: &Vec<String>, b: &Vec<String>) -> Vec<Vec<i32>> {
    let mut matrix = vec![vec![0; b.len() + 1]; a.len() + 1];
    for i in matrix.iter_mut() {
        for j in i.iter_mut() {
            *j = 0;
        }
    }

    for (i_pos, i) in a.iter().enumerate() {
        for (j_pos, j) in b.iter().enumerate() {
            if i == j {
                matrix[i_pos + 1][j_pos + 1] = matrix[i_pos][j_pos] + 1;
            } else {
                matrix[i_pos + 1][j_pos + 1] =
                    std::cmp::max(matrix[i_pos + 1][j_pos], matrix[i_pos][j_pos + 1]);
            }
        }
    }
    matrix
}

/// i y j len(x) y len(y) respectivamente
fn show_diff(x: &Vec<String>, y: &Vec<String>, i: usize, j: usize) {
    // Enunciado
    // C es la grilla computada por lcs()
    // X e Y son las secuencias
    // i y j especifican la ubicacion dentro de C que se quiere buscar cuando
    // se lee el diff. Al llamar a estar funcion inicialmente, pasarle
    // i=len(X) y j=len(Y)
    let c: Vec<Vec<i32>> = compute_longest_common_subsequence_matrix(x, y);

    if i > 0 && j > 0 && x[i - 1] == y[j - 1] {
        show_diff(x, y, i - 1, j - 1);
        println!("  {}", x[i - 1]);
    } else if j > 0 && (i == 0 || c[i][j - 1] >= c[i - 1][j]) {
        show_diff(x, y, i, j - 1);
        println!(">> {}", y[j - 1]);
    } else if i > 0 && (j == 0 || c[i][j - 1] < c[i - 1][j]) {
        show_diff(x, y, i - 1, j);
        println!("<< {}", x[i - 1]);
    } else {
        println!();
    }
}

/// Returns the difference between two files as a vector of strings.
///
/// This function takes two file paths, reads their contents, and calculates the difference
/// between them using a simple line-based diff algorithm. The result is a vector of strings
/// representing the lines that differ between the two files.
///
/// # Arguments
///
/// * `path_a` - A string representing the path to the first file.
/// * `path_b` - A string representing the path to the second file.
///
/// # Returns
///
/// Returns a `Result` containing a vector of strings representing the difference
/// between the two files. If there is an error reading the files, an `Err` variant
/// with an error message is returned.
///
pub fn return_diff(path_a: &str, path_b: &str) -> Result<Vec<String>, String> {
    let archivo_a = read_file_lines(path_a)?;
    let archivo_b = read_file_lines(path_b)?;

    Ok(diff_to_vec(
        &archivo_a,
        &archivo_b,
        archivo_a.len(),
        archivo_b.len(),
    ))
}

/// Converts the result of a longest common subsequence matrix computation into a vector of strings representing the differences.
///
/// This function takes two sequences represented by vectors (`x` and `y`) and their corresponding longest common subsequence matrix (`c`).
/// It recursively processes the matrix to identify and generate a vector of strings representing the differences between the two sequences.
///
/// # Arguments
///
/// * `x` - A reference to the first sequence as a vector of strings.
/// * `y` - A reference to the second sequence as a vector of strings.
/// * `i` - The index in the first sequence.
/// * `j` - The index in the second sequence.
///
/// # Returns
///
/// Returns a vector of strings representing the differences between the two sequences.
///
fn diff_to_vec(x: &Vec<String>, y: &Vec<String>, i: usize, j: usize) -> Vec<String> {
    let c: Vec<Vec<i32>> = compute_longest_common_subsequence_matrix(x, y);
    let mut output: Vec<String> = Vec::new();

    if i > 0 && j > 0 && x[i - 1] == y[j - 1] {
        output.append(&mut diff_to_vec(x, y, i - 1, j - 1));
        output.push(format!("{}\n", x[i - 1]));
    } else if j > 0 && (i == 0 || c[i][j - 1] >= c[i - 1][j]) {
        output.append(&mut diff_to_vec(x, y, i, j - 1));
        output.push(format!(">>>>>>> {}\n", y[j - 1]));
    } else if i > 0 && (j == 0 || c[i][j - 1] < c[i - 1][j]) {
        output.append(&mut diff_to_vec(x, y, i - 1, j));
        output.push(format!("<<<<<<< {}\n", x[i - 1]));
    }
    output
}

/// Returns a string representing the differences between two Git objects identified by their commit hashes.
///
/// This function takes two commit hashes (`hash_a` and `hash_b`) and the Git directory path (`git_dir`).
/// It retrieves the content of the Git objects corresponding to the given hashes, then computes the differences
/// between their content using the `diff_to_vec` function.
///
/// # Arguments
///
/// * `hash_a` - The commit hash of the first Git object.
/// * `hash_b` - The commit hash of the second Git object.
/// * `git_dir` - The path to the Git directory.
///
/// # Returns
///
/// Returns a `Result` where `Ok` contains a string representing the differences between the two Git objects,
/// and `Err` contains an error message if any error occurs during the process.
///
pub fn return_object_diff_string(
    hash_a: &str,
    hash_b: &str,
    git_dir: &str,
) -> Result<String, String> {
    let object_a = match cat_file_return_content(hash_a, git_dir) {
        Ok(content) => content,
        Err(error) => return Err(error.to_string()),
    };
    let object_b = match cat_file_return_content(hash_b, git_dir) {
        Ok(content) => content,
        Err(error) => return Err(error.to_string()),
    };
    let object_a_vec = object_a
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let object_b_vec = object_b
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut output: Vec<String> = Vec::new();
    for line in diff_to_vec(
        &object_a_vec,
        &object_b_vec,
        object_a_vec.len(),
        object_b_vec.len(),
    ) {
        output.push(line);
    }
    Ok(output.join(""))
}
