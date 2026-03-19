mod bitbucket;
mod config;
mod filter;
mod llm;

use crate::config::DiagnosisConfig;
use anyhow::{Context, Result};
use bitbucket::BitbucketClient;
use clap::{Parser, Subcommand};
use config::Config;
use dotenv::dotenv;
use filter::apply_filters;
use std::env;
use std::io::Write;
use std::process::{Command as ProcessCommand, Stdio};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Configure defaults for workspace and repo
    Configure {
        #[arg(long)]
        workspace: Option<String>,
        #[arg(long)]
        repo: Option<String>,
    },
    /// Diagnose a failed pipeline run
    Diagnose {
        /// The repository workspace
        #[arg(long, short)]
        workspace: Option<String>,

        /// The repository slug
        #[arg(long, short)]
        repo: Option<String>,

        /// The pipeline ID (uuid) to diagnose. If not provided, finds the latest failure.
        #[arg(long, short)]
        pipeline_id: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let args = Args::parse();

    // Load configuration
    let cfg: Config =
        confy::load("bbpipelinediag", None).context("Failed to load configuration")?;

    match args.command {
        Commands::Configure { workspace, repo } => {
            let mut updated = false;
            let mut new_cfg = cfg.clone();

            if let Some(w) = workspace {
                new_cfg.workspace = Some(w);
                updated = true;
            }
            if let Some(r) = repo {
                new_cfg.repo = Some(r);
                updated = true;
            }

            let path = confy::get_configuration_file_path("bbpipelinediag", None)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            if updated {
                confy::store("bbpipelinediag", None, &new_cfg)
                    .context("Failed to save configuration")?;
                println!("Configuration updated at {}", path);
            } else {
                println!("Current configuration file: {}", path);
                println!("Use --workspace or --repo to update defaults.");
            }
        }
        Commands::Diagnose {
            workspace,
            repo,
            pipeline_id,
        } => {
            let workspace = workspace
                .or_else(|| cfg.workspace.clone())
                .context("Workspace is required. Set via --workspace or in config.")?;
            let repo = repo
                .or_else(|| cfg.repo.clone())
                .context("Repo slug is required. Set via --repo or in config.")?;

            let username = env::var("BITBUCKET_USERNAME")
                .ok()
                .or_else(|| cfg.bitbucket_username.clone())
                .context("BITBUCKET_USERNAME env var or bitbucket_username in config not set")?;
            let app_password = env::var("BITBUCKET_APP_PASSWORD")
                .ok()
                .or_else(|| cfg.bitbucket_app_password.clone())
                .context(
                    "BITBUCKET_APP_PASSWORD env var or bitbucket_app_password in config not set",
                )?;

            let client = BitbucketClient::new(username, app_password);

            println!("Diagnosing {}/{}...", workspace, repo);

            let pipeline_uuid = match pipeline_id {
                Some(id) => id,
                None => {
                    println!("Finding latest failed pipeline...");
                    let pipelines = client.get_pipelines(&workspace, &repo).await?;
                    let failed = pipelines.values.into_iter().find(|p| {
                        p.state
                            .result
                            .as_ref()
                            .map(|r| r.name == "FAILED")
                            .unwrap_or(false)
                    });

                    match failed {
                        Some(p) => {
                            println!(
                                "Found failed pipeline #{} (UUID: {})",
                                p.build_number, p.uuid
                            );
                            p.uuid
                        }
                        None => {
                            println!("No failed pipelines found in recent history.");
                            return Ok(());
                        }
                    }
                }
            };

            // Get steps
            let steps = client.get_steps(&workspace, &repo, &pipeline_uuid).await?;
            let failed_steps: Vec<_> = steps
                .values
                .into_iter()
                .filter(|s| {
                    s.state
                        .result
                        .as_ref()
                        .map(|r| r.name == "FAILED")
                        .unwrap_or(false)
                })
                .collect();

            if failed_steps.is_empty() {
                println!(
                    "Pipeline {} is marked as failed but no failed steps found (maybe a setup error?).",
                    pipeline_uuid
                );
                return Ok(());
            }

            println!("Found {} failed steps.", failed_steps.len());

            let range_header = cfg.http_range.as_ref().map(|r| {
                match (r.start, r.end) {
                    (Some(s), Some(e)) if s >= 0 => format!("{}-{}", s, e),
                    (Some(s), None) if s >= 0 => format!("{}-", s),
                    (Some(s), None) if s < 0 => format!("{}", s), // e.g. -10240 for last 10KiB
                    _ => "".to_string(),
                }
            }).filter(|s| !s.is_empty());

            for step in failed_steps {
                println!("\n--- Step: {} ({}) ---", step.name, step.uuid);
                let log = client
                    .get_step_log(&workspace, &repo, &pipeline_uuid, &step.uuid, range_header.clone())
                    .await?;

                let filtered_log = apply_filters(&log, &cfg.filters)?;

                if let Some(diag_cfg) = &cfg.diagnosis {
                    match diag_cfg {
                        DiagnosisConfig::Llm { config } => {
                            println!("\n--- Sending to LLM ---");
                            if let Err(e) = llm::diagnose(&filtered_log, config).await {
                                eprintln!("LLM Diagnosis failed: {:#}", e);
                            }
                        }
                        DiagnosisConfig::Exec { command } => {
                            println!("\n--- Piping to: {} ---", command);
                            let mut child = ProcessCommand::new("sh")
                                .arg("-c")
                                .arg(command)
                                .stdin(Stdio::piped())
                                .spawn()
                                .context("Failed to spawn shell command")?;

                            if let Some(mut stdin) = child.stdin.take() {
                                if let Err(e) = stdin.write_all(filtered_log.as_bytes()) {
                                    eprintln!("Failed to write to stdin: {}", e);
                                }
                            }

                            match child.wait() {
                                Ok(status) => println!("Command exited with: {}", status),
                                Err(e) => eprintln!("Command failed to wait: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
