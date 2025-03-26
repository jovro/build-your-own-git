use anyhow::Context;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use fs2::FileExt;
use sha1::Digest;
use sha1::Sha1;
use std::ffi::CStr;
use std::fmt;
use std::fs;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub struct GitObject<R> {
    pub kind: Kind,
    pub expected_size: u64,
    pub reader: R,
}

impl GitObject<()> {
    pub fn blob_from_file(file: impl AsRef<Path>) -> anyhow::Result<GitObject<impl Read>> {
        let path = file.as_ref();
        let file = fs::OpenOptions::new().read(true).open(path)?;
        file.lock_exclusive()?;
        let stat = std::fs::metadata(path)?;
        let contents = std::fs::File::open(path)?;
        fs2::FileExt::unlock(&file)?;
        Ok(GitObject {
            kind: Kind::Blob,
            expected_size: stat.len(),
            reader: contents,
        })
    }

    pub fn read(hash: &str) -> anyhow::Result<GitObject<impl BufRead>> {
        let f = std::fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("Failed to open hash in objects")?;
        let z = ZlibDecoder::new(f);
        let mut z = BufReader::new(z);
        let mut buf = Vec::new();
        z.read_until(0, &mut buf)
            .context("Failed to read header in objects")?;
        let header = CStr::from_bytes_with_nul(&buf)?;
        let header = header.to_str()?;
        let Some((kind, size)) = header.split_once(' ') else {
            anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
        };
        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => unreachable!(),
        };
        let size = size
            .parse::<u64>()
            .context("File header size metadata is invalid: {size}")?;
        let z = z.take(size);
        Ok(GitObject {
            kind,
            expected_size: size,
            reader: z,
        })
    }
}

impl<R> GitObject<R>
where
    R: Read,
{
    pub fn write(mut self, writer: impl Write) -> anyhow::Result<[u8; 20]> {
        let writer = ZlibEncoder::new(writer, Compression::default());
        let mut writer = HashWriter {
            writer,
            hasher: Sha1::new(),
        };
        write!(writer, "{} {}\0", self.kind, self.expected_size)?;
        std::io::copy(&mut self.reader, &mut writer).context("Failed to copy data to writer")?;
        let _ = writer.writer.finish()?;
        let hash = writer.hasher.finalize();
        Ok(hash.into())
    }

    pub fn write_to_objects(self) -> anyhow::Result<[u8; 20]> {
        let tmp = ".tempfile";
        let hash = self
            .write(std::fs::File::create(tmp).context("construct temporary file for tree")?)
            .context("Failed to write tree object to object file")?;
        let hash_hex = hex::encode(hash);
        fs::create_dir_all(format!(".git/objects/{}/", &hash_hex[..2]))
            .context("Failed to create directories under objects")?;
        fs::rename(
            tmp,
            format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
        )
        .context("Failed to rename tempfile to target")?;
        Ok(hash)
    }
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buffer)?;
        self.hasher.update(&buffer[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
