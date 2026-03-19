# bbpipelinediag (Bitbucket Pipeline Diagnoser)

`bbpipelinediag` is a Rust-based CLI tool designed to fetch logs from failing Bitbucket Pipeline steps, apply filters to isolate relevant errors, and provide diagnosis through local execution or LLM integration.

## Features

- **Automatic Failure Detection**: Finds the latest failed pipeline and steps in a repository.
- **Log Filtering**: Efficiently filter large logs using `head`, `tail`, and `grep` with context (`--before`, `--after`).
- **Diagnosis Tools**:
    - **Local Command**: Pipe filtered logs into a local shell command (e.g., `grep`, `sed`, or custom scripts).
    - **LLM-Powered Diagnosis**: Send filtered logs directly to an OpenAI-compatible API for root cause analysis.
- **Configuration**: Standardized configuration via `confy`.
- **Environment Support**: Load credentials from a `.env` file or environment variables.

## Quick Start

### 1. Configure Defaults
Edit the configuration file to set your workspace, repo, filters, and diagnosis method. You can use `bbpipelinediag configure` to set simple defaults:

```bash
bbpipelinediag configure --workspace my-workspace --repo my-repo
```

For advanced configuration (filters and diagnosis), manually edit the config file:
- **macOS**: `~/Library/Application Support/bbpipelinediag/config.toml`
- **Linux**: `~/.config/bbpipelinediag/config.toml`

### 2. Set up Credentials
Create a `.env` file or export environment variables:
```bash
export BITBUCKET_USERNAME="your_username"
export BITBUCKET_APP_PASSWORD="your_app_password"
export LLM_API_KEY="sk-..." # Optional, for LLM diagnosis
```

### 3. Usage
Diagnose the latest failed pipeline using settings from your config:

```bash
bbpipelinediag diagnose
```

## Configuration

`bbpipelinediag` is now configuration-driven. Filters and diagnosis methods are defined in the TOML file.

```toml
workspace = "my-workspace"
repo = "my-repo"

[[filters]]
type = "grep"
pattern = "ERROR|panic"
before = 10
after = 2

[http_range]
start = -10240 # Request last 10 KiB of logs

[diagnosis]
type = "llm"
```

## Command Line Arguments

`bbpipelinediag` supports simplified CLI arguments:

-   `configure`: Set default workspace and repo.
-   `diagnose`: Fetch and analyze logs using configured filters and diagnosis.
    -   `-w, --workspace <WS>`: Override workspace.
    -   `-r, --repo <REPO>`: Override repository slug.
    -   `-p, --pipeline-id <ID>`: Specify a pipeline UUID (defaults to latest).

## Development

Managed via `just`:
-   `just build`: Build the release binary.
-   `just test`: Run the test suite.
-   `just check`: Run linting and formatting checks.

## Examples

Check the [examples/](./examples) directory for:
-   `config.toml`: A sample configuration file.
-   `claude_skill/SKILL.md`: A skill definition for Claude.
-   `gemini_skill/SKILL.md`: A native Gemini CLI skill definition.
