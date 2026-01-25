# AI Agent Interaction Guidelines

- **Never merge Pull Requests.** Always ask the user to merge manually or confirm explicitly before taking any action that modifies the upstream repository state via merge.
- **Verify CI Locally.** Before pushing changes, ALWAYS verify that the build, linter, tests, and formatter pass locally.
  - Rust: `cargo check`, `cargo clippy -- -D warnings`, `cargo test`. ALWAYS run `cargo fmt` to ensure code is formatted correctly.
  - Frontend: `npm run lint`, `npm run build`, `npm test`. ALWAYS run `npm run format` to ensure code is formatted correctly.
- **Minimal Comments.** Only add comments if they explain *why* a complex piece of logic exists. Do not add comments describing *what* the code is doing (e.g., avoid `// Start server` or `// Define routes`).
- **Configuration.** The backend uses the 12-factor app methodology. Configuration is loaded from environment variables using `AppConfig`. Do not hardcode constants for configuration.
- **PR Naming.** Use [Conventional Commits](https://www.conventionalcommits.org/) for PR titles (e.g., `feat: ...`, `fix: ...`). In the PR body, include a brief description and explicitly close linked issues using `Closes #123`. Do not include "Implements issue #..." in the title.
- **Branch Naming.** Use descriptive kebab-case names that reveal the purpose of the work (e.g., `feat/visual-indicator`, `fix/mobile-responsiveness`, `docs/update-guidelines`). Do NOT use generic names like `implement-issue-46`.
