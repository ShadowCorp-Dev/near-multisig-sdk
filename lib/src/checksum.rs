use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SHA256SUMS {
    entries: Vec<ChecksumEntry>,
}

pub struct ChecksumEntry {
    pub hash: String,
    pub filename: String,
    pub binary_mode: bool,
}

impl SHA256SUMS {
    pub fn from_directory(dir: &Path) -> Result<Self> {
        let mut entries = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                let hash = Self::hash_file(&path)?;
                entries.push(ChecksumEntry {
                    hash,
                    filename: path.file_name().unwrap().to_string_lossy().to_string(),
                    binary_mode: true,
                });
            }
        }

        Ok(Self { entries })
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                anyhow::bail!("Invalid SHA256SUMS format: {}", line);
            }

            let hash = parts[0].to_string();
            let filename_part = parts[1];

            let (binary_mode, filename) = if let Some(name) = filename_part.strip_prefix('*') {
                (true, name.to_string())
            } else {
                (false, filename_part.to_string())
            };

            entries.push(ChecksumEntry {
                hash,
                filename,
                binary_mode,
            });
        }

        Ok(Self { entries })
    }

    pub fn write_to_file(&self, path: &PathBuf) -> Result<()> {
        let mut content = String::new();

        for entry in &self.entries {
            let mode_indicator = if entry.binary_mode { "*" } else { " " };
            content.push_str(&format!(
                "{} {}{}\n",
                entry.hash, mode_indicator, entry.filename
            ));
        }

        fs::write(path, content)?;
        Ok(())
    }

    pub fn verify(&self, dir: &Path) -> Result<VerificationResult> {
        let mut results = VerificationResult::default();

        for entry in &self.entries {
            let file_path = dir.join(&entry.filename);

            if !file_path.exists() {
                results.missing.push(entry.filename.clone());
                continue;
            }

            let actual_hash = Self::hash_file(&file_path)?;

            if actual_hash == entry.hash {
                results.verified.push(entry.filename.clone());
            } else {
                results.mismatch.push(ChecksumMismatch {
                    filename: entry.filename.clone(),
                    expected: entry.hash.clone(),
                    actual: actual_hash,
                });
            }
        }

        Ok(results)
    }

    fn hash_file(path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};
        let bytes = fs::read(path)?;
        let hash = Sha256::digest(&bytes);
        Ok(format!("{:x}", hash))
    }
}

#[derive(Default)]
pub struct VerificationResult {
    pub verified: Vec<String>,
    pub mismatch: Vec<ChecksumMismatch>,
    pub missing: Vec<String>,
}

pub struct ChecksumMismatch {
    pub filename: String,
    pub expected: String,
    pub actual: String,
}

impl VerificationResult {
    pub fn is_success(&self) -> bool {
        self.mismatch.is_empty() && self.missing.is_empty()
    }
}
