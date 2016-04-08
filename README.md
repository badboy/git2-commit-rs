# git2-commit

[![Build Status](https://travis-ci.org/badboy/git2-commit-rs.svg?branch=master)](https://travis-ci.org/badboy/git2-commit-rs)
[![Clippy Linting Result](https://clippy.bashy.io/github/badboy/git2-commit-rs/master/badge.svg)](https://clippy.bashy.io/github/badboy/git2-commit-rs/master/log)

A reimplementation of a few git commands.  
Based on [`git2-rs`](https://github.com/alexcrichton/git2-rs).

## Implemented

* `add`: Only files, no directories. With force mode.
* `commit`: Only the most basic message-inline version
* `tag`: Always annotated
* `push`: Only existing remotes and branches or tags. Branches or tags are detected.
* `branch`: Only listing branches (listing remote branches possible)
