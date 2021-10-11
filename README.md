# rem

`rem` is a tool for managing **rem**ote bash scripts.

It is meant to be used in CI/CD contexts and for local automation purposes. `rem` can import script libraries into new scripts, or run scripts directly in your command line. 

## Example

```bash
# Run a script from a git repository
rem run git@github.com:JosefZIla/bash2048:bash2048.sh

# Save a github repository
rem repo add ansi github.com/fidian/ansi

# Import the repository
source <(rem import ansi:ansi)
echo "$(ansi --green SUCCESS:) We can now use the imported functions!"

# Add your private repositories (password will be read from $TOKEN)
rem repo add ci gitlab.com:mycompany/ci-scripts --password-env TOKEN

# Run your scripts whenever you need them
rem run ci:generate-coverage.sh ./tests/*

# Pin your scripts to a specific ref
rem run ci@v1.2.2:upload-results.sh
```

## How to install

You can directly install one of the released binaries:

```bash
# dynamically linked version
curl -sLo rem https://github.com/happenslol/rem/releases/download/v0.3.0/rem

# musl version (for alpine, etc.)
curl -sLo rem https://github.com/happenslol/rem/releases/download/v0.3.0/rem-musl

chmod +x rem
mv rem /usr/bin
```

`rem` is written in Rust and can also be installed using the Cargo package manager:

```bash
cargo install rem-bash
```

If you don't have Rust installed, [rustup](https://rustup.rs/) is recommended. You'll need the latest nightly version of the Rust toolchain:

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install nightly components
rustup install nightly

# Switch default toolchain to nightly
rustup default nightly
```

Now you can either use the above command to install `rem` directly, or clone the repo and build it yourself:

```bash
git clone git@github.com:happenslol/rem
cd rem
cargo install --path .
```

### Script sources

#### Git

You can access scripts either directly from any git repository, or through the API of the implemented providers. To run the script directly, simply paste the URL you would usually use to clone the repository over SSH, and attach the path of the script in the repository with a colon:

```
# Running /util/my-script.bash from the repository git@github.com:user/scripts:
rem run git@github.com:user/scripts:util/my-script.bash

# Using a specific ref:
rem run git@github.com:user/scripts@v1.0:util/my-script.bash
```

This command will make a shallow clone of the repository in `$HOME/.cache/rem`, and run the specified script from there. `rem` will shell out and use your actual `git` executable, so that you don't need any extra authentication.

**Running the command a second time will use the cached version of the repository.** This means that if you're running a script from the `HEAD` ref (which the command defaults to), you might be executing a stale script. You can however force a fresh download by passing `-f (--fresh)` to either `import` or `run`.

Using raw git scripts is recommended if you're running scripts locally on your machine, on a non-regular basis. Remember that you can always inspect the contents of a script without executing it by running `rem import` first. **You should always make sure to inspect scripts from untrusted sources before running them!**

#### API

Right now, the github and gitlab APIs are supported. You can save any number of scripts in your local repository list simply by providing the URL and giving them an alias:

```bash
# Use either a short version
rem repo add github.com/me/myscripts

# Or the full URL
rem repo add scripts https://github.com/me/myscripts

# Provide a token/password through stdin
cat my-token.txt | rem repo add private github.com/me/privatescripts --password-stdin

# Or read it from a variable everytime you run the script
export MY_TOKEN="$(cat my-token.txt)"
rem repo add private github.com/me/privatescripts --password-env

# List your local repositories
rem repo ls

# And remove them
rem repo rm private
```

Using API sourced scripts is recommended for CI/CD contexts.

Your added repositories will be saved at `$HOME/.remconf.toml`. It is recommended to use the `--password-env` option so you don't accidentally leave any plaintext passwords in your bash history.  
In CI/CD contexts, this is also the preferred way since you can safely build docker images with configured repos in them. The only thing stored in the configuration will be the name of the variable the token will be read from.

### TODO

The tool is in a usable (and hopefully useful) state right now, but there's a few things missing for it to be reliable and useful in more contexts. Here are the things I have planned:

* Support scripts from non-git sources
* Allow validating saved repositories
* Add local caching for offline contexts
* Validate scripts before they are run (shebang, static analysis, arbitrary checks)
* Add tests
