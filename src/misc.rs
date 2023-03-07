use error_stack::{Context, IntoReport, Result, ResultExt};
use std::env;

#[derive(Debug)]
pub struct EnvironmentError;

impl std::fmt::Display for EnvironmentError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("Environment Error: An error occurred while trying to read the environment")
    }
}

impl Context for EnvironmentError {}

pub fn get_env(key: &str) -> Result<String, EnvironmentError> {
    let env_var = env::var(key)
        .into_report()
        .attach_printable_lazy(|| format!("Failed to read environment variable: {}", key))
        .change_context(EnvironmentError)?;
    Ok(env_var)
}

pub fn bool_to_emoji(bool: bool) -> &'static str {
    if bool {
        "✅"
    } else {
        "❌"
    }
}
