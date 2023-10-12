use messi::hash_object;

fn main() {
    println!("Hello, World!");
    //Este es un comentario para probar GitHub Actions
    let result = hash_object::store_file("src/hash_object.rs");
    println!("{:?}", result)
}
