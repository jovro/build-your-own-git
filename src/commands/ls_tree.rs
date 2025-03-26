use crate::objects::{GitObject, Kind};
use anyhow::Context;
use std::{
    ffi::CStr,
    io::{BufRead, Read, Write},
};

pub fn invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    let mut object = GitObject::read(tree_hash).context("Failed to read tree object")?;
    if let Kind::Tree = object.kind {
        let mut buf = Vec::new();
        let mut hashbuf = [0; 20];
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        loop {
            buf.clear();
            let n = object
                .reader
                .read_until(0, &mut buf)
                .context("Failed to read next entry")?;
            if n == 0 {
                break;
            }
            object
                .reader
                .read_exact(&mut hashbuf[..])
                .context("Failed to read entry hash")?;

            let mode_and_name = CStr::from_bytes_with_nul(&buf).context("invalid tree entry")?;
            // If on nightly toolchain
            // let mut bits = mode_and_name.to_bytes().split_once(|&b| b == b' ');
            let mut bits = mode_and_name.to_bytes().splitn(2, |&b| b == b' ');
            let mode = bits.next().expect("Split to always have one element");
            let name = bits
                .next()
                .ok_or_else(|| anyhow::anyhow!("Split failed, file name missing"))?;

            if name_only {
                stdout.write_all(name)?
            } else {
                let mode = std::str::from_utf8(mode).expect("File mode is always valid");
                let hash = hex::encode(hashbuf);
                let object = GitObject::read(&hash)?;
                write!(stdout, "{mode:0>6} {} {hash} ", object.kind)?;
                stdout.write_all(name)?
            }
            writeln!(stdout)?;
        }
    } else {
        eprintln!("Don't know how to traverse {} yet!", object.kind)
    }
    Ok(())
}
