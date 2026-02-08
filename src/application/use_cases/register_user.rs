/// RegisterUser use case

use crate::application::dto::{RegisterUserInput, RegisterUserOutput};
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::UserRepository;
use crate::domain::entities::user::User;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Use case for registering a new user
pub struct RegisterUser<'a> {
    user_repo: &'a mut dyn UserRepository,
}

impl<'a> RegisterUser<'a> {
    pub fn new(user_repo: &'a mut dyn UserRepository) -> Self {
        Self { user_repo }
    }

    pub fn execute(&mut self, input: RegisterUserInput) -> AppResult<RegisterUserOutput> {
        // Check if username already exists
        if self.user_repo.exists_by_username(&input.username) {
            return Err(AppError::UserAlreadyExists(input.username));
        }

        // Hash the password using argon2
        let password_hash = Self::hash_password(&input.password)
            .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?;

        // Create the user
        let user = User::new(
            input.username.clone(),
            input.email,
            password_hash,
            input.timezone,
        );

        // Save the user
        let user_id = self.user_repo.save(user)?;

        Ok(RegisterUserOutput {
            user_id,
            username: input.username,
        })
    }

    /// Hash a password using argon2
    fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(password_hash)
    }

    /// Verify a password against a hash (for future login use case)
    #[allow(dead_code)]
    pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(password_hash)?;
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "test_password_123";
        let hash = RegisterUser::hash_password(password).unwrap();
        
        assert!(RegisterUser::verify_password(password, &hash).unwrap());
        assert!(!RegisterUser::verify_password("wrong_password", &hash).unwrap());
    }
}
