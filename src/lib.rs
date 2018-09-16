extern crate url;
extern crate git2;
extern crate log;

use std::error::Error as StdError;
use std::path::{Path, PathBuf};
use std::fs;
use git2::{Config, Repository, Signature, Error, PushOptions, RemoteCallbacks, BranchType,
           ResetType, ObjectType};
use url::Url;
use utils::{with_authentication, fetch};

mod utils;

pub struct Author {
    pub name: String,
    pub email: String,
}

pub fn get_signature() -> Result<Author, Error> {
    let config = try!(Config::open_default());
    let author = try!(config.get_string("user.name"));
    let email = try!(config.get_string("user.email"));
    Ok(Author {
        name: author.to_string(),
        email: email.to_string(),
    })
}


pub fn add<P: AsRef<Path>>(repo: &str, files: &[P], force: bool) -> Result<(), Error> {
    let repo = try!(Repository::open(repo));
    let mut index = try!(repo.index());

    for path in files {
        let path = path.as_ref();
        if force || !try!(repo.status_should_ignore(path)) {
            try!(index.add_path(path.as_ref()));
        }
    }

    index.write()
}

pub fn commit(repo: &str, name: &str, email: &str, message: &str) -> Result<(), Error> {
    let signature = try!(Signature::now(name, email));
    let update_ref = Some("HEAD");

    let repo = try!(Repository::open(repo));

    let oid = try!(repo.refname_to_id("HEAD"));
    let parent_commit = try!(repo.find_commit(oid));
    let parents = vec![&parent_commit];

    let mut index = try!(repo.index());
    let tree_oid = try!(index.write_tree());
    let tree = try!(repo.find_tree(tree_oid));

    repo
        .commit(update_ref, &signature, &signature, message, &tree, &parents)
        .map(|_| ())
}

pub fn tag(repo: &str, name: &str, email: &str, tag_name: &str, message: &str) -> Result<(), Error> {
    let repo = try!(Repository::open(repo));
    let obj = try!(repo.revparse_single("HEAD"));
    let signature = try!(Signature::now(name, email));

    repo.tag(tag_name, &obj, &signature, message, false)
        .map(|_| ())
}

fn ref_tag_or_branch(repo: &Repository, names: &[String]) -> Result<Vec<String>, Error> {
    names.iter().fold(Ok(vec![]), |acc, name| {
        acc.and_then(|mut v| {
            let tagnames = try!(repo.tag_names(Some(name)));

            let is_tag = tagnames.iter().any(|t| {
                match t {
                    None => false,
                    Some(ref t) => t == name
                }
            });

            let item = if is_tag {
                format!("refs/tags/{}", name)
            } else if repo.find_branch(name, BranchType::Local).is_ok() {
                format!("refs/heads/{}", name)
            } else {
                let s = format!("Could not find matching tag or branch: '{}'", name);
                return Err(Error::from_str(&s[..]));
            };

            v.push(item);
            Ok(v)
        })
    })
}

pub fn push(repo: &str, remote_name: &str, branches: &[String]) -> Result<(), Error> {
    let repo = try!(Repository::open(repo));
    let config = try!(repo.config());

    let remote = try!(repo.find_remote(remote_name));
    let remote_url = match remote.url() {
        Some(url) => url,
        None => return Err(Error::from_str(&format!("No remote URL found for '{}'", remote_name))),
    };

    with_authentication(remote_url, &config, |f| {
        let mut cbs = RemoteCallbacks::new();
        cbs.credentials(f);
        let mut opts = PushOptions::new();
        opts.remote_callbacks(cbs);

        let refs = try!(ref_tag_or_branch(&repo, branches));
        let refs = refs.iter().map(|r| &r[..]).collect::<Vec<_>>();
        let mut remote = try!(repo.remote_anonymous(remote_url));
        remote.push(&refs[..], Some(&mut opts))
    })
}

pub fn branch(repo: &str, branch_type: BranchType) -> Result<Vec<String>, Error> {
    let repo = try!(Repository::open(repo));

    let head = try!(repo.head());
    let short = head.shorthand().unwrap_or("empty");

    let mut v = vec![];
    if branch_type == BranchType::Local {
        if head.is_branch() {
            v.push(format!("* {}", short));
        } else {
            let oid = try!(head.target().ok_or(Error::from_str("Could not find head-target")));
            v.push(format!("* ({} detached at {})", short, oid));
        }
    }

    let branches = try!(repo.branches(Some(branch_type)));

    for branch in branches {
        if let Ok((branch, _)) = branch {
            let name = try!(branch.name());
            let name = try!(name.ok_or(Error::from_str("Could not find branch name")));

            if name != short {
                v.push(String::from(name));
            }
        }
    }

    Ok(v)
}

pub fn clone<S: AsRef<str>>(url: &str, directory: Option<S>) -> Result<(), Error> {
    let parsed_url = try!(Url::parse(url).map_err(|e| Error::from_str(e.description())));

    let dst = match directory {
        Some(dir) => PathBuf::from(dir.as_ref()),
        None => {
            let url_paths = match parsed_url.path_segments() {
                None => return Err(Error::from_str("URL has no path. Can't extract target directory")),
                Some(p) => p.collect::<Vec<_>>()
            };

            let len = url_paths.len();
            let last_path = &url_paths[len-1];
            let mut path = PathBuf::from(last_path);
            path.set_extension("");
            path
        }
    };

    if fs::metadata(&dst).is_ok() {
        return Err(Error::from_str("Target path exists."));
    }

    try!(fs::create_dir_all(&dst).map_err(|e| Error::from_str(e.description())));
    let repo = try!(git2::Repository::init(&dst));

    try!(fetch(&repo, url, "refs/heads/*:refs/heads/*"));
    let head = try!(repo.head());
    let head_obj = try!(head.peel(ObjectType::Commit));
    try!(repo.reset(&head_obj, ResetType::Hard, None));
    try!(repo.remote("origin", url));

    Ok(())
}
