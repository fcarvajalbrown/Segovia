# Contributing to Segovia

Thanks for your interest. Segovia is in an early, pre-implementation phase — the most useful
contributions right now are issues, design feedback, and small focused PRs.

## Development setup (Windows-first)

A `.venv` at the project root is required. On Windows (PowerShell), run commands separately — do not
chain with `&&`.

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install maturin
maturin develop --release
```

`maturin develop --release` recompiles the Rust extension and installs the editable Python package.
**Re-run it after any Rust change** before running Python or tests.

## Checks before opening a PR

```powershell
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## Conventions

- **Conventional commits:** `<type>(<scope>): <description>` — lowercase, present-tense imperative.
  Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `ci`. One commit per logical change.
- **No code comments.** Names and types are the documentation.
- **Bug fixes target the root cause** — no workarounds that only make tests pass.
- **PR descriptions use STAR format:** Situation / Task / Action / Result.

## Licensing of contributions

Unless you state otherwise, any contribution you submit is dual-licensed under
**MIT OR Apache-2.0**, the same terms as the project, without any additional conditions.
