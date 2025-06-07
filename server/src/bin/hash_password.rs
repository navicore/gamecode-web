use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};

fn main() {
    let password = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "gamecode".to_string());
    
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => {
            println!("Password: {}", password);
            println!("Hash: {}", hash);
            println!("\nAdd this to your config/default.toml:");
            println!("password_hash = \"{}\"", hash);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}