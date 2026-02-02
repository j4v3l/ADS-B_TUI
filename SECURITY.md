# Security Policy

## 🔒 Security Overview

ADS-B TUI takes security seriously. As an application that handles aviation data and network communications, we are committed to ensuring the security and privacy of our users.

## 🚨 Reporting Security Vulnerabilities

If you discover a security vulnerability, please report it responsibly:

### 📧 Private Reporting
- **Email**: security@j4v3l.dev
- **GitHub Security Advisories**: [Report privately](https://github.com/j4v3l/ADS-B_TUI/security/advisories/new)

**Do not create public issues for security vulnerabilities.**

### 📋 What to Include
When reporting a security issue, please include:
- A clear description of the vulnerability
- Steps to reproduce the issue
- Potential impact and severity
- Any suggested fixes or mitigations
- Your contact information for follow-up

### ⏱️ Response Timeline
- **Initial Response**: Within 48 hours
- **Vulnerability Assessment**: Within 7 days
- **Fix Development**: Within 30 days for critical issues
- **Public Disclosure**: After fix is deployed and tested

## 🛡️ Security Considerations

### Network Security
- ADS-B TUI communicates with ADS-B data sources over HTTP/HTTPS
- Always use HTTPS when possible
- Be aware of data transmission over public networks

### Data Privacy
- Aircraft data may contain sensitive information
- ADS-B TUI does not store or transmit personal data
- Local data storage is minimal and configurable

### Dependencies
- We regularly audit our dependencies for security vulnerabilities
- Updates are applied promptly when security issues are discovered
- See our [dependency audit workflow](.github/workflows/ci.yml)

## 🔧 Security Best Practices for Users

### Installation
- Download releases only from official sources (GitHub Releases)
- Verify checksums when available
- Keep your system and dependencies updated

### Configuration
- Use strong, unique passwords for any authenticated services
- Limit network access to trusted ADS-B data sources
- Regularly review and update your configuration

### Data Sources
- Only connect to trusted ADS-B receivers
- Be aware of data authenticity and integrity
- Consider local network security when exposing ADS-B receivers

### Usage
- ADS-B TUI is designed for personal use
- Commercial use may require additional security considerations
- Monitor system resources when running continuously

## 📚 Security Updates

Security updates will be:
- Released as soon as possible after verification
- Documented in release notes
- Communicated through GitHub releases
- Marked with appropriate security advisories

## 🤝 Security Hall of Fame

We appreciate security researchers who help make ADS-B TUI safer. With permission, we'll acknowledge your contribution in our security hall of fame.

## 📞 Contact

For security-related questions or concerns:
- **Security Issues**: security@j4v3l.dev
- **General Security Questions**: Create a discussion in [GitHub Discussions](https://github.com/j4v3l/adsb-tui/discussions)

## 📜 Security Policy Changes

This security policy may be updated as needed. Significant changes will be communicated through:
- GitHub releases
- Repository announcements
- Direct communication to security reporters

---

**Last Updated**: February 1, 2026
