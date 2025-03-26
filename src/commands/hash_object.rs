use crate::objects::GitObject;
use anyhow::Context;
use std::path::Path;

pub fn invoke(write: bool, file: &Path) -> anyhow::Result<()> {
    let object = GitObject::blob_from_file(file).context("Failed to open blob")?;
    let hash = if write {
        object
            .write_to_objects()
            .context("Failed to write to field")?
    } else {
        object
            .write(std::io::sink())
            .context("Failed to write to sink")?
    };

    println!("{}", hex::encode(hash));

    Ok(())
}
