# ccperm

Claude Code permissions analyzer for ghq repositories.

## What it does

`ccperm` scans all your ghq repositories and collects Claude Code permissions from `.claude/settings.local.json` files.

## Usage

```bash
# Show all allowed permissions
$ ccperm
Bash(./target/debug/ccperm:*)
Bash(bun add:*)
Bash(cargo clippy:*)
Bash(npm run test:*)
WebFetch(domain:docs.anthropic.com)
...

# Show denied permissions
$ ccperm --deny
(no output if no deny permissions exist)

# JSON output
$ ccperm --json
{
  "permissions": {
    "allow": [
      "Bash(cargo clippy:*)",
      "Bash(npm run test:*)",
      "WebFetch(domain:docs.anthropic.com)"
    ]
  }
}
```

## Options

```
  -g, --ghq-root <GHQ_ROOT>  Path to ghq root directory (defaults to `ghq root`)
      --json                 Output as JSON format
      --deny                 Show deny permissions instead of allow
  -h, --help                 Print help
```

## Installation

```bash
cargo install --git https://github.com/funwarioisii/ccperm
```

## Merge with global settings

```bash
jq -s '.[0].permissions.allow = (.[0].permissions.allow + .[1].permissions.allow | unique | sort) | .[0]' \
  ~/.claude/settings.json <(ccperm --json) | tee ~/.claude/settings.json >/dev/null
```
