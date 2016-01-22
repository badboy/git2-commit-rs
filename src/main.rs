extern crate git2;
extern crate git2_commit;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;
use git2::Error;

const USAGE: &'static str = "
git2-commit

Usage:
  git2-commit [options]
  git2-commit [options] add <file>...
  git2-commit [options] commit <message>

Options:
  -r <path>, --repository=<path>  Path to the repository's working directory [default: .]
  -h, --help                      Show this screen.
";


#[derive(Debug,RustcDecodable)]
struct Args {
    arg_file: Vec<String>,

    arg_message: String,

    flag_repository: String,
    cmd_add: bool,
    cmd_commit: bool,
}

fn git_add(args: &Args) -> Result<(), Error> {
    let repo = &args.flag_repository;
    let files = &args.arg_file;

    git2_commit::add(repo, files)
}

fn git_commit(args: &Args) -> Result<(), Error> {
    let repo = &args.flag_repository;

    let signature = try!(git2_commit::get_signature());
    let message = &args.arg_message;

    git2_commit::commit(repo, &signature.name, &signature.email, message)
}

fn run(args: &Args) -> Result<(), Error> {

    if args.cmd_add {
        return git_add(args);
    }

    if args.cmd_commit {
        return git_commit(args);
    }

    Err(Error::from_str("Unknown command"))
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
