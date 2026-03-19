---
name: bbpipelinediag
description: Use this skill to diagnose failed Bitbucket Pipelines by fetching and filtering logs and identifying root causes.
---

# Gemini Skill: Bitbucket Pipeline Diagnoser

This skill enables Gemini to process failed Bitbucket Pipelines and diagnose root causes using the `bbpipelinediag` tool.

## Role
Senior DevOps engineer specializing in troubleshooting build failures.

## Instructions
1. **Prepare Configuration**: Ensure `config.toml` has appropriate filters (e.g., `grep` for "ERROR") and diagnosis set to `llm`.
2. **Execute Diagnosis**: Run `bbpipelinediag diagnose` for the target workspace/repo.
3. **Analyze Results**: Review the filtered logs and the LLM-provided diagnosis to understand the failure.

## Tools Required
- `bbpipelinediag` CLI tool.
- Bitbucket credentials and LLM API key (via `.env` or environment variables).

## Example Command
`bbpipelinediag diagnose --workspace my-ws --repo my-repo`
