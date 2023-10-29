use crate::{NameError, Result};


pub fn sanitize_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if name.is_empty() {
        Err(NameError::Empty)?;
    }

    Ok(name)
}