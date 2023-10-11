use messi::hash_object;

fn main() {
    println!("Hello, World!");

    let result = hash_object::store_file("src/hash_object.rs");
    println!("{:?}", result)
}
