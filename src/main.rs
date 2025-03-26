use anyhow::Context;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

pub mod commands;
pub mod objects;

/// A homebrew git implementation based on a challenge
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Provide contents or details of repository objects
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    /// Record changes to the repository (write + commit tree)
    Commit {
        #[clap(short = 'm')]
        message: String,
    },
    /// Create a new commit object
    CommitTree {
        #[clap(short = 'm')]
        message: String,

        #[clap(short = 'p')]
        parent_hash: Option<String>,

        tree_hash: String,
    },
    /// Compute object ID and optionally create an object from a file
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
    /// Initialise repository
    Init,
    /// List the contents of a tree object
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_hash: String,
    },
    /// Create a tree object from the current index
    WriteTree,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // CI/CD logging
    eprintln!("Logs from your program will appear here!");

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialised git directory")
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(pretty_print, &object_hash)?,
        Command::HashObject { write, file } => commands::hash_object::invoke(write, &file)?,
        Command::LsTree {
            name_only,
            tree_hash,
        } => commands::ls_tree::invoke(name_only, &tree_hash)?,
        Command::WriteTree => commands::write_tree::invoke()?,
        Command::CommitTree {
            message,
            tree_hash,
            parent_hash,
        } => commands::commit_tree::invoke(message, tree_hash, parent_hash)?,
        Command::Commit { message } => {
            let head_ref = std::fs::read_to_string(".git/HEAD").context("Failed to read HEAD")?;
            let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
                anyhow::bail!("In detached HEAD mode, cannot commit");
            };
            let head_ref = head_ref.trim();
            let parent_hash = std::fs::read_to_string(format!(".git/{head_ref}"))
                .with_context(|| format!("Failed to read parent hash for '{head_ref}'"))?;

            let Some(tree_hash) = commands::write_tree::write_tree_for(Path::new("."))
                .context("Failed to write tree for curdir")?
            else {
                eprintln!("Empty write tree!");
                return Ok(());
            };

            let commit_hash = commands::commit_tree::write_commit(
                &message,
                &hex::encode(tree_hash),
                Some(parent_hash.trim()),
            )
            .context("Failed to write commit")?;
            let commit_hash = hex::encode(commit_hash);

            std::fs::write(format!(".git/{head_ref}"), &commit_hash)
                .with_context(|| format!("Failed to update HEAD reference {head_ref}"))?;

            println!("HEAD is now at {commit_hash}");
        }
    }

    Ok(())
}
