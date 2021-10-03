# rem

`rem` is a tool for managing **rem**ote bash scripts.

It is meant to be used in CI/CD contexts and for local automation purposes. `rem` can import script libraries into new scripts, or run scripts directly in your command line. 

## Example

```bash
# Add a github repository
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

`rem` is written in Rust and can be installed using the Cargo package manager.

```bash
cargo install rem-bash
```

If you don't have Rust installed, [rustup](https://rustup.rs/) is recommended. You'll need the latest nightly version of the Rust toolchain:

```bash
# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Switch to nightly
rustup default nightly
```

### Script sources

Right now, github and gitlab are supported through the API. You can save any number of scripts in your local repository list simply by providing the URL and giving them an alias:

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

Your added repositories will be saved at `$HOME/.remconf.toml`. It is recommended to use the `--password-env` option so you don't accidentally leave any plaintext passwords in your bash history.  
In CI/CD contexts, this is also the preferred way since you can safely build docker images with configured repos in them. The only thing stored in the configuration will be the name of the variable the token will be read from.

### TODO

The tool is in a usable state right now, but there's a few things missing for it to be reliable and useful in more contexts. Here are the things I have planned:

* Support raw git repositories from any providers
* Support scripts from non-git sources
* Allow validating saved repositories
* Add local caching for offline contexts
* Validate scripts before they are run (shebang, static analysis, arbitrary checks)
* Add tests
