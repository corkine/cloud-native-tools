use std::env;

pub const API_USER: &str = "API_USER";
pub const API_PASSWORD: &str = "API_PASSWORD";

pub fn get_env_var(key: &str) -> String {
    env::var(key).unwrap_or_default()
}