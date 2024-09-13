use log::debug;
use rand::{Rng, SeedableRng};
use sha2::{Digest, Sha256};

/// Generate a random salt for password hashing
fn salt_pw(seed: u64) -> String {
    debug!("Generating salt for password hashing");
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let salt: String = (0..32)
        .map(|_| {
            let choice = rng.gen_range(0..3);
            match choice {
                0 => rng.gen_range(b'A'..=b'Z') as char,
                1 => rng.gen_range(b'a'..=b'z') as char,
                2 => rng.gen_range(b'0'..=b'9') as char,
                _ => unreachable!(),
            }
        })
        .collect();
    debug!("Salting complete");
    salt
}

/// Generate a password hash
#[must_use]
pub fn pw_hasher(password: &str) -> String {
    debug!("Hashing password");

    let salt = salt_pw(rand::random::<u64>());

    let hasher = Sha256::default();
    let _ = hasher
        .clone()
        .chain_update(format!("{password}: {salt}").as_bytes());
    let hash = hasher.finalize();
    let hash_str = format!("{hash:x}",);
    debug!("Hashed password: {}", hash_str);
    hash_str
}
