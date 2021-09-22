use anyhow::Result;
use clap::{clap_app, crate_version};

mod config;
mod gitlab;
mod repo;

#[tokio::main]
async fn main() -> Result<()> {
    let app = clap_app!(rem =>
        (version: crate_version!())
        (author: "Hilmar Wiegand <me@hwgnd.de>")
        (about: "Remote bash script execution and library import")
        (@setting DeriveDisplayOrder)
        (@setting ColoredHelp)
        (@setting SubcommandRequiredElseHelp)

        (@subcommand repo =>
            (about: "Read and modify locally saved repositories")
            (@setting DeriveDisplayOrder)
            (@setting ColoredHelp)
            (@setting SubcommandRequiredElseHelp)

            (@subcommand list =>
                (about: "Lists all locally saved repositories")
                (@setting ColoredHelp)
            )

            (@subcommand add =>
                (about: "Adds a repository to the local repository list")
                (@setting ColoredHelp)
                (@arg NAME: +required "Local alias for the repository to add")
                (@arg URI: +required "URI of the repository to add")
                (@arg username: -u --username [USERNAME] "Username for the repository (if required)")
                (@arg password: -p --password [PASSWORD] "Password or token for the repository (if required)")
                (@arg password_env: --("password-env") [VAR_NAME] "Reads the password from the given var on access")
                (@arg password_stdin: --("password-stdin") "Reads the password or token from stdin")
            )

            (@subcommand check =>
                (about: "Checks whether a repository is accessible and prints out details about the repository")
                (@setting ColoredHelp)
                (@arg NAME: +required "Local alias for the repository to add")
            )

            (@subcommand remove =>
                (about: "Removes a repository from the local repository list")
                (@setting ColoredHelp)
                (@arg NAME: +required "Local alias for the repository to remove")
            )
        )

        (@subcommand import =>
            (about: "Imports a script and prints it to stdout")
            (@setting ColoredHelp)
            (@arg SCRIPT: +required "Script identifier in the format `<repo>:<script_path>`")
        )

        (@subcommand run =>
            (about: "Runs a script using the locally installed bash shell")
            (@setting ColoredHelp)
            (@arg SCRIPT: +required "Script identifier in the format `<repo>:<script_path>`")
        )
    );

    let _config = config::load_config().await?;
    let _args = app.get_matches();

    Ok(())
}
