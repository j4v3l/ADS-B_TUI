# VS Code Configuration for ADS-B TUI

This directory contains VS Code settings and configurations optimized for Rust development on the ADS-B TUI project.

## Files

- `settings.json` - Workspace-specific settings for Rust development
- `extensions.json` - Recommended extensions for the project
- `launch.json` - Debug configurations for the application
- `tasks.json` - Build and test tasks

## Key Features

### Rust Analyzer
- Automatic clippy checks on save
- Full Cargo feature support
- Proc macro support enabled
- Smart diagnostics

### Editor Settings
- Format on save enabled
- Auto-fix and import organization
- Rust-analyzer as default formatter

### Git Integration
- Smart commit enabled
- Auto-fetch enabled
- No sync confirmation prompts

### Development Tasks
- Build, test, and check commands
- Debug configurations for development and release
- Integrated terminal with zsh

## Git Commit Signing

All commits in this repository are signed using SSH keys for security and verification.

### Global Configuration

The following git configuration is applied globally:

```bash
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
git config --global gpg.ssh.allowedSignersFile ~/.ssh/allowed_signers
```

### For Other Repositories

To enable commit signing in other repositories, run:

```bash
# Copy the signing configuration
cp ~/.gitconfig ~/.gitconfig.backup  # Backup first
git config --global commit.gpgsign true
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global gpg.ssh.allowedSignersFile ~/.ssh/allowed_signers
```

### Verifying Signatures

To verify commit signatures locally:

```bash
git log --show-signature
```

On GitHub, signed commits will show a "Verified" badge.

## Recommended Extensions

Install the recommended extensions for the best development experience:

- Rust Analyzer
- Even Better TOML
- GitHub Copilot
- Prettier
- YAML Support

## Getting Started

1. Open the workspace in VS Code
2. Install recommended extensions when prompted
3. Use Ctrl+Shift+P → "Tasks: Run Task" to access build commands
4. Use F5 to start debugging
