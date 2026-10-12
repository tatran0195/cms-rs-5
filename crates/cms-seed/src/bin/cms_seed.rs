use std::process::ExitCode;

use clap::Parser;
use cms_seed::{is_seeded, seed_dev};
use sqlx::postgres::PgPoolOptions;

#[derive(Parser, Debug)]
#[command(
    name = "cms_seed",
    author = "CMS Team",
    version = "0.1.0",
    about = "Database fixture generator for development and staging"
)]
struct Cli {
    /// PostgreSQL connection URL (or via DATABASE_URL env var).
    #[arg(short, long)]
    database_url: Option<String>,

    /// Fixture profile to apply (options: "dev", "minimal").
    #[arg(short, long, default_value = "dev")]
    profile: String,

    /// Force execution even if the database is already seeded.
    #[arg(short, long)]
    force: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let _ = tracing_subscriber::fmt::try_init();
    let args = Cli::parse();

    let db_url = args
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok());

    let db_url = match db_url {
        Some(url) if !url.trim().is_empty() => url,
        _ => {
            eprintln!("Error: DATABASE_URL must be provided via argument or environment variable");
            return ExitCode::FAILURE;
        }
    };

    println!("Connecting to database...");
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Database connection failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    if !args.force {
        match is_seeded(&pool).await {
            Ok(true) => {
                println!("Database is already initialized and seeded. Use --force to re-seed.");
                return ExitCode::SUCCESS;
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("Failed to check seed status: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    println!("Applying fixture profile: '{}'...", args.profile);
    match seed_dev(&pool).await {
        Ok(res) => {
            println!("Database seeding completed successfully!");
            println!("  Users seeded:       {}", res.users_count);
            println!("  Projects seeded:    {}", res.projects_count);
            println!("  Pages seeded:       {}", res.pages_count);
            println!("  Memberships seeded: {}", res.members_count);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Seeding failed: {e}");
            ExitCode::FAILURE
        }
    }
}
