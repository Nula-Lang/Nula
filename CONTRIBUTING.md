# Contributing to Nula

We’re thrilled that you’re interested in contributing to **Nula**, our innovative programming language built with Rust, Go, and Zig! Nula is designed to be a collaborative project, and we want as many people as possible to join in expanding and improving it. Whether you're a beginner or an experienced developer, your contributions can help shape the future of Nula. From fixing bugs and adding features to improving documentation, writing examples, or even helping with installation scripts—every effort counts. Let's build this together!

This guide outlines how you can get involved, the project structure, and best practices to ensure a smooth contribution process.

## Project Overview
Nula is a programming language project that combines multiple technologies:
- **Core Implementation**: Written in Rust (located in `/nula`), Go (in `/nula-go`), and Zig (in `/nula-zig`).
- **Installation Scripts**: PowerShell (`.ps1`) and Bash (`.sh`) scripts in `/install` for easy setup on Windows, Linux, and macOS.
- **No Dedicated Docs Folder**: Documentation is currently integrated into the codebase (e.g., README.md, inline comments). We encourage contributions to expand docs—propose adding a `/docs` folder if needed!

The project is modular, allowing contributors to focus on specific parts based on their expertise in Rust, Go, Zig, or scripting.

## How You Can Help
We welcome contributions from everyone! Here are some ways to get involved:

### 1. Reporting Issues
If you find a bug, spot a documentation gap, or have an idea for a new feature:
- Check the [GitHub Issues](https://github.com/Nula-Lang/Nula/issues) to see if it's already reported.
- Open a new issue with:
  - A descriptive title.
  - Detailed description, including steps to reproduce (for bugs).
  - Screenshots, logs, or code snippets if relevant.
  - Label it appropriately (e.g., `bug`, `enhancement`, `rust`, `go`, `zig`, `install`).

### 2. Discussions and Suggestions
Share ideas, ask questions, or propose major changes:
- Use [GitHub Discussions](https://github.com/Nula-Lang/Nula/discussions) for brainstorming features, language design, or community feedback.
- Topics could include new syntax ideas, performance optimizations, or cross-platform improvements.
- Be constructive and respectful—diverse perspectives make Nula stronger!

### 3. Direct Code Contributions
Contribute to the codebase in Rust, Go, Zig, or installation scripts. Follow these steps:
1. **Fork the Repository**:
   - Fork [Nula on GitHub](https://github.com/Nula-Lang/Nula) to your account.
2. **Set Up Your Environment** (see [Development Environment Setup](#development-environment-setup) below).
3. **Create a Branch**:
   - Clone your fork: `git clone https://github.com/your-username/Nula.git`.
   - Create a feature branch: `git checkout -b feature/your-feature-name` or `bugfix/issue-number`.
4. **Make Changes**:
   - Focus on one area (e.g., Rust in `/nula`, Go in `/nula-go`).
   - Write clean code following the guidelines below.
   - Test locally: Build and run tests for the affected components.
   - Update any relevant documentation or comments.
5. **Commit and Push**:
   - Use clear commit messages: `[Rust] Add feature X` or `[Install] Fix Windows script bug`.
   - Push to your fork: `git push origin your-branch`.
6. **Open a Pull Request (PR)**:
   - Target the `main` branch.
   - Describe your changes, link to related issues (e.g., `Closes #123`).
   - Ensure the PR passes CI checks.
   - Be ready for feedback—collaborate with reviewers!

### 4. Documentation and Examples
- Improve inline comments, README.md, or propose new docs.
- Contribute code examples, tutorials, or benchmarks to showcase Nula's features.
- If documentation grows, suggest creating a `/docs` folder via a PR.

### 5. Installation and Tooling
- Enhance scripts in `/install` for better cross-platform support.
- Add CI/CD configurations or build tools.

### 6. Community Building
- Spread the word about Nula on social media, forums, or blogs.
- Translate docs or error messages to other languages.
- Organize or participate in hackathons focused on Nula.

### 7. Direct Contact
Unsure where to start? Email us at **voidarcstudio@gmail.com** for guidance, mentorship, or collaboration ideas. We're here to help newcomers!

## Guidelines
To keep Nula maintainable and welcoming:

### General Guidelines
- **Inclusivity**: Follow our [Code of Conduct](https://github.com/Nula-Lang/Nula/blob/main/CODE_OF_CONDUCT.md) (add one if missing!).
- **Scope**: Keep PRs focused—one feature or fix per PR.
- **Testing**: Always test changes. Add unit/integration tests where possible.
- **Licensing**: Ensure your contributions align with the project's license.

### Coding Guidelines by Language
- **Rust (/nula)**:
  - Follow Rust's idioms: Use safe code, leverage ownership/borrowing.
  - Run `cargo fmt` and `cargo clippy` before committing.
  - Add tests with `cargo test`.
- **Go (/nula-go)**:
  - Use Go modules; keep code simple and idiomatic.
  - Format with `go fmt`; lint with `go vet`.
  - Test with `go test`.
- **Zig (/nula-zig)**:
  - Embrace Zig's safety features and comptime.
  - Build and test with `zig build` and `zig test`.
- **Installation Scripts (/install)**:
  - Make scripts idempotent and error-resistant.
  - Test on multiple OS (Windows via PowerShell, Unix via Bash).
  - Use comments to explain logic.

### Documentation Guidelines
- Use Markdown for readability.
- Include code snippets with syntax highlighting.
- Explain "why" alongside "how" for complex changes.

### Pull Request Best Practices
- Reference issues in the description.
- Include before/after examples if applicable.
- Respond to reviews within a few days.
- Rebase if needed: `git rebase main`.

## Development Environment Setup
To build and test Nula locally:
1. **Prerequisites**:
   - Rust: Install via [rustup](https://rustup.rs/).
   - Go: Download from [golang.org](https://golang.org/).
   - Zig: Get from [ziglang.org](https://ziglang.org/).
   - Git for version control.
2. **Clone and Build**:
   - Clone: `git clone https://github.com/Nula-Lang/Nula.git`.
   - For Rust: `cd nula && cargo build`.
   - For Go: `cd nula-go && go build`.
   - For Zig: `cd nula-zig && zig build`.
   - Run install scripts: `./install/install.sh` (Unix) or `.\install\install.ps1` (Windows).
3. **Testing**:
   - Rust: `cargo test` in `/nula`.
   - Go: `go test ./...` in `/nula-go`.
   - Zig: `zig test` in `/nula-zig`.
   - End-to-end: Test the full language setup after building.
4. **Tools**:
   - Use VS Code or your preferred IDE with extensions for Rust, Go, Zig.
   - For debugging: `cargo run`, `go run`, `zig run`.

## Getting Started for New Contributors
- Look for issues labeled `good first issue`, `help wanted`, or `beginner-friendly`.
- Start small: Fix a typo, improve a comment, or add a test.
- Join discussions to learn more about Nula's vision.
- If you're new to open source, check resources like [GitHub's guide](https://docs.github.com/en/get-started/quickstart/contributing-to-projects).

Thank you for joining the Nula community! With your help, we can make Nula a powerful, accessible programming language. Let's collaborate and grow this project together—every contribution brings us closer! 🚀
