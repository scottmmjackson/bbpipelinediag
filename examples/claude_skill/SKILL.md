---
name: bbpipelinediag-claude
description: This skill allows Claude to identify and diagnose failed Bitbucket Pipelines using the bbpipelinediag tool.
---

# Claude Skill: Bitbucket Pipeline Diagnoser

This skill allows Claude to identify and diagnose failed Bitbucket Pipelines using the `bbpipelinediag` tool.

## Capabilities
- Fetch the latest failed pipeline for a repository.
- Retrieve logs for failed pipeline steps.
- Apply filters (grep, head, tail) to logs to isolate errors.
- Provide automated diagnosis using LLM integration or local scripts.

## Usage
To use this skill, provide the repository workspace and slug. Claude will then use `bbpipelinediag` to find the failure and analyze it.

## Example Command
- `bbpipelinediag diagnose --workspace my-ws --repo my-repo --grep "panic" --before 10`

## Configuration
Ensure that `bbpipelinediag` is installed and the `BITBUCKET_USERNAME` and `BITBUCKET_APP_PASSWORD` environment variables are set.
