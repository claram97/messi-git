pub fn cat_file(hash: &str) {
    let content = std::fs::read_to_string(format!("objects/{}", hash)).expect("Unable to read file");
    println!("{}", content);
}