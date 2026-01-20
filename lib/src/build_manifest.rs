use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildManifest {
    pub version: String,
    pub source: SourceInfo,
    pub build: BuildInfo,
    pub output: OutputInfo,
    pub metadata: MetadataInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceInfo {
    pub repository: String,
    pub commit: String,
    pub tag: Option<String>,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildInfo {
    pub builder_image: String,
    pub builder_image_digest: String,
    pub command: Vec<String>,
    pub timestamp: String,
    pub toolchain: ToolchainInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolchainInfo {
    pub rust_version: String,
    pub near_sdk_version: String,
    pub cargo_near_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputInfo {
    pub wasm_hash: String, // "sha256:abc123..."
    pub wasm_size: u64,
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataInfo {
    pub reproducible: bool,
    pub standards: Vec<Standard>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Standard {
    pub name: String,
    pub version: String,
}

impl BuildManifest {
    pub fn generate(wasm_path: &PathBuf) -> Result<Self> {
        let wasm_bytes = fs::read(wasm_path)?;
        let wasm_hash = format!("sha256:{}", hex::encode(sha256(&wasm_bytes)));
        let wasm_size = wasm_bytes.len() as u64;

        // Try to get git info, fallback to placeholders
        let (repository, commit, tag) = get_git_info();

        Ok(Self {
            version: "1.0.0".to_string(),
            source: SourceInfo {
                repository,
                commit,
                tag,
                path: ".".to_string(),
            },
            build: BuildInfo {
                builder_image: "sourcescan/cargo-near:0.18.0-rust-1.86.0".to_string(),
                builder_image_digest:
                    "sha256:2d0d458d2357277df669eac6fa23a1ac922e5ed16646e1d3315336e4dff18043"
                        .to_string(),
                command: vec![
                    "cargo".to_string(),
                    "near".to_string(),
                    "build".to_string(),
                    "--locked".to_string(),
                ],
                timestamp: chrono::Utc::now().to_rfc3339(),
                toolchain: ToolchainInfo {
                    rust_version: "1.86.0".to_string(),
                    near_sdk_version: get_near_sdk_version()
                        .unwrap_or_else(|_| "5.24.0".to_string()),
                    cargo_near_version: "0.18.0".to_string(),
                },
            },
            output: OutputInfo {
                wasm_hash,
                wasm_size,
                filename: wasm_path.file_name().unwrap().to_string_lossy().to_string(),
            },
            metadata: MetadataInfo {
                reproducible: true,
                standards: vec![
                    Standard {
                        name: "nep330".to_string(),
                        version: "1.3.0".to_string(),
                    },
                    Standard {
                        name: "slsa".to_string(),
                        version: "1.0".to_string(),
                    },
                ],
            },
        })
    }

    pub fn write_to_file(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

fn sha256(data: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn get_git_info() -> (String, String, Option<String>) {
    use std::process::Command;

    // Get repository URL
    let repository = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get current commit hash
    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get current tag if any
    let tag = Command::new("git")
        .args(["describe", "--tags", "--exact-match"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    (repository, commit, tag)
}

fn get_near_sdk_version() -> Result<String> {
    // Try to read from Cargo.toml
    let cargo_toml = fs::read_to_string("Cargo.toml")?;

    for line in cargo_toml.lines() {
        if line.contains("near-sdk") && line.contains("=") {
            // Extract version from line like: near-sdk = "5.24.0"
            if let Some(version_start) = line.find('"') {
                if let Some(version_end) = line[version_start + 1..].find('"') {
                    return Ok(line[version_start + 1..version_start + 1 + version_end].to_string());
                }
            }
        }
    }

    anyhow::bail!("Could not find near-sdk version in Cargo.toml")
}
