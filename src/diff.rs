use crate::cat_file::cat_file_return_content;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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

// pub fn write_diff_to_file(path_a: &str, path_b: &str, output: &mut impl Write) -> io::Result<()> {
//     let archivo_a = read_file_lines(path_a)?;
//     let archivo_b = read_file_lines(path_b)?;

//     let mut output = BufWriter::new(output);
//     for line in diff_to_vec(&archivo_a, &archivo_b, archivo_a.len(), archivo_b.len()) {
//         writeln!(output, "{}", line)?;
//     }
//     Ok(())
// }

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
