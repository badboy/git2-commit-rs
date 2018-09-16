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
Usage:
  git2-commit [options]
  git2-commit [options] add [--force] <file>...
  git2-commit [options] commit <message>
  git2-commit [options] tag <tag-name> <tag-message>
  git2-commit [options] push <remote> <branches>...
  git2-commit [options] branch [--remotes]
  git2-commit [options] clone <clone-url> [<clone-directory>]

Options:
  -h, --help                Show this screen.
  -p <path>, --path=<path>  Path to the repository's working directory [default: .]
  -f, --force               Allow adding otherwise ignored files.
  -r, --remotes             List remote-tracking branches
```
