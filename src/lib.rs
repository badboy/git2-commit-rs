extern crate git2;

use std::path::Path;
use git2::{Repository, Signature, Error};

pub fn add<P: AsRef<Path>>(repo: &str, files: &[P]) -> Result<(), Error> {
    let repo = try!(Repository::open(repo));
    let mut index = try!(repo.index());

    for path in files {
        let _ = try!(index.add_path(path.as_ref()));
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

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
