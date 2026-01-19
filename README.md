# WEFT - Version Control for Dummies and Teams

Never lose work to merge hell. WEFT makes git rebase async and unbreakable.

## Install

```bash
# Install in 10 seconds
curl -sSL https://raw.githubusercontent.com/weft-vcs/weft/main/install.sh | bash

# Or download binary directly
curl -sSL https://github.com/weft-vcs/weft/releases/download/v0.1.0/weft-x86_64-unknown-linux-gnu | sudo install -m 755 /dev/stdin /usr/local/bin/weft
```

**Requires**: [jj (Jujutsu)](https://github.com/martinvonz/jj) v0.15 or later

Install jj:
- macOS: `brew install jj`
- Linux: `cargo install jj` or download from releases
- See [jj installation docs](https://github.com/martinvonz/jj#installation) for more options

## Quick Start

```bash
cd my-git-repo

# Initialize weft
weft init

# Save your work (never blocks, no staging)
weft save "checkpoint: added user authentication"

# Sync with main (conflicts become tangled commits you resolve later)
weft sync

# Check status
weft status

# Undo if needed
weft undo
```

## How It Works

WEFT treats version control as a weaving process:

- **Main branch** = The warp (stable, linear foundation)
- **Your work** = The weft (personal thread that weaves in at its own pace)

### Key Concepts

**Save** (`weft save "message"`)
- Atomically snapshots your working tree
- Appends to your personal weft branch
- No staging required
- Safe for AI agents to call every 30 seconds

**Sync** (`weft sync`)
- Rebases your weft onto the latest main
- Conflicts become "tangled commits" - first-class commits you resolve later
- Never blocks - always succeeds

**Tangled Commits**
- When sync encounters conflicts, it creates a tangled commit
- You can `weft save` on top of tangled commits
- Resolve them later with `weft untangle` (v0.2)

**Undo** (`weft undo`)
- Walks back the operation log
- Reverts your last operation silently

## Commands (v0.1)

| Command | Description |
|---------|-------------|
| `weft init` | Initialize weft in a git repo |
| `weft save "msg"` | Save current state to your weft |
| `weft sync` | Sync weft onto main (never blocks) |
| `weft status` | Show weft status and tangled commits |
| `weft undo` | Undo the last operation |

## Commands Coming in v0.2

- `weft untangle` - Interactive conflict resolution
- `weft share` - Push weft to remote namespace
- `weft propose` - Submit for integration
- `weft weave` - Atomically update main

## Why WEFT?

Traditional git workflows:
```
git add . && git commit -m "wip"      # Staging is annoying
git pull --rebase                     # Conflicts block you
git push                              # Race conditions
```

WEFT workflow:
```
weft save "checkpoint"                # No staging, always works
weft sync                             # Never blocks, creates tangled commits
weft share                            # Pushes to your personal branch
```

## For Teams

WEFT is designed for asynchronous collaboration:

1. Each developer has their own `refs/weft/$USER/head` branch
2. Developers save frequently without blocking others
3. `weft share` pushes personal work to a shared namespace (v0.2)
4. Integration (propose/weave) happens when ready

## Building from Source

```bash
git clone https://github.com/weft-vcs/weft.git
cd weft
cargo build --release
cargo install --path .
```

## Contributing

PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT
