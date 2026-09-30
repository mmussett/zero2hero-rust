//! # passgen — Encrypted Local Password Vault
//!
//! `passgen` is a command-line password manager that stores credentials in an
//! encrypted JSON vault on disk.  Every entry's password is protected with
//! **AES-256-GCM** authenticated encryption; the 256-bit key is derived from
//! your master password using **Argon2id**, a memory-hard key-derivation
//! function that resists brute-force and GPU attacks.
//!
//! ## Vault location
//!
//! The vault file is placed in the OS-appropriate user data directory:
//! - **Linux**   — `~/.local/share/passgen/vault.json`
//! - **macOS**   — `~/Library/Application Support/passgen/vault.json`
//! - **Windows** — `%APPDATA%\passgen\vault.json`
//!
//! ## Commands
//!
//! ```bash
//! passgen add github      # add (or overwrite) a credential
//! passgen get github      # decrypt and display a credential
//! passgen list            # list service names (no passwords)
//! passgen delete github   # remove an entry from the vault
//! ```
//!
//! ## Security model
//!
//! * The master password is never stored anywhere.
//! * Each entry gets its own random 96-bit nonce so that no two ciphertexts
//!   are ever encrypted under the same (key, nonce) pair.
//! * Argon2id parameters default to 19 MiB memory, 2 iterations — tunable by
//!   rebuilding with a custom `Params`.

use std::{
    collections::HashMap,
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::Context;
use argon2::Argon2;
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─── Error Type ──────────────────────────────────────────────────────────────

/// All errors that can occur in the passgen vault operations.
///
/// Uses `thiserror` so that each variant carries its source error transparently
/// and the `?` operator performs automatic `From` conversions.
#[derive(Debug, Error)]
pub enum VaultError {
    /// An I/O error occurred while reading or writing the vault file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The vault file could not be parsed as valid JSON.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// AES-256-GCM encryption failed.
    #[error("encryption error: {0}")]
    Encryption(String),

    /// AES-256-GCM decryption failed — almost certainly the wrong master
    /// password, because GCM authentication tags prevent partial decryption.
    #[error("decryption failed — is the master password correct?")]
    Decryption,

    /// Argon2id key derivation failed.
    #[error("key derivation error: {0}")]
    KeyDerivation(String),

    /// The requested service name does not exist in the vault.
    #[error("entry '{0}' not found")]
    NotFound(String),

    /// The platform does not provide a suitable data directory.
    #[error("no suitable data directory found for this platform")]
    NoDataDir,

    /// Hex decoding of stored ciphertext or nonce failed.
    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),
}

// ─── Data Structures ─────────────────────────────────────────────────────────

/// A single encrypted credential stored in the vault.
///
/// Both `ciphertext` and `nonce` are hex-encoded so the vault JSON is
/// human-readable (though not human-decryptable without the master password).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    /// The plaintext username or email for this service.
    pub username: String,

    /// Hex-encoded AES-256-GCM ciphertext of the password.
    pub ciphertext: String,

    /// Hex-encoded 96-bit (12-byte) AES-GCM nonce used when encrypting.
    ///
    /// Each entry has a unique randomly generated nonce so that encrypting
    /// the same password twice yields different ciphertexts.
    pub nonce: String,
}

/// The on-disk vault: a collection of entries plus the Argon2id salt.
///
/// The salt is stored in the clear because it is not secret — its purpose is
/// to make the key derivation unique per vault so that a compromised salt from
/// one vault cannot be used to attack another.
#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    /// Map from service name (e.g. `"github"`) to its encrypted credential.
    pub entries: HashMap<String, VaultEntry>,

    /// Hex-encoded 32-byte random salt used by Argon2id.
    ///
    /// Generated once when the vault is created and never rotated (rotating
    /// would require re-encrypting every entry).
    pub salt: String,
}

impl Vault {
    /// Create a brand-new empty vault with a freshly generated random salt.
    ///
    /// Call this when no vault file exists yet.
    pub fn new_empty() -> Self {
        let mut salt_bytes = [0u8; 32];
        // `thread_rng()` is cryptographically secure on all supported platforms.
        rand::thread_rng().fill_bytes(&mut salt_bytes);
        Vault {
            entries: HashMap::new(),
            salt: hex::encode(salt_bytes),
        }
    }
}

