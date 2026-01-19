# Gemini Interaction Guidelines

- **Never merge Pull Requests.** Always ask the user to merge manually or confirm explicitly before taking any action that modifies the upstream repository state via merge.
- **Verify CI Locally.** Before pushing changes, ALWAYS verify that the build, linter, tests, and formatter pass locally.
  - Rust: `cargo check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo fmt --all -- --check`.
  - Frontend: `npm run lint`, `npm run build`, `npm test`.
- **Minimal Comments.** Only add comments if they explain *why* a complex piece of logic exists. Do not add comments describing *what* the code is doing (e.g., avoid `// Start server` or `// Define routes`).
