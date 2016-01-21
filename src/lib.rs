extern crate git2;

use git2::{Repository, Signature, Error};

struct User {
    email: String,
    name: String,
    time: String
}

pub fn add_and_commit(repo: &str, name: &str, email: &str, message: &str, files: &[&str]) -> Result<(), Error> {
    let signature = try!(Signature::now(name, email));
    let update_ref = Some("HEAD");

    let repo = try!(Repository::open(repo));

    let oid = try!(repo.refname_to_id("HEAD"));
    let commit = try!(repo.find_commit(oid));

    let tree = try!(commit.tree());
    let parents = vec![&commit];

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
