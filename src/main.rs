extern crate git2;
extern crate git2_commit;

use git2_commit::add_and_commit;

fn main() {
    add_and_commit(".", "jer", "foobar", "first commit", &[]).unwrap();
}
