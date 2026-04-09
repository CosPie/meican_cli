use anyhow::Result;
use colored::Colorize;

const REPO_OWNER: &str = "CosPie";
const REPO_NAME: &str = "meican_cli";
const BIN_NAME: &str = "meican";

pub fn self_update() -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    println!(
        "{}",
        format!("Current version: v{}", current).cyan()
    );
    println!("Checking for updates...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(current)
        .show_download_progress(true)
        .show_output(true)
        .no_confirm(true)
        .build()?
        .update()?;

    if status.updated() {
        println!(
            "{}",
            format!("Updated to v{}!", status.version()).green().bold()
        );
    } else {
        println!("{}", "Already up to date.".green());
    }

    Ok(())
}
