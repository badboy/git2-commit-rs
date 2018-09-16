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
    let config = Config::open_default()?;
    let author = config.get_string("user.name")?;
    let email = config.get_string("user.email")?;
    Ok(Author {
        name: author.to_string(),
        email: email.to_string(),
    })
}


pub fn add<P: AsRef<Path>>(repo: &str, files: &[P], force: bool) -> Result<(), Error> {
    let repo = Repository::open(repo)?;
    let mut index = repo.index()?;

    for path in files {
        let path = path.as_ref();
        if force || !repo.status_should_ignore(path)? {
            index.add_path(path.as_ref())?;
        }
    }

    index.write()
}

pub fn commit(repo: &str, name: &str, email: &str, message: &str) -> Result<(), Error> {
    let signature = Signature::now(name, email)?;
    let update_ref = Some("HEAD");

    let repo = Repository::open(repo)?;

    let oid = repo.refname_to_id("HEAD")?;
    let parent_commit = repo.find_commit(oid)?;
    let parents = vec![&parent_commit];

    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    repo
        .commit(update_ref, &signature, &signature, message, &tree, &parents)
        .map(|_| ())
}

pub fn tag(repo: &str, name: &str, email: &str, tag_name: &str, message: &str) -> Result<(), Error> {
    let repo = Repository::open(repo)?;
    let obj = repo.revparse_single("HEAD")?;
    let signature = Signature::now(name, email)?;

    repo.tag(tag_name, &obj, &signature, message, false)
        .map(|_| ())
}

fn ref_tag_or_branch(repo: &Repository, names: &[String]) -> Result<Vec<String>, Error> {
    names.iter().fold(Ok(vec![]), |acc, name| {
        acc.and_then(|mut v| {
            let tagnames = repo.tag_names(Some(name))?;

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
    let repo = Repository::open(repo)?;
    let config = repo.config()?;

    let remote = repo.find_remote(remote_name)?;
    let remote_url = match remote.url() {
        Some(url) => url,
        None => return Err(Error::from_str(&format!("No remote URL found for '{}'", remote_name))),
    };

    with_authentication(remote_url, &config, |f| {
        let mut cbs = RemoteCallbacks::new();
        cbs.credentials(f);
        let mut opts = PushOptions::new();
        opts.remote_callbacks(cbs);

        let refs = ref_tag_or_branch(&repo, branches)?;
        let refs = refs.iter().map(|r| &r[..]).collect::<Vec<_>>();
        let mut remote = repo.remote_anonymous(remote_url)?;
        remote.push(&refs[..], Some(&mut opts))
    })
}

pub fn branch(repo: &str, branch_type: BranchType) -> Result<Vec<String>, Error> {
    let repo = Repository::open(repo)?;

    let head = repo.head()?;
    let short = head.shorthand().unwrap_or("empty");

    let mut v = vec![];
    if branch_type == BranchType::Local {
        if head.is_branch() {
            v.push(format!("* {}", short));
        } else {
            let oid = head.target().ok_or(Error::from_str("Could not find head-target"))?;
            v.push(format!("* ({} detached at {})", short, oid));
        }
    }

    let branches = repo.branches(Some(branch_type))?;

    for branch in branches {
        if let Ok((branch, _)) = branch {
            let name = branch.name()?;
            let name = name.ok_or(Error::from_str("Could not find branch name"))?;

            if name != short {
                v.push(String::from(name));
            }
        }
    }

    Ok(v)
}

pub fn clone<S: AsRef<str>>(url: &str, directory: Option<S>) -> Result<(), Error> {
    let parsed_url = Url::parse(url).map_err(|e| Error::from_str(e.description()))?;

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

    fs::create_dir_all(&dst).map_err(|e| Error::from_str(e.description()))?;
    let repo = git2::Repository::init(&dst)?;

    fetch(&repo, url, "refs/heads/*:refs/heads/*")?;
    let head = repo.head()?;
    let head_obj = head.peel(ObjectType::Commit)?;
    repo.reset(&head_obj, ResetType::Hard, None)?;
    repo.remote("origin", url)?;

    Ok(())
}
