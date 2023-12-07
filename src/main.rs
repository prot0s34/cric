use clap::{App, Arg};
use colored::*;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::Client;
use serde_json::Value;
use std::error::Error;

const DOCKER_REGISTRY: &str = "registry-1.docker.io";
const GITHUB_REGISTRY: &str = "ghcr.io";
const GCR_REGISTRY: &str = "gcr.io";
const K8S_REGISTRY: &str = "registry.k8s.io";
const QUAY_REGISTRY: &str = "quay.io";
const ZALANDO_REGISTRY: &str = "registry.opensource.zalan.do";

// public.ecr.aws
// registry.gitlab.com
// nvcr.io

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Container Registry Image Checker")
        .version("0.1.0")
        .about("Checks the availability of a container image in public registries")
        .arg(
            Arg::with_name("IMAGE")
                .help("The container image to check, e.g., 'library/nginx:latest'")
                .required(true)
                .index(1),
        )
        .get_matches();

    let image = matches.value_of("IMAGE").unwrap();
    let (repo, tag) = parse_image_name(image);

    let client = Client::new();

    check_image_availability(&client, DOCKER_REGISTRY, repo, tag, "https://").await?;
    check_image_availability(&client, GITHUB_REGISTRY, repo, tag, "https://").await?;
    check_image_availability(&client, GCR_REGISTRY, repo, tag, "https://").await?;
    check_image_availability(&client, K8S_REGISTRY, repo, tag, "https://").await?;
    check_image_availability(&client, QUAY_REGISTRY, repo, tag, "https://").await?;
    check_image_availability(&client, ZALANDO_REGISTRY, repo, tag, "https://").await?;
    
    Ok(())
}

async fn check_image_availability(
    client: &Client,
    registry: &str,
    repo: &str,
    tag: &str,
    protocol: &str,
) -> Result<(), Box<dyn Error>> {
    let manifest_format = match registry {
        GCR_REGISTRY => "application/vnd.oci.image.index.v1+json",
        _ => "application/vnd.docker.distribution.manifest.v2+json",
    };

    let url = format!(
        "{protocol}{registry}/v2/{repo}/manifests/{tag}",
        protocol = protocol,
        registry = registry,
        repo = repo,
        tag = tag
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(manifest_format),
    );

    let token_result = match registry {
        DOCKER_REGISTRY => get_docker_token(client, repo).await,
        GITHUB_REGISTRY => get_github_token(client, repo).await,
        GCR_REGISTRY => get_google_token(client, repo).await,
        _ => Ok(String::new()),
    };

    if let Ok(token) = token_result {
        if !token.is_empty() {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token))?,
            );
        }

        let res = client.get(&url).headers(headers).send().await?;
        match res.status().is_success() {
            true => println!(
                "✓ Image {}:{} is available in {}.",
                repo,
                tag,
                registry.green()
            ),
            false => println!(
                "✗ Image {}:{} is not available in {}.",
                repo,
                tag,
                registry.red()
            ),
        }
    } else {
        println!(
            "✗ Image {}:{} is not available in {}.",
            repo,
            tag,
            registry.red()
        );
    }

    Ok(())
}

async fn get_docker_token(client: &Client, repo: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://auth.docker.io/token?service=registry.docker.io&scope=repository:{repo}:pull",
        repo = repo
    );
    let res = client.get(&url).send().await?;
    let body = res.text().await?;
    let v: Value = serde_json::from_str(&body)?;

    Ok(v["token"].as_str().unwrap().to_string())
}

async fn get_github_token(client: &Client, repo: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://ghcr.io/token?scope=repository:{repo}:pull",
        repo = repo
    );
    let res = client.get(&url).send().await?;
    let body = res.text().await?;
    let v: Value = serde_json::from_str(&body)?;

    match v["token"].as_str() {
        Some(token) => Ok(token.to_string()),
        None => Err("Token not found".into()),
    }
}

async fn get_google_token(client: &Client, repo: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://gcr.io/v2/token?scope=repository:{repo}:pull&service=gcr.io",
        repo = repo
    );
    let res = client.get(&url).send().await?;
    let body = res.text().await?;
    let v: Value = serde_json::from_str(&body)?;
    
    match v["token"].as_str() {
        Some(token) => Ok(token.to_string()),
        None => Err("Token not found".into()),
    }
} 

fn parse_image_name(image: &str) -> (&str, &str) {
    let parts: Vec<&str> = image.split(':').collect();
    let repo = parts[0];
    let tag = if parts.len() > 1 { parts[1] } else { "latest" };

    (repo, tag)
}
