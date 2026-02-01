use argon2::{
    self,
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};
use crate::domain::User;

pub fn get_argon2_instance() -> Argon2<'static> {
    //            Regarding Argon2 parameters:
    //            ----------------------------
    //            For API keys / tokens:
    //            - higher m is OK (e.g., 96 MB --> 98304 KiB, 128 MB --> 131072 KiB)
    //            - t can be lower
    //            - UX doesn’t matter as much
    //
    //            For low-RAM environments:
    //            - don’t go above ~32 MB (32768 KiB)
    //            - keep t ≥ 2

    let params = Params::new(
        65536, // m: memory in KiB (64 MB)
        3,     // t: iterations
        1,     // p: parallelism
        None,  // output length (None = default)
    )
    .expect("invalid Argon2 params");

    return Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = get_argon2_instance();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(user: Option<User>, password: &str) -> Option<User> {
    let argon2 = get_argon2_instance();

    // A valid dummy hash generated with the same Argon2 params.
    // This is used only when the user is not found.
    const DUMMY_HASH: &str =
        "$argon2id$v=19$m=65536,t=3,p=1$2aYZPLsX/K0wjEZ1Hy6leg$ZxY80K0Lq3nS/PKsOciRJodOH9u8BRVdiAhjKFDUbCE";

    // Get the user's hash, or the dummy hash if the user is None.
    // This avoids branching before the verification step.
    let hash_to_verify = user.as_ref().map_or(DUMMY_HASH, |u| u.password());

    // Parse the hash. If parsing fails (e.g., corrupted hash in DB), it's an automatic failure.
    let parsed_hash = match PasswordHash::new(hash_to_verify) {
        Ok(h) => h,
        Err(_) => {
            // If parsing fails, we still run a dummy verification to keep timing consistent.
            let dummy_parsed = PasswordHash::new(DUMMY_HASH).unwrap(); // This unwrap is safe.
            let _ = argon2.verify_password(b"dummy password", &dummy_parsed); // Intentionally ignore result
            return None; // Invalid hash means failure.
        }
    };

    // Perform the verification.
    let is_valid = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    // Only return the user if the password is valid AND the user actually existed.
    if is_valid && user.is_some() {
        user
    } else {
        None
    }
}