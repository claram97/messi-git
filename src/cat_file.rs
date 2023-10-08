use std::io;

pub fn cat_file(hash: &str) -> io::Result<String> {
    let content = std::fs::read_to_string(format!("objects/{}", hash))?;
    println!("{}", content);
    Ok(content)
}