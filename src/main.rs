use crate::aws::loader::LoadSecrets;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;
use aws_sdk_ssm::Client as SsmClient;
use clap::{Parser, Subcommand};

mod aws;
mod vector;

/// A helper tool for Vector to retrieve secrets from AWS SSM Parameter Store and AWS Secrets
/// Manager using the exec backend.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Change endpoint URL for the command.
    #[arg(short, long)]
    endpoint_url: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Get secrets from AWS Systems Manager Parameter Store.
    Ssm {},
    /// Get secrets from AWS Secrets Manager.
    Secretsmanager {},
}

#[tokio::main]
async fn main() {
    // Parse the CLI arguments.
    let cli = Cli::parse();

    // Parse the JSON from stdin into a SecretsToFetch struct. It assumes that Vector will always
    // provide valid JSON, so we can simply unwrap the result. Probably should implement proper
    // pattern matching for the result here at some point instead.
    let secrets_to_fetch: vector::SecretsToFetch =
        serde_json::from_reader(std::io::stdin()).unwrap();

    // Load the AWS SDK config using the default credential provider chain.
    let aws_sdk_config = aws_config::load_from_env().await;

    // Run the command.
    let secrets_loader: Box<dyn LoadSecrets> = match &cli.command {
        Commands::Ssm {} => {
            let mut config_builder = aws_sdk_ssm::config::Builder::from(&aws_sdk_config);
            if cli.endpoint_url.is_some() {
                config_builder = config_builder.endpoint_url(cli.endpoint_url.unwrap());
            }
            let config = config_builder.build();
            Box::new(aws::ssm::SsmSecretsLoader::new(
                SsmClient::from_conf(config),
                // Always decrypt SecureString parameters.
                true,
            ))
        }
        Commands::Secretsmanager {} => {
            let mut config_builder = aws_sdk_secretsmanager::config::Builder::from(&aws_sdk_config);
            if cli.endpoint_url.is_some() {
                config_builder = config_builder.endpoint_url(cli.endpoint_url.unwrap());
            }
            let config = config_builder.build();
            Box::new(aws::secretsmanager::SecretsManagerSecretsLoader::new(
                SecretsManagerClient::from_conf(config),
            ))
        }
    };

    // Return the fetched secrets to stdout in the format expected by Vector.
    let fetched_secrets: vector::FetchedSecrets = secrets_loader.load(secrets_to_fetch).await;
    println!("{}", serde_json::to_string(&fetched_secrets).unwrap());
}
