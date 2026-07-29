# Contributing to this project

First off, thanks for taking the time to contribute!

The following is a set of guidelines for contributing to this repository. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Development Workflow

We follow the standard **Fork & Pull** workflow combined with a simplified **Gitflow** branching strategy.

1.  **Fork** the repository on GitHub.
2.  **Clone** your fork locally.
3.  **Create a Branch** for your specific task (do not work directly on `main`).
4.  **Commit** your changes following the conventions.
5.  **Push** to your fork and submit a **Pull Request**.

### Branching Strategy

Your branch names should be descriptive and use the following prefixes:

* `feature/` for new features (e.g., `feature/backup-system`)
* `fix/` for bug fixes (e.g., `fix/login-error`)
* `docs/` for documentation updates
* `chore/` for maintenance and config changes

## Commit Convention

We use **Conventional Commits** to ensure a clean history and automated release notes. Please ensure your commit messages follow this format:

```text
<type>: <description>

[optional body]

```

### Allowed Types

* **feat**: A new feature
* **fix**: A bug fix
* **docs**: Documentation only changes
* **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc)
* **refactor**: A code change that neither fixes a bug nor adds a feature
* **perf**: A code change that improves performance
* **test**: Adding missing tests or correcting existing tests
* **chore**: Changes to the build process or auxiliary tools

### Examples

* `feat: add dark mode to settings`
* `fix: prevent crash when database is locked`
* `chore: update dependencies`

## Pull Request Guidelines

When you are ready to submit your Pull Request, please follow these steps:

1. **Search** to see if a similar PR already exists.
2. **Link the Issue** related to your PR in the description (e.g., "Closes #123") to auto-close it upon merge.
3. Ensure your code adheres to the existing coding style.
4. Keep your PR small and focused on a single task. If you have multiple distinct changes, please submit separate PRs.

