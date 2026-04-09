mod api;
mod auth;
mod commands;
mod config;
mod display;
mod error;
mod models;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Meal {
    Breakfast,
    Lunch,
    Dinner,
}

#[derive(Parser)]
#[command(name = "meican", version, about = "Meican (美餐) CLI - Order meals from your terminal")]
struct Cli {
    /// Output as table instead of JSON
    #[arg(long, global = true)]
    table: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Login to Meican with username and password
    Login {
        /// Your Meican username (email)
        username: String,

        /// Your Meican password (if omitted, will be prompted interactively)
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Logout and clear saved session
    Logout,

    /// Check current login status
    Status,

    /// Show today's meal slots and orders
    Today,

    /// Show calendar for a date range
    Calendar {
        /// Start date (YYYY-MM-DD)
        begin: String,

        /// End date (YYYY-MM-DD)
        end: String,
    },

    /// List available dishes for a meal slot
    Dishes {
        /// Meal type: breakfast, lunch, dinner
        meal: Option<Meal>,

        /// Tab unique ID (advanced, overrides meal)
        #[arg(long)]
        tab: Option<String>,
    },

    /// List available restaurants for a meal slot
    Restaurants {
        /// Meal type: breakfast, lunch, dinner
        meal: Option<Meal>,

        /// Tab unique ID (advanced, overrides meal)
        #[arg(long)]
        tab: Option<String>,
    },

    /// List delivery addresses
    Addresses,

    /// Place an order
    Order {
        /// Meal type: breakfast, lunch, dinner
        meal: Option<Meal>,

        /// Tab unique ID (advanced, overrides meal)
        #[arg(long)]
        tab: Option<String>,

        /// Dish ID to order
        #[arg(long)]
        dish: String,
    },

    /// Cancel an existing order
    Cancel {
        /// Meal type: breakfast, lunch, dinner
        meal: Option<Meal>,

        /// Order unique ID (advanced, overrides meal)
        #[arg(long)]
        id: Option<String>,
    },

    /// Show order history
    History {
        /// Number of days to look back (default: 30)
        #[arg(long, default_value = "30")]
        days: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let table = cli.table;

    match cli.command {
        Commands::Login { username, password } => auth::login(&username, password.as_deref()).await,
        Commands::Logout => auth::logout(),
        Commands::Status => auth::status(),
        Commands::Today => commands::menu::today(table).await,
        Commands::Calendar { begin, end } => commands::menu::calendar(&begin, &end, table).await,
        Commands::Dishes { meal, tab } => commands::menu::dishes(meal, tab.as_deref(), table).await,
        Commands::Restaurants { meal, tab } => {
            commands::menu::restaurants(meal, tab.as_deref(), table).await
        }
        Commands::Addresses => commands::menu::addresses(table).await,
        Commands::Order { meal, tab, dish } => {
            commands::order::add_order(meal, tab.as_deref(), &dish, table).await
        }
        Commands::Cancel { meal, id } => {
            commands::order::cancel_order(meal, id.as_deref()).await
        }
        Commands::History { days } => commands::history::history(days, table).await,
    }
}
