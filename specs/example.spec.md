# Example spec: simple Copilot CLI test

Description

Run a simple Copilot CLI command and verify the output.

Command

```bash
copilot --version
```

Expected outcome

- The command exits with status code 0.
- Stdout contains a semantic version string (e.g. "1.2.3").
