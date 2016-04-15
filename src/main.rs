extern crate git2;
extern crate git2_commit;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;
use git2::{Error, BranchType};

const USAGE: &'static str = "git2-commit

Usage:
  git2-commit [options]
  git2-commit [options] add [--force] <file>...
  git2-commit [options] commit <message>
  git2-commit [options] tag <tag-name> <tag-message>
  git2-commit [options] push <remote> <branches>...
  git2-commit [options] branch [--remotes]
  git2-commit [options] clone <clone-url> [<clone-directory>]

Options:
  -f, --force               Allow adding otherwise ignored files.
  -p <path>, --path=<path>  Path to the repository's working directory [default: .]
  -h, --help                Show this screen.
  -r, --remotes             List remote-tracking branches
";


#[derive(Debug,RustcDecodable)]
struct Args {
    arg_file: Vec<String>,

    arg_message: String,

    arg_tag_name: String,
    arg_tag_message: String,

    arg_remote: String,
    arg_branches: Vec<String>,

    arg_clone_url: String,
    arg_clone_directory: Option<String>,

    flag_force: bool,
    flag_path: String,

    flag_remotes: bool,

    cmd_add: bool,
    cmd_commit: bool,
    cmd_tag: bool,
    cmd_push: bool,
    cmd_branch: bool,
    cmd_clone: bool,
}

fn git_add(args: &Args) -> Result<(), Error> {
    let repo = &args.flag_path;
    let files = &args.arg_file;

    git2_commit::add(repo, files, args.flag_force)
}

fn git_commit(args: &Args) -> Result<(), Error> {
    let repo = &args.flag_path;

    let signature = try!(git2_commit::get_signature());
    let message = &args.arg_message;

    git2_commit::commit(repo, &signature.name, &signature.email, message)
}

fn git_tag(args: &Args) -> Result<(), Error> {
    let repo = &args.flag_path;

    let signature = try!(git2_commit::get_signature());
    let tag_name = &args.arg_tag_name;
    let tag_message = &args.arg_tag_message;
    git2_commit::tag(repo, &signature.name, &signature.email, tag_name, tag_message)
}

fn git_push(args: &Args) -> Result<(), Error> {
    let repo = &args.flag_path;

    let remote = &args.arg_remote;
    let branches = &args.arg_branches;
    git2_commit::push(repo, remote, branches)
}

fn git_branch(args: &Args) -> Result<(), Error> {
    let repo = &args.flag_path;
    let branch_type = match args.flag_remotes {
        false => BranchType::Local,
        true => BranchType::Remote,
    };
    git2_commit::branch(repo, branch_type)
}

fn git_clone(args: &Args) -> Result<(), Error> {
    let url = &args.arg_clone_url;
    let directory = args.arg_clone_directory
        .as_ref()
        .map(|s| &s[..])
        .clone();

    git2_commit::clone(url, directory)
}

fn run(args: &Args) -> Result<(), Error> {

    if args.cmd_add {
        return git_add(args);
    }

    if args.cmd_commit {
        return git_commit(args);
    }

    if args.cmd_tag {
        return git_tag(args);
    }

    if args.cmd_push {
        return git_push(args);
    }

    if args.cmd_branch {
        return git_branch(args);
    }

    if args.cmd_clone {
        return git_clone(args);
    }

    println!("{}", USAGE);

    Ok(())
}

fn main() {
    let args : Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e.message()),
    }
}
