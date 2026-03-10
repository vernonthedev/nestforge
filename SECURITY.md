# Security Policy

## Supported Versions

NestForge security fixes are applied to the latest released `main` line first.
Older releases may receive patches at maintainer discretion, but only the most
recent published version should be considered actively supported by default.

## Reporting a Vulnerability

Do not open public GitHub issues for suspected vulnerabilities.

Report security issues through one of these private channels:

- GitHub Security Advisories:
  `https://github.com/vernonthedev/nestforge/security/advisories/new`
- Email:
  `techjaja2@gmail.com`

Include the following when possible:

- Affected NestForge version
- Affected crate or module
- Rust version and operating system
- Minimal proof of concept or reproduction steps
- Security impact assessment
- Any known mitigations or workarounds

## Response Expectations

Maintainers will aim to:

- acknowledge receipt within 5 business days
- confirm severity and reproduction status as quickly as possible
- coordinate a fix and disclosure timeline when the report is valid

Please avoid public disclosure until a fix or mitigation has been released and a
coordinated disclosure date has been agreed.

## Scope

This policy covers vulnerabilities in NestForge framework crates, official CLI
scaffolding, and first-party examples when they demonstrate framework behavior.
Third-party dependencies may still require reporting upstream where appropriate.
