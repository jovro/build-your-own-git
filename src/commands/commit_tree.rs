use crate::objects::{GitObject, Kind};
use anyhow::Context;
use std::fmt::Write;
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn write_commit(
    message: &str,
    tree_hash: &str,
    parent_hash: Option<&str>,
) -> anyhow::Result<[u8; 20]> {
    let mut buffer = String::new();
    writeln!(buffer, "tree {tree_hash}")?;
    if let Some(parent_hash) = parent_hash {
        writeln!(buffer, "parent {parent_hash}")?;
    }
    let name = "Firstname Lastname";
    let email = "first@last.com";
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get UNIX epoch time")?;
    writeln!(buffer, "author {name} <{email}> {} +0000", time.as_secs())?;
    writeln!(
        buffer,
        "committer {name} <{email}> {} +0000",
        time.as_secs()
    )?;
    writeln!(buffer)?;
    writeln!(buffer, "{message}")?;
    GitObject {
        kind: Kind::Commit,
        expected_size: buffer.len() as u64,
        reader: Cursor::new(buffer),
    }
    .write_to_objects()
    .context("Failed to write commit objects")
}

pub fn invoke(
    message: String,
    tree_hash: String,
    parent_hash: Option<String>,
) -> anyhow::Result<()> {
    let hash = write_commit(&message, &tree_hash, parent_hash.as_deref())
        .context("Failed to write commit")?;

    println!("{}", hex::encode(hash));

    Ok(())
}