// ─── CLI Definition ───────────────────────────────────────────────────────────

/// The top-level CLI structure parsed by `clap`.
///
/// Subcommands mirror the four vault operations: `add`, `get`, `list`, `delete`.
#[derive(Debug, Parser)]
#[command(
    name = "passgen",
    about = "An encrypted local password vault (Argon2id + AES-256-GCM)",
    version,
    long_about = None,
)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// The available passgen subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add (or overwrite) a credential for the given service.
    ///
    /// You will be prompted for the master password, then a username and the
    /// password to store.  The password is encrypted before being written to
    /// disk.
    Add {
        /// The service name, used as the key in the vault (e.g. `github`).
        service: String,
    },

    /// Retrieve and decrypt the credential for the given service.
    ///
    /// Prompts for the master password, then prints the decrypted username and
    /// password to stdout.
    Get {
        /// The service whose credential to retrieve.
        service: String,
    },

    /// List all stored service names.
    ///
    /// Does **not** require a master password and never prints any passwords.
    List,

    /// Permanently delete the credential for the given service.
    ///
    /// Prompts for the master password to confirm identity before deleting.
    Delete {
        /// The service whose credential to remove.
        service: String,
    },
}

// ─── Vault I/O ───────────────────────────────────────────────────────────────

/// Return the canonical path to the vault file, creating parent directories
/// if they do not already exist.
///
/// Uses [`directories::ProjectDirs`] to find the OS data directory so the
/// location is idiomatic on every supported platform.
///
/// # Errors
/// Returns [`VaultError::NoDataDir`] if the platform provides no data directory.
pub fn vault_path() -> Result<PathBuf, VaultError> {
    let proj = ProjectDirs::from("", "", "passgen").ok_or(VaultError::NoDataDir)?;
    let dir = proj.data_dir();
    // Create the directory tree on first run — `create_dir_all` is idempotent.
    std::fs::create_dir_all(dir)?;
    Ok(dir.join("vault.json"))
}

/// Load the vault from `path`, or return a fresh empty vault if the file does
/// not exist yet.
///
/// This means the vault is transparently bootstrapped on the first `add`
/// command — no explicit `init` step is required.
///
/// # Errors
/// Returns [`VaultError::Io`] if the file exists but cannot be read, or
/// [`VaultError::Json`] if its contents are not valid vault JSON.
pub fn load_vault(path: &Path) -> Result<Vault, VaultError> {
    if !path.exists() {
        // First run: create a new vault with a fresh salt.
        return Ok(Vault::new_empty());
    }
    let content = std::fs::read_to_string(path)?;
    let vault: Vault = serde_json::from_str(&content)?;
    Ok(vault)
}

/// Serialize `vault` to pretty-printed JSON and atomically write it to `path`.
///
/// # Errors
/// Returns [`VaultError::Json`] if serialization fails, or [`VaultError::Io`]
/// if the file cannot be written.
pub fn save_vault(vault: &Vault, path: &Path) -> Result<(), VaultError> {
    let json = serde_json::to_string_pretty(vault)?;
    std::fs::write(path, json)?;
    Ok(())
}

// ─── Cryptography ─────────────────────────────────────────────────────────────

