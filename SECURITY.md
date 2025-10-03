# Nula - Security Policy

## Introduction

This document outlines the security policy for the Nula programming language, including supported versions, security update practices, and procedures for reporting vulnerabilities. Ensuring the security of Nula is a priority to maintain trust and reliability for all users.

Nula is a modern programming language created on September 28, 2025, designed for simplicity and high performance. This policy applies to the Nula compiler, runtime, and associated tools as hosted on the official GitHub repository: [https://github.com/Nula-Lang/Nula/](https://github.com/Nula-Lang/Nula/).

## Supported Versions

The Nula project actively maintains security updates for specific versions of the language. Below is a table detailing the support status for each version as of October 2025.

| Version | Supported |
|-------|-------------------|
| v0.4  | ❌ Problems with arguments write etc. the compiler does not work |
| v0.3  | ❌ Not Supported. |
| v0.2  | ❌ Not Supported. |
| v0.1  | ❌ Not Supported. |

> **Note**:  
> Fully Supported versions receive regular security patches, bug fixes, and performance improvements.  
> Partially Supported versions receive only critical security updates for a limited period.  
> Users on unsupported versions should upgrade to the latest supported version to ensure security and compatibility.

### Version History

- **v0.3.x** (Latest, Released October 2025): Introduces enhanced runtime security, improved dependency management, and support for atomic Linux distributions.
- **v0.2.x** (Released October 2025): Added initial support for Zig-based compilation and basic Python translation features.
- **v0.1.x** (Released September 2025): Initial release with core syntax and interpreted mode.

To check your installed Nula version:

```bash
nula --version
```

To upgrade to the latest version, follow the installation instructions in the official documentation.

## Reporting a Vulnerability

If you discover a security vulnerability in Nula, we encourage responsible disclosure to help us address the issue promptly. Below is the process for reporting vulnerabilities.

### How to Report

**Submit a Report**:
- Email your vulnerability report to [security@nula-lang.org](mailto:security@nula-lang.org).
- Alternatively, create a private issue on the Nula GitHub repository: [https://github.com/Nula-Lang/Nula/issues](https://github.com/Nula-Lang/Nula/issues) (use the "Security" label and request private visibility).

**Include Details**:
- Provide a clear description of the vulnerability.
- Include steps to reproduce the issue, if possible.
- Specify the affected version(s) (e.g., v0.3.x).
- Mention any potential impact (e.g., code execution, data exposure).

**Use Encryption (Optional but Recommended)**:
- For sensitive disclosures, encrypt your report using our public PGP key (available on the GitHub repository).

**Avoid Public Disclosure**:
- Do not share details of the vulnerability publicly until it has been resolved and disclosed by the Nula team.

### What to Expect

- **Acknowledgment**: You will receive a confirmation of receipt within 48 hours.
- **Updates**: We aim to provide updates on the status of your report every 7 days.
- **Resolution Timeline**:
  - Critical vulnerabilities: Addressed within 7–14 days.
  - Moderate vulnerabilities: Addressed within 30 days.
  - Low-priority issues: Scheduled for the next minor release.
- **Outcome**:
  - **Accepted**: If the vulnerability is valid, we will work on a fix and credit you in the release notes (unless you prefer anonymity).
  - **Declined**: If the report is not a security issue or is out of scope (e.g., user error, unsupported versions), we will provide a clear explanation.

> **Warning**:  
> Please refrain from exploiting vulnerabilities in production environments or sharing exploit code publicly before a fix is released. Violating this may result in exclusion from the acknowledgment process.

### Scope of Vulnerabilities

We consider vulnerabilities in the following components:
- Nula compiler and runtime.
- Standard libraries included with Nula.
- Installation scripts (e.g., `install.sh`, `install.ps1`).
- CLI tools (`nula` command).

Out-of-scope issues include:
- Vulnerabilities in unsupported versions (e.g., v0.1.x or earlier).
- Issues in third-party dependencies (report to the respective projects).
- Misconfigurations or user errors not directly tied to Nula's codebase.

## Security Best Practices for Users

To ensure secure usage of Nula:

- **Use Supported Versions**:
  - Always run the latest supported version (e.g., v0.3.x) to benefit from security patches.
- **Verify Installation Scripts**:
  - Download scripts only from the official repository: [https://raw.githubusercontent.com/Nula-Lang/Nula/main/install/](https://raw.githubusercontent.com/Nula-Lang/Nula/main/install/).
  - Check script integrity using checksums (published in release notes).
- **Secure Dependency Management**:
  - Use trusted sources for dependencies (`<(dependency)>` or `bottles.nula`).
  - Audit dependencies before installation with `nula install {dep}`.
- **Run in Safe Environments**:
  - Test untrusted Nula code in isolated environments (e.g., containers or VMs).
- **Monitor Updates**:
  - Subscribe to the Nula GitHub repository for release notifications and security advisories.

## Security Updates and Disclosure

The Nula team is committed to transparency and timely updates.

- **Security Advisories**: Published on the GitHub repository under the "Security" tab.
- **Release Notes**: Include details of security fixes with CVE identifiers (if applicable).
- **Disclosure Policy**:
  - We disclose vulnerabilities only after a fix is released.
  - Affected users are notified via GitHub issues or the Nula mailing list.

> **Note**:  
> If a vulnerability affects v0.2.x or v0.3.x, a patch will be issued for both branches unless v0.2.x has reached end-of-life (EOL). Check the repository for EOL announcements.

## Installation Instructions for Secure Setup

To ensure a secure installation, follow these steps for your operating system.

### Windows

**Download Installation Script**:

```powershell
curl -L -o "$env:TEMP\install.ps1" https://raw.githubusercontent.com/Nula-Lang/Nula/main/install/install.ps1
```

**Run as Administrator**:

```powershell
Start-Process powershackell -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File \"$env:TEMP\install.ps1\""
```

### Linux, Unix, and macOS

**Download Installation Script**:

```bash
curl -L -o /tmp/install.sh https://raw.githubusercontent.com/Nula-Lang/Nula/main/install/install.sh
```

**Run the Script**:

```bash
cd /tmp && sudo chmod +x ./install.sh && ./install.sh
```

### Linux Atomic Distributions

**Download and Modify Script**:

```bash
curl -L -o /tmp/install.sh https://raw.githubusercontent.com/Nula-Lang/Nula/main/install/install.sh && sed -i 's/is_atomic=false/is_atomic=true/' /tmp/install.sh
```

**Run the Script**:

```bash
cd /tmp && sudo chmod +x ./install.sh && ./install.sh
```

> **Tip**:  
> Verify the script's checksum before execution. Checksums are published in the release notes on GitHub.

## Contributing to Security

We welcome contributions to improve Nula’s security. To contribute:

- Fork the repository: [https://github.com/Nula-Lang/Nula/](https://github.com/Nula-Lang/Nula/).
- Submit patches or improvements via pull requests.
- Use the "Security" label for security-related contributions.

> **Note**:  
> Security-related pull requests may be reviewed privately to prevent premature disclosure.

## Contact Information

For security-related inquiries:
- **Email**: [voidarcstudio@gmail.com](mailto:voidarcstudio@gmail.com)
- **GitHub**: [https://github.com/Nula-Lang/Nula/](https://github.com/Nula-Lang/Nula/)
- **PGP Key**: Available in the repository’s SECURITY.md

For general support or questions:
- Open an issue on GitHub.
- Join the Nula community (details in the repository’s README).

## Acknowledgments

We thank all researchers and contributors who responsibly report vulnerabilities. Your efforts help keep Nula secure. Acknowledged contributors are listed in release notes unless anonymity is requested.
