use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use config::args::BackupMode;
use server_common::backup_encryption::decrypt;
use sha2::{Digest, Sha256};

struct Sha256Finder {
    stack: Vec<std::fs::ReadDir>,
}

impl Sha256Finder {
    fn new(root: &Path) -> Result<Self, String> {
        let iter = std::fs::read_dir(root)
            .map_err(|e| format!("Failed to read {}: {e}", root.display()))?;
        Ok(Self { stack: vec![iter] })
    }

    fn next_sha256_file(&mut self) -> Result<Option<PathBuf>, String> {
        while let Some(iter) = self.stack.last_mut() {
            match iter.next() {
                Some(Ok(entry)) => {
                    let path = entry.path();
                    if path.is_dir() {
                        let sub_iter = std::fs::read_dir(&path)
                            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
                        self.stack.push(sub_iter);
                    } else if path.extension().and_then(|s| s.to_str()) == Some("sha256") {
                        return Ok(Some(path));
                    }
                }
                Some(Err(e)) => return Err(e.to_string()),
                None => {
                    self.stack.pop();
                }
            }
        }
        Ok(None)
    }
}

fn read_sha256_file(path: &Path) -> Result<Vec<(String, String)>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let mut entries = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: "hash *filename" (binary mode) or "hash  filename" (text mode)
        let (hash, filename) = if let Some(pos) = line.find(" *") {
            let (h, f) = line.split_at(pos);
            (h.to_string(), f[2..].to_string())
        } else if let Some(pos) = line.find("  ") {
            let (h, f) = line.split_at(pos);
            (h.to_string(), f[2..].to_string())
        } else {
            return Err(format!(
                "{}:{}: invalid sha256sum line format",
                path.display(),
                i + 1
            ));
        };
        entries.push((hash, filename));
    }
    Ok(entries)
}

fn verify_file_checksum(data_path: &Path, expected_hash: &str) -> Result<(), String> {
    let mut file = std::fs::File::open(data_path)
        .map_err(|e| format!("Failed to open {}: {e}", data_path.display()))?;
    use std::io::Read;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("Failed to read {}: {e}", data_path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let actual = base16ct::lower::encode_string(&hasher.finalize());
    if actual == expected_hash {
        Ok(())
    } else {
        Err(format!(
            "Checksum mismatch for {}: expected {expected_hash}, got {actual}",
            data_path.display()
        ))
    }
}

fn verify_backup_dir(dir: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    let mut errors = 0u64;

    for subdir in ["files", "content"] {
        let root = dir.join(subdir);
        if !root.exists() {
            continue;
        }
        let mut finder = Sha256Finder::new(&root)?;
        while let Some(sha_path) = finder.next_sha256_file()? {
            let entries = read_sha256_file(&sha_path)?;
            for (hash, filename) in &entries {
                let data_path = sha_path.parent().unwrap().join(filename);
                if !data_path.exists() {
                    errors += 1;
                    eprintln!(
                        "ERROR: data file for checksum not found: {:?} -> {:?}",
                        sha_path, data_path
                    );
                    continue;
                }
                total += 1;
                if let Err(e) = verify_file_checksum(&data_path, hash) {
                    errors += 1;
                    eprintln!("ERROR: {e}");
                }
            }
        }
    }

    println!("Verified {total} files, {errors} errors");
    if errors > 0 {
        Err(format!("{errors} checksum verification(s) failed"))
    } else {
        Ok(total)
    }
}

/// Find all `.sha256` files under `root`, read the paired data file,
/// decrypt it with the given key, and write the plaintext alongside
/// the encrypted file with a `.decrypted` suffix.
fn decrypt_backup_subdir(
    root: &Path,
    key: &[u8; 16],
    decrypted: &mut u64,
    errors: &mut u64,
) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }

    let mut finder = Sha256Finder::new(root)?;
    while let Some(sha_path) = finder.next_sha256_file()? {
        let entries = read_sha256_file(&sha_path)?;
        for (_hash, filename) in &entries {
            let data_path = sha_path.parent().unwrap().join(filename);
            if !data_path.exists() {
                *errors += 1;
                eprintln!("ERROR: data file not found: {data_path:?}");
                continue;
            }

            let out_path = data_path.with_extension("decrypted");
            let mut out_file = match std::fs::File::create(&out_path) {
                Ok(f) => f,
                Err(e) => {
                    *errors += 1;
                    eprintln!("ERROR: failed to create {}: {e}", out_path.display());
                    continue;
                }
            };

            let mut stream = decrypt::DecryptBackupDataStream::open(&data_path, *key)
                .map_err(|e| format!("Failed to open {}: {e}", data_path.display()))?;

            loop {
                match stream.decrypt_next_chunk() {
                    Ok(Some(chunk)) => {
                        if let Err(e) = std::io::Write::write_all(&mut out_file, &chunk) {
                            *errors += 1;
                            eprintln!("ERROR: IO error writing {}: {e}", out_path.display());
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(decrypt::DecryptError::Io(e)) => {
                        *errors += 1;
                        eprintln!("ERROR: IO error decrypting {}: {e}", data_path.display());
                        break;
                    }
                    Err(decrypt::DecryptError::DecryptionFailed)
                    | Err(decrypt::DecryptError::NotEnoughData) => {
                        *errors += 1;
                        eprintln!("ERROR: decryption failed for {}", data_path.display());
                        break;
                    }
                }
            }
            *decrypted += 1;
            println!(
                "Decrypted: {} -> {}",
                data_path.display(),
                out_path.display()
            );
        }
    }

    Ok(())
}

fn decrypt_backup_dir(dir: &Path, key: &[u8; 16]) -> Result<u64, String> {
    let mut decrypted = 0u64;
    let mut errors = 0u64;

    for subdir in ["files", "content"] {
        if let Err(e) = decrypt_backup_subdir(&dir.join(subdir), key, &mut decrypted, &mut errors) {
            eprintln!("ERROR: {subdir}: {e}");
            errors += 1;
        }
    }

    println!("Decrypted {decrypted} files, {errors} errors");
    if errors > 0 {
        Err(format!("{errors} decryption(s) failed"))
    } else {
        Ok(decrypted)
    }
}

fn read_key_file(path: &Path) -> Result<[u8; 16], String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read key file {}: {e}", path.display()))?;
    let trimmed = content.trim();
    let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, trimmed)
        .map_err(|e| format!("Failed to base64-decode key file {}: {e}", path.display()))?;
    if decoded.len() != 16 {
        return Err(format!(
            "Decoded key must be exactly 16 bytes (AES-128 key), got {} bytes",
            decoded.len()
        ));
    }
    let mut key = [0u8; 16];
    key.copy_from_slice(&decoded);
    Ok(key)
}

pub fn handle_backup_mode(mode: BackupMode) -> ExitCode {
    match mode {
        BackupMode::Verify { dir } => {
            if !dir.exists() {
                eprintln!("Directory not found: {}", dir.display());
                return ExitCode::FAILURE;
            }
            match verify_backup_dir(&dir) {
                Ok(_) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::FAILURE
                }
            }
        }
        BackupMode::Decrypt { dir, key_file } => {
            if !dir.exists() {
                eprintln!("Directory not found: {}", dir.display());
                return ExitCode::FAILURE;
            }
            let key = match read_key_file(&key_file) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            match decrypt_backup_dir(&dir, &key) {
                Ok(_) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
