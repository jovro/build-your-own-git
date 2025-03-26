use crate::objects::{GitObject, Kind};
use anyhow::Context;

pub fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    if pretty_print {
        eprintln!("Pretty printing is not supported (quite yet)")
    }

    let mut object = GitObject::read(object_hash).context("Parse object from hash")?;
    if let Kind::Blob = object.kind {
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        std::io::copy(&mut object.reader, &mut stdout).context("Write .git/objects file")?;
    } else {
        eprintln!("Don't know how to print object {} yet!", object.kind)
    }

    Ok(())
}
