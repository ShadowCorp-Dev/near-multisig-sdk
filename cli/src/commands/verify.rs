use anyhow::Result;
use near_multisig_lib::checksum::SHA256SUMS;
use std::path::Path;

pub fn run(release_dir: &str, reproduce: bool) -> Result<()> {
    let release_path = Path::new(release_dir);

    // Read SHA256SUMS
    let checksums_file = release_path.join("SHA256SUMS");
    if !checksums_file.exists() {
        anyhow::bail!("SHA256SUMS not found in {}", release_dir);
    }

    let checksums = SHA256SUMS::from_file(&checksums_file)?;

    println!("Verifying checksums...");
    let result = checksums.verify(release_path)?;

    for file in &result.verified {
        println!("✓ {} (checksum matches)", file);
    }

    for mismatch in &result.mismatch {
        println!("✗ {} (checksum mismatch)", mismatch.filename);
        println!("  Expected: {}", mismatch.expected);
        println!("  Actual:   {}", mismatch.actual);
    }

    for missing in &result.missing {
        println!("✗ {} (file missing)", missing);
    }

    if !result.is_success() {
        anyhow::bail!("Verification failed");
    }

    if reproduce {
        println!("\nReproducibility testing not yet implemented");
        // TODO: Clone repo, checkout commit, rebuild, compare
    }

    Ok(())
}