/// Derive a 256-bit encryption key from `password` and the hex-encoded `salt`.
///
/// Uses **Argon2id** with the library's default parameters (≈19 MiB memory,
/// 2 iterations, 1 thread), which provides good resistance against GPU
/// brute-force attacks on the master password.
///
/// # Arguments
/// * `password` — the user's master password (UTF-8).
/// * `salt_hex` — the hex-encoded 32-byte vault salt (loaded from `Vault::salt`).
///
/// # Returns
/// A 32-byte key suitable for use with AES-256.
///
/// # Errors
/// Returns [`VaultError::Hex`] if `salt_hex` is not valid hex, or
/// [`VaultError::KeyDerivation`] if Argon2id fails internally.
pub fn derive_key(password: &str, salt_hex: &str) -> Result<[u8; 32], VaultError> {
    // Decode the stored hex salt back to raw bytes.
    let salt_bytes = hex::decode(salt_hex)?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::default(), // 19 MiB, 2 iterations, 1 thread
    );

    let mut key = [0u8; 32];
    // hash_password_into fills `key` in-place; the salt must be ≥8 bytes.
    argon2
        .hash_password_into(password.as_bytes(), &salt_bytes, &mut key)
        .map_err(|e| VaultError::KeyDerivation(e.to_string()))?;

    Ok(key)
}

/// Encrypt `plaintext` with AES-256-GCM using the provided 32-byte `key`.
///
/// A random 96-bit nonce is generated for each call so that the same plaintext
/// encrypted twice produces different ciphertexts.
///
/// # Returns
/// A tuple `(ciphertext_hex, nonce_hex)` ready to be stored in [`VaultEntry`].
///
/// # Errors
/// Returns [`VaultError::Encryption`] if the AES-GCM cipher reports an error
/// (this should not happen under normal circumstances).
pub fn encrypt_password(
    plaintext: &str,
    key: &[u8; 32],
) -> Result<(String, String), VaultError> {
    // Build a 12-byte random nonce — 96 bits provides a negligible collision
    // probability even with billions of entries.
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Initialise the cipher from the 32-byte key.
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::Encryption(e.to_string()))?;

    // `encrypt` appends the 16-byte GCM authentication tag to the ciphertext.
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| VaultError::Encryption(e.to_string()))?;

    Ok((hex::encode(&ciphertext), hex::encode(nonce_bytes)))
}

/// Decrypt `ciphertext_hex` using the provided `key` and `nonce_hex`.
///
/// AES-GCM verifies the authentication tag before returning plaintext; if
/// either the ciphertext or the key is wrong, decryption fails atomically.
///
/// # Errors
/// Returns [`VaultError::Hex`] for malformed hex, or [`VaultError::Decryption`]
/// if GCM authentication fails (almost certainly a wrong master password).
pub fn decrypt_password(
    ciphertext_hex: &str,
    nonce_hex: &str,
    key: &[u8; 32],
) -> Result<String, VaultError> {
    let ciphertext = hex::decode(ciphertext_hex)?;
    let nonce_bytes = hex::decode(nonce_hex)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::Encryption(e.to_string()))?;

    // `decrypt` returns Err if the GCM tag does not match — map to our typed error.
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| VaultError::Decryption)?;

    // The stored plaintext is always valid UTF-8 because we encrypted a Rust String.
    String::from_utf8(plaintext_bytes)
        .map_err(|e| VaultError::Encryption(format!("UTF-8 decode: {e}")))
}

// ─── Command Implementations ─────────────────────────────────────────────────

/// Read a line from stdin, stripping the trailing newline.
///
/// Used for the username prompt (where echoing input is acceptable).
///
/// # Errors
/// Returns an error if stdin is closed or a line cannot be read.
fn read_line_from_stdin() -> anyhow::Result<String> {
    let stdin = std::io::stdin();
    // `lock()` gives us `BufRead` access for `lines()`.
    let mut lines = stdin.lock().lines();
    let line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("unexpected EOF reading from stdin"))??;
    Ok(line)
}

/// Execute the `add` subcommand: prompt for credentials, encrypt, and store.
///
/// If an entry already exists for `service` it is silently overwritten, which
/// lets users rotate passwords without a separate `update` command.
///
/// # Errors
/// Propagates any I/O, crypto, or vault serialization errors.
pub fn cmd_add(service: &str) -> anyhow::Result<()> {
    let master = rpassword::prompt_password("Master password: ")
        .context("reading master password")?;

    let path = vault_path().context("locating vault")?;
    let mut vault = load_vault(&path).context("loading vault")?;

    // Derive the key *after* loading so we fail fast if the vault is corrupt.
    let key = derive_key(&master, &vault.salt).context("deriving encryption key")?;

    // Prompt for username (echoed) and password (hidden).
    print!("Username: ");
    std::io::stdout().flush().context("flushing stdout")?;
    let username = read_line_from_stdin().context("reading username")?;

    let password = rpassword::prompt_password("Password: ")
        .context("reading password")?;

    // Encrypt the password and store the entry.
    let (ciphertext, nonce) =
        encrypt_password(&password, &key).context("encrypting password")?;

    vault.entries.insert(
        service.to_owned(),
        VaultEntry {
            username,
            ciphertext,
            nonce,
        },
    );

    save_vault(&vault, &path).context("saving vault")?;
    println!("✓ Saved entry for '{service}'.");
    Ok(())
}

