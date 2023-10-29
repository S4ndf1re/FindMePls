use crate::{NameError, Result};


pub fn name_rules(name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        Err(NameError::Empty)?;
    }

    Ok(())
}