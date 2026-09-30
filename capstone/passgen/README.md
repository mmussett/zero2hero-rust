# passgen

An encrypted local password vault that stores credentials using Argon2id key derivation and AES-256-GCM encryption.

## Build

```bash
cargo build -p passgen --release
```

## Usage

```bash
# Add a new entry
passgen add github

# Retrieve an entry (decrypts and prints username + password)
passgen get github

# List all stored service names (no passwords shown)
passgen list

# Delete an entry
passgen delete github
```

The vault is stored at the OS-appropriate data directory (e.g. `~/.local/share/passgen/vault.json` on Linux, `%APPDATA%\passgen\vault.json` on Windows). You will be prompted for your master password on every operation.
