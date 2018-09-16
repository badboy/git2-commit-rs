extern crate git2;
extern crate git2_commit;
#[macro_use]
extern crate structopt;
extern crate log;
extern crate env_logger;

use structopt::StructOpt;
use git2::{Error, BranchType};

/// git2-commit - Simple git commands, reimplemented.
#[derive(Debug, StructOpt)]
struct Git {
    /// Path to the repository's working directory [default: .]
    #[structopt(short = "p", long = "path")]
    path: Option<String>,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Add file contents to the index
    #[structopt(name = "add")]
    Add {
        /// Allow adding otherwise ignored files
        #[structopt(short = "f", long = "force")]
        force: bool,
        /// Files to add content from
        files: Vec<String>,
    },

    /// Record changes to a repository
    #[structopt(name = "commit")]
    Commit {
        /// Use the given message as the commit message
        message: String,
    },

    /// Create a tag
    #[structopt(name = "tag")]
    Tag {
        /// Name of the new tag
        tag: String,
        /// Message for the new tag
        message: String,
    },

    /// Push local commits to a remote
    #[structopt(name = "push")]
    Push {
        /// Remote to push to
        remote: String,
        /// Branches to push
        branches: Vec<String>,

    },

    /// List branches
    #[structopt(name = "branch")]
    Branch {
        /// List remote-tracking branches
        #[structopt(short = "r", long = "remotes")]
        remotes: bool,
    },

    /// Clone a repository
    #[structopt(name = "clone")]
    Clone {
        /// URL to clone from
        url: String,
        /// Directory to clone to [default: .]
        directory: Option<String>,
    },
}

fn run(git: Git) -> Result<(), Error> {
    let repo = git.path.unwrap_or_else(|| ".".to_string());

    use Command::*;
    match git.cmd {
        Add { force, files } => {
            git2_commit::add(&repo, &files, force)
        },

        Commit { message } => {
            let signature = try!(git2_commit::get_signature());
            git2_commit::commit(&repo, &signature.name, &signature.email, &message)
        },

        Tag { tag, message } => {
            let signature = try!(git2_commit::get_signature());
            git2_commit::tag(&repo, &signature.name, &signature.email, &tag, &message)
        },

        Push { remote, branches } => {
            git2_commit::push(&repo, &remote, &branches)
        },

        Branch { remotes } => {
            let branch_type = if remotes {
                BranchType::Remote
            } else {
                BranchType::Local
            };
            let branches = git2_commit::branch(&repo, branch_type)?;
            for branch in branches {
                if branch.starts_with("* ") {
                    println!("{}", branch);
                } else {
                    println!("  {}", branch);
                }
            }
            Ok(())
        },

        Clone { url, directory } => {
            git2_commit::clone(&url, directory)
        },
    }
}

fn main() {
    env_logger::init();
    let git = Git::from_args();

    match run(git) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e.message()),
    }
}
