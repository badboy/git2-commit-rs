use git2;
use std::env;

/// Adopted from Cargo's `git/utils.rs`
/// See
/// <https://github.com/rust-lang/cargo/blob/31214eba27a935933f00702df189e764937d65af/src/cargo/sources/git/utils.rs#L389>
/// for the original version
///
/// Prepare the authentication callbacks for cloning a git repository.
///
/// The main purpose of this function is to construct the "authentication
/// callback" which is used to clone a repository. This callback will attempt to
/// find the right authentication on the system (without user input) and will
/// guide libgit2 in doing so.
///
/// The callback is provided `allowed` types of credentials, and we try to do as
/// much as possible based on that:
///
/// * Prioritize SSH keys from the local ssh agent as they're likely the most
///   reliable. The username here is prioritized from the credential
///   callback, then from whatever is configured in git itself, and finally
///   we fall back to the generic user of `git`.
///
/// * If a username/password is allowed, then we fallback to git2-rs's
///   implementation of the credential helper. This is what is configured
///   with `credential.helper` in git, and is the interface for the OSX
///   keychain, for example.
///
/// * After the above two have failed, we just kinda grapple attempting to
///   return *something*.
///
/// If any form of authentication fails, libgit2 will repeatedly ask us for
/// credentials until we give it a reason to not do so. To ensure we don't
/// just sit here looping forever we keep track of authentications we've
/// attempted and we don't try the same ones again.
pub fn with_authentication<F>(url: &str, cfg: &git2::Config, mut f: F)
                             -> Result<(), git2::Error>
    where F: FnMut(&mut git2::Credentials) -> Result<(), git2::Error>
{
    // We try a couple of different user names when cloning via ssh as there's a
    // few possibilities if one isn't mentioned, and these are used to keep
    // track of that.
    enum UsernameAttempt {
        Arg,
        CredHelper,
        Local,
        Git,
    }

    let mut cred_helper = git2::CredentialHelper::new(url);
    cred_helper.config(cfg);

    let mut attempted = git2::CredentialType::empty();
    let mut failed_cred_helper = false;
    let mut username_attempt = UsernameAttempt::Arg;
    let mut username_attempts = Vec::new();

    f(&mut |url, username, allowed| {
        let allowed = allowed & !attempted;

        // libgit2's "USERNAME" authentication actually means that it's just
        // asking us for a username to keep going. This is currently only really
        // used for SSH authentication and isn't really an authentication type.
        // The logic currently looks like:
        //
        //      let user = ...;
        //      if (user.is_null())
        //          user = callback(USERNAME, null, ...);
        //
        //      callback(SSH_KEY, user, ...)
        //
        // So if we have a USERNAME request we just pass it either `username` or
        // a fallback of "git". We'll do some more principled attempts later on.
        if allowed.contains(git2::USERNAME) {
            attempted = attempted | git2::USERNAME;
            return git2::Cred::username(username.unwrap_or("git"))
        }

        // If User and password in plaintext is allowed
        // _and_ we have a token set, we assume we're pushing to GitHub using this token
        if allowed.contains(git2::USER_PASS_PLAINTEXT) {
            attempted = attempted | git2::USER_PASS_PLAINTEXT;
            if let Ok(token) = env::var("GH_TOKEN") {
                return git2::Cred::userpass_plaintext(&token, "")
            }
        }

        // An "SSH_KEY" authentication indicates that we need some sort of SSH
        // authentication. This can currently either come from the ssh-agent
        // process or from a raw in-memory SSH key. Cargo only supports using
        // ssh-agent currently.
        //
        // We try a few different usernames here, including:
        //
        //  1. The `username` argument, if provided. This will cover cases where
        //     the user was passed in the URL, for example.
        //  2. The global credential helper's username, if any is configured
        //  3. The local account's username (if present)
        //  4. Finally, "git" as it's a common fallback (e.g. with github)
        if allowed.contains(git2::SSH_KEY) {
            loop {
                let name = match username_attempt {
                    UsernameAttempt::Arg => {
                        username_attempt = UsernameAttempt::CredHelper;
                        username.map(|s| s.to_string())
                    }
                    UsernameAttempt::CredHelper => {
                        username_attempt = UsernameAttempt::Local;
                        cred_helper.username.clone()
                    }
                    UsernameAttempt::Local => {
                        username_attempt = UsernameAttempt::Git;
                        env::var("USER").or_else(|_| env::var("USERNAME")).ok()
                    }
                    UsernameAttempt::Git => {
                        attempted = attempted | git2::SSH_KEY;
                        Some("git".to_string())
                    }
                };
                if let Some(name) = name {
                    let ret = git2::Cred::ssh_key_from_agent(&name);
                    username_attempts.push(name);
                    return ret
                }
            }
        }

        // Sometimes libgit2 will ask for a username/password in plaintext. This
        // is where Cargo would have an interactive prompt if we supported it,
        // but we currently don't! Right now the only way we support fetching a
        // plaintext password is through the `credential.helper` support, so
        // fetch that here.
        if allowed.contains(git2::USER_PASS_PLAINTEXT) {
            attempted = attempted | git2::USER_PASS_PLAINTEXT;
            let r = git2::Cred::credential_helper(cfg, url, username);
            failed_cred_helper = r.is_err();
            return r
        }

        // I'm... not sure what the DEFAULT kind of authentication is, but seems
        // easy to support?
        if allowed.contains(git2::DEFAULT) {
            attempted = attempted | git2::DEFAULT;
            return git2::Cred::default()
        }

        // Whelp, we tried our best
        Err(git2::Error::from_str("no authentication available"))
    })
}

pub fn fetch(repo: &git2::Repository, url: &str, refspec: &str) -> Result<(), git2::Error> {
    with_authentication(url, &try!(repo.config()), |f| {
        let mut cb = git2::RemoteCallbacks::new();
        cb.credentials(f);
        let mut remote = try!(repo.remote_anonymous(&url));
        let mut opts = git2::FetchOptions::new();
        opts.remote_callbacks(cb)
            .download_tags(git2::AutotagOption::All);
        try!(remote.fetch(&[refspec], Some(&mut opts), None));
        Ok(())
    })
}
