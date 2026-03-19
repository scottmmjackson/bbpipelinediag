## Bitbucket Pipeline Diagnoser (bbpipelinediag)

A Rust CLI tool that fetches logs from failing Bitbucket Pipeline steps, applies filters to isolate relevant errors, and passes the context to a diagnoser (local command or LLM API).

### Requirements

- **Ergonomics & Configuration**:
    - Employs a clap-based CLI with clear subcommands and flags.
    - Configuration is stored via the `confy` crate in a standard, cross-platform location (application name: `bbpipelinediag`).
    - Supports authentication via environment variables: `BITBUCKET_USERNAME`, `BITBUCKET_APP_PASSWORD`, and `BITBUCKET_TOKEN`.
- **Data Retrieval**:
    - Fetches pipeline execution details and identifies failed steps.
    - Retrieves raw logs for the identified failing steps.
- **Log Filtering**:
    - `head`: Truncates output after the first *n* lines.
    - `tail`: Truncates output before the last *n* lines.
    - `grep`: Identifies the first, last, or *j-th* match of a regular expression and returns the match with *n* lines of leading context and *k* lines of trailing context.
- **Diagnosis**:
    - **Local Command**: Pipes the filtered logs into a specified shell command via `stdin`.
    - **LLM API**: Sends the filtered logs as context to an OpenAI-compatible API (requires configuration for `base_url`, `model`, and `api_key`).
- **Output**:
    - Provides a structured summary of the failure and the resulting diagnosis.
    - Supports saving the diagnostic report to a Markdown file.

### Implementation Lessons & Maintenance Notes

- **Log Volume**: Bitbucket Pipeline logs can be extremely large. Filters must be applied efficiently (ideally streaming) to avoid excessive memory usage.
- **Step Identification**: Pipelines can have parallel steps or multiple stages. The tool should prioritize the most recent failure or allow the user to specify a step name/ID.
- **API Compatibility**: When using the LLM diagnoser, ensure the prompt clearly distinguishes between the "system" instructions and the "user" provided log context.
- **Filtering Logic**: The `grep` filter is the most complex. It should support standard regex syntax and handle overlapping context windows gracefully.

### Coding Standards

- Include docstrings on all functions, traits, and structs.
- Use `anyhow` for error handling and `tokio` for async operations.
- Always test new code for correctness and robustness, especially the log filtering logic.
- Update this document with newly-adopted standards or implementation lessons as the tool evolves.
- Bump the version in `Cargo.toml` when adding new features.
