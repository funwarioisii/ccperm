# ccperm

Claude Code permissions analyzer for ghq repositories.

## Usage

```bash
# Show all allowed permissions
ccperm

# Show denied permissions  
ccperm --deny

# JSON output
ccperm --json
ccperm --deny --json
```

## Options

```
  -g, --ghq-root <GHQ_ROOT>  Path to ghq root
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
