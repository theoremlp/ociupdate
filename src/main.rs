use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_ecr::types::{builders::ListImagesFilterBuilder, TagStatus};
use clap::{Parser, Subcommand};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};
use version::GitDescribeVersion;

mod repo;
mod version;

#[derive(Parser)]
struct Cli {
    /// Path to a Theorem OCI lockfile (defaults to './oci.lock.json')
    #[clap(long)]
    lockfile: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Updates ECR images in the lockfile to the latest version
    Update(UpdateArgs),
}

#[derive(clap::Args)]
struct UpdateArgs {
    #[arg(long, help = "Allow resolving to snapshot releases")]
    allow_snapshots: bool,
}

#[derive(Serialize, Deserialize)]
struct Lockfile {
    v1: LockfileV1,
}

#[derive(Serialize, Deserialize)]
struct LockfileV1 {
    images: BTreeMap<String, ImageDefinition>,
}

#[derive(Serialize, Deserialize)]
struct ImageDefinition {
    image: String,
    tag: String,
    digest: String,
    platforms: Vec<String>,
}

async fn latest_image(
    client: &aws_sdk_ecr::Client,
    image: &str,
    allow_snapshots: bool,
) -> Option<(GitDescribeVersion, String)> {
    let (registry_id, repository_name) = repo::extract(image)?;

    let items: Result<Vec<_>, _> = client
        .list_images()
        .registry_id(registry_id)
        .repository_name(repository_name)
        .filter(
            ListImagesFilterBuilder::default()
                .tag_status(TagStatus::Tagged)
                .build(),
        )
        .into_paginator()
        .items()
        .send()
        .collect()
        .await;

    items
        .expect("Unable to retrieve image list")
        .into_iter()
        .filter_map(|id| {
            id.image_tag()
                .and_then(|tag| GitDescribeVersion::from_str(tag).ok())
                .map(|v| (v, id.image_digest().unwrap().to_owned()))
        })
        .filter(|(version, _)| allow_snapshots || version.is_release())
        .max_by(|(a, _), (b, _)| a.cmp(b))
}

async fn update_lockfile(
    client: aws_sdk_ecr::Client,
    lockfile: &std::path::Path,
    allow_snapshots: bool,
) {
    let contents = fs::read_to_string(lockfile).expect("Unable to load lockfile");

    let deser: Lockfile = serde_json::from_str(&contents).expect("Unable to deserialize lockfile");

    let images = deser.v1.images;

    let image_futures = images.into_iter().map(|(name, definition)| async {
        let maybe_latest_image = latest_image(&client, &definition.image, allow_snapshots).await;
        if maybe_latest_image.is_none() {
            return (name, definition);
        }

        let (latest_tag, latest_digest) = maybe_latest_image.unwrap();
        if GitDescribeVersion::from_str(&definition.tag).unwrap() < latest_tag {
            println!("Updating {name} to {latest_tag} ({latest_digest})");
            (
                name,
                ImageDefinition {
                    image: definition.image,
                    platforms: definition.platforms,
                    tag: latest_tag.to_string(),
                    digest: latest_digest,
                },
            )
        } else {
            (name, definition)
        }
    });

    let images = join_all(image_futures).await.into_iter().collect();

    let updated = Lockfile {
        v1: LockfileV1 { images },
    };

    let contents = serde_json::to_string_pretty(&updated).unwrap();
    fs::write(lockfile, contents + "\n").expect("Error updating lockfile")
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let lockfile = cli
        .lockfile
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new("./oci.lock.json"));

    if !lockfile.exists() {
        panic!("Cannot find lockfile '{:?}'", lockfile);
    }

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = aws_sdk_ecr::Client::new(&config);

    match &cli.command {
        Commands::Update(args) => update_lockfile(client, lockfile, args.allow_snapshots).await,
    }
}