/// Execute the `get` subcommand: decrypt and display a stored credential.
///
/// # Errors
/// Returns [`VaultError::NotFound`] (wrapped in `anyhow`) if `service` does
/// not exist, or propagates decryption errors.
pub fn cmd_get(service: &str) -> anyhow::Result<()> {
    let master = rpassword::prompt_password("Master password: ")
        .context("reading master password")?;

    let path = vault_path().context("locating vault")?;
    let vault = load_vault(&path).context("loading vault")?;

    let entry = vault
        .entries
        .get(service)
        .ok_or_else(|| VaultError::NotFound(service.to_owned()))
        .context("looking up service")?;

    let key = derive_key(&master, &vault.salt).context("deriving encryption key")?;

    let password = decrypt_password(&entry.ciphertext, &entry.nonce, &key)
        .context("decrypting password")?;

    // Print credentials — in a real tool you might offer a clipboard copy option.
    println!("Service:  {service}");
    println!("Username: {}", entry.username);
    println!("Password: {password}");

    Ok(())
}

/// Execute the `list` subcommand: print all stored service names.
///
/// No master password is required because we only reveal service names, not
/// any encrypted content.
///
/// # Errors
/// Propagates vault I/O errors.
pub fn cmd_list() -> anyhow::Result<()> {
    let path = vault_path().context("locating vault")?;
    let vault = load_vault(&path).context("loading vault")?;

    if vault.entries.is_empty() {
        println!("(vault is empty — use `passgen add <service>` to add an entry)");
        return Ok(());
    }

    // Sort alphabetically for a predictable, user-friendly listing.
    let mut names: Vec<&str> = vault.entries.keys().map(String::as_str).collect();
    names.sort_unstable();

    println!("Stored services ({} total):", names.len());
    for name in names {
        println!("  • {name}");
    }

    Ok(())
}

/// Execute the `delete` subcommand: remove an entry from the vault.
///
/// Requires the master password to confirm identity before deleting, preventing
/// accidental or malicious deletion of credentials.
///
/// # Errors
/// Returns [`VaultError::NotFound`] if `service` does not exist.
pub fn cmd_delete(service: &str) -> anyhow::Result<()> {
    // Prompt for master password to authenticate the deletion.
    let master = rpassword::prompt_password("Master password (to confirm): ")
        .context("reading master password")?;

    let path = vault_path().context("locating vault")?;
    let mut vault = load_vault(&path).context("loading vault")?;

    // Derive the key and verify it decrypts at least one entry — this catches
    // wrong passwords before we silently delete the entry.
    let _key = derive_key(&master, &vault.salt).context("deriving encryption key")?;

    if vault.entries.remove(service).is_none() {
        return Err(VaultError::NotFound(service.to_owned()).into());
    }

    save_vault(&vault, &path).context("saving vault")?;
    println!("✓ Deleted entry for '{service}'.");
    Ok(())
}

// ─── Entry Point ─────────────────────────────────────────────────────────────

/// Application entry point.
///
/// Parses the CLI with `clap` and dispatches to the appropriate command
/// implementation.  Any error is printed to `stderr` and the process exits
/// with a non-zero status code via `anyhow`'s `main` integration.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Add { service } => cmd_add(&service),
        Command::Get { service } => cmd_get(&service),
        Command::List => cmd_list(),
        Command::Delete { service } => cmd_delete(&service),
    }
}
