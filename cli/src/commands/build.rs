use anyhow::{Context, Result};
use near_multisig_lib::{build_manifest::BuildManifest, checksum::SHA256SUMS};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(release_dir: &str) -> Result<()> {
    println!("Building WASM...");

    // Run cargo near build (non-reproducible for now, fast local builds)
    let status = Command::new("cargo")
        .args(["near", "build", "non-reproducible-wasm"])
        .status()
        .context("Failed to run cargo near build")?;

    if !status.success() {
        anyhow::bail!("Build failed");
    }

    // Find built WASM
    let wasm_path = PathBuf::from("target/near")
        .read_dir()?
        .find_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "wasm" {
                Some(path)
            } else {
                None
            }
        })
        .context("No WASM file found in target/near/")?;

    println!("✓ Built: {}", wasm_path.display());

    // Create release directory
    let release_path = Path::new(release_dir);
    fs::create_dir_all(release_path)?;

    // Copy WASM to release/
    let wasm_filename = wasm_path.file_name().unwrap();
    let release_wasm = release_path.join(wasm_filename);
    fs::copy(&wasm_path, &release_wasm)?;

    // Generate SHA256SUMS
    let checksums = SHA256SUMS::from_directory(release_path)?;
    checksums.write_to_file(&release_path.join("SHA256SUMS"))?;
    println!("✓ Generated: {}/SHA256SUMS", release_dir);

    // Generate build manifest
    let manifest = BuildManifest::generate(&release_wasm)?;
    manifest.write_to_file(&release_path.join("build-manifest.json"))?;
    println!("✓ Generated: {}/build-manifest.json", release_dir);

    println!("✓ Artifacts ready in: {}/", release_dir);

    Ok(())
}
