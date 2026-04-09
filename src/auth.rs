use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;

use crate::api::MeicanClient;
use crate::config;
use crate::models::Session;

pub async fn login(username: &str, password: Option<&str>) -> Result<()> {
    println!("{}", "Logging in to Meican...".cyan());

    let password = match password {
        Some(p) => p.to_string(),
        None => rpassword::prompt_password(format!("Password for {}: ", username))?,
    };

    let cookies = MeicanClient::login(username, &password).await?;

    let session = Session {
        cookies,
        created_at: Utc::now().to_rfc3339(),
        username: username.to_string(),
    };

    config::save_session(&session)?;
    println!("{}", "Login successful!".green().bold());
    Ok(())
}

pub fn logout() -> Result<()> {
    config::delete_session()?;
    println!("{}", "Logged out.".green());
    Ok(())
}

pub fn status() -> Result<()> {
    match config::load_session() {
        Ok(session) => {
            let created: DateTime<Utc> = session
                .created_at
                .parse()
                .unwrap_or_else(|_| Utc::now());
            let age = Utc::now().signed_duration_since(created);
            let hours = age.num_hours();
            let minutes = age.num_minutes() % 60;

            println!("{}", "Logged in".green().bold());
            println!("  User:        {}", session.username.cyan());
            println!("  Session age: {}h {}m", hours, minutes);
        }
        Err(_) => {
            println!("{}", "Not logged in".red().bold());
            println!("  Run `meican login` to authenticate.");
        }
    }
    Ok(())
}
