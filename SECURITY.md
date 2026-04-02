# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in cql-lint, please report it through
[GitHub Security Advisories](https://github.com/hiboma/cql-lint/security/advisories/new).

**Do not open a public issue for security vulnerabilities.**

When reporting, please include:

- A description of the vulnerability
- Steps to reproduce the issue
- The potential impact (e.g., denial of service, arbitrary code execution)

We will acknowledge your report on a best-effort basis, typically within 7 days.
Once confirmed, we will work on a fix and coordinate disclosure with you.

## Security Measures

This project takes the following steps to maintain supply chain security:

- **Dependency auditing**: `cargo-audit` and `cargo-deny` run in CI to detect known vulnerabilities in dependencies.
- **Automated dependency updates**: Dependabot monitors and proposes updates for outdated or vulnerable dependencies.
- **Pinned GitHub Actions**: All third-party actions are pinned by commit SHA to prevent supply chain attacks through compromised tags.
- **Minimal workflow permissions**: GitHub Actions workflows use explicitly scoped permissions, granting only what each job requires.
