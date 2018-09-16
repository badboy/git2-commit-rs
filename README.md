# git2-commit

[![Build Status](https://travis-ci.org/badboy/git2-commit-rs.svg?branch=master)](https://travis-ci.org/badboy/git2-commit-rs)

A reimplementation of a few git commands.  
Based on [`git2-rs`](https://github.com/alexcrichton/git2-rs).

## Implemented

* `add`: Only files, no directories. With force mode.
* `commit`: Only the most basic message-inline version
* `tag`: Always annotated
* `push`: Only existing remotes and branches or tags. Branches or tags are detected.
* `branch`: Only listing branches (listing remote branches possible)
* `clone`: A simple clone is possible. No fancy options though. Target directory can be specified.

## Usage

```
git2-commit 0.1.0
Jan-Erik Rediger <janerik@fnordig.de>
git2-commit - Simple git commands, reimplemented.

USAGE:
    git2-commit [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --path <path>    Path to the repository's working directory [default: .]

SUBCOMMANDS:
    add       Add file contents to the index
    branch    List branches
    clone     Clone a repository
    commit    Record changes to a repository
    help      Prints this message or the help of the given subcommand(s)
    push      Push local commits to a remote
    tag       Create a tag
```
