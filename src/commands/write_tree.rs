use crate::objects::{GitObject, Kind};
use anyhow::Context;
use std::cmp::Ordering;
use std::fs;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let dir = fs::read_dir(path)
        .with_context(|| format!("Failed to walk files in directory {}", path.display()))?;

    let mut entries = Vec::new();
    for entry in dir {
        let entry =
            entry.with_context(|| format!("Failed to parse file in {:?}", path.display()))?;
        let name = entry.file_name();
        let meta = entry
            .metadata()
            .context("Failed to get directory metadata")?;
        entries.push((entry, name, meta));
    }

    // See git's own tree.c for reference
    //
    entries.sort_unstable_by(|a, b| {
        let a_filename = &a.1;
        let a_filename = a_filename.as_encoded_bytes();
        let b_filename = &b.1;
        let b_filename = b_filename.as_encoded_bytes();
        let shorter_length = std::cmp::min(a_filename.len(), b_filename.len());
        let comparison_result = a_filename[..shorter_length].cmp(&b_filename[..shorter_length]);
        if comparison_result != Ordering::Equal {
            return comparison_result;
        }
        if a_filename.len() == b_filename.len() {
            return Ordering::Equal;
        }
        let c1 = if let Some(c) = a_filename.get(shorter_length).copied() {
            Some(c)
        } else if a.2.is_dir() {
            Some(b'/')
        } else {
            None
        };
        let c2 = if let Some(c) = b_filename.get(shorter_length).copied() {
            Some(c)
        } else if b.2.is_dir() {
            Some(b'/')
        } else {
            None
        };

        c1.cmp(&c2)
    });

    let mut tree_object = Vec::new();
    for (entry, file_name, meta) in entries {
        if file_name == ".git" {
            continue;
        }
        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if (meta.permissions().mode() & 0o111) != 0 {
            // has at least one executable bit set
            "100755"
        } else {
            "100644"
        };
        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                continue;
            };
            hash
        } else {
            let tmp = ".tempfile";
            let hash = GitObject::blob_from_file(&path)
                .context("Failed to open blob file")?
                .write(std::fs::File::create(tmp).context("Failed to open tempfile")?)
                .context("Failed to write to tempfile")?;
            let hash_hex = hex::encode(hash);
            fs::create_dir_all(format!(".git/objects/{}/", &hash_hex[..2]))
                .context("Failed to create directory for objects")?;
            std::fs::rename(
                tmp,
                format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
            )
            .context("Failed to rename tempfile to target")?;
            hash
        };
        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }

    if tree_object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            GitObject {
                kind: Kind::Tree,
                expected_size: tree_object.len() as u64,
                reader: Cursor::new(tree_object),
            }
            .write_to_objects()
            .context("Failed to write tree object")?,
        ))
    }
}

pub fn invoke() -> anyhow::Result<()> {
    let Some(hash) = write_tree_for(Path::new(".")).context("Failed to write tree for curdir")?
    else {
        anyhow::bail!("Cannot make a tree for empty hash");
    };

    println!("{}", hex::encode(hash));

    Ok(())
}
