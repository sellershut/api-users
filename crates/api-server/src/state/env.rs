use tracing::{debug, warn};

pub fn extract_variable(variable: &str, default: &str) -> String {
    let fallback = || -> String {
        debug!(var = variable, value = default, "[ENV] using default");
        default.to_owned()
    };

    match std::env::var(variable) {
        Ok(value) => {
            if value.trim().is_empty() {
                warn!(var = variable, "[ENV] variable is empty");
                fallback()
            } else {
                value
            }
        }
        Err(e) => match e {
            std::env::VarError::NotPresent => {
                warn!(variable = variable, "[ENV] Not present");
                fallback()
            }
            std::env::VarError::NotUnicode(_) => {
                warn!(variable = variable, "[ENV] Not unicode");
                fallback()
            }
        },
    }
}
