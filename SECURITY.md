# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.0.0-alpha.x | :white_check_mark: |

**Note**: This project is in alpha stage. Security updates will be provided for the latest alpha version.

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in consola-rs, please report it privately.

### How to Report

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please email the maintainer at:
- **Email**: muntasir.joypurhat@gmail.com

Or open a private security advisory:
1. Go to https://github.com/MuntasirSZN/consola-rs/security/advisories
2. Click "Report a vulnerability"
3. Fill out the form with details

### What to Include

When reporting a vulnerability, please include:

1. **Description**: Clear description of the vulnerability
2. **Impact**: What could an attacker do?
3. **Reproduction**: Steps to reproduce the issue
4. **Environment**: Rust version, platform, features enabled
5. **Suggested Fix**: If you have ideas for fixing it

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: 7-14 days
  - High: 14-30 days
  - Medium: 30-60 days
  - Low: Next minor release

## Security Considerations

### Dependency Management

consola-rs uses:
- `cargo-deny` for dependency auditing
- Regular dependency updates
- Minimal dependency tree
- Feature gates to reduce attack surface

### Known Security Areas

#### 1. Log Injection

**Risk**: User-controlled input in log messages could inject fake log entries

**Mitigation**: 
- Always validate/sanitize user input before logging
- Use structured logging (JSON reporter) for machine parsing
- Never log sensitive data (passwords, tokens, etc.)

#### 2. Resource Exhaustion

**Risk**: Malicious input could cause memory/CPU exhaustion

**Mitigation**:
- Throttling limits repetitive messages
- Pause queue has configurable capacity
- Depth limiting for error chains

#### 3. ANSI Injection

**Risk**: Malicious ANSI codes in log messages could manipulate terminal

**Mitigation**:
- Sanitize user input before logging
- Use `strip_ansi` utility when needed
- Consider disabling colors in production

#### 4. Dependency Vulnerabilities

**Risk**: Vulnerabilities in dependencies

**Mitigation**:
- Regular `cargo audit` runs in CI
- `cargo-deny` for automated checking
- Minimal dependency tree
- Optional features to reduce surface area

### Best Practices

#### For Library Users

1. **Validate Input**
   ```rust
   // BAD: User input directly in log
   info!("User input: {}", untrusted_input);
   
   // GOOD: Sanitize first
   let sanitized = sanitize_log_input(untrusted_input);
   info!("User input: {}", sanitized);
   ```

2. **Don't Log Secrets**
   ```rust
   // BAD: Logging sensitive data
   info!("API key: {}", api_key);
   
   // GOOD: Redact or omit
   info!("API key configured: {}", api_key.len() > 0);
   ```

3. **Use Structured Logging**
   ```rust
   // Use JSON reporter for automated parsing
   // Prevents log injection attacks
   ```

4. **Limit Log Levels in Production**
   ```rust
   // Don't expose debug/trace logs in production
   // They may contain sensitive information
   ```

#### For Contributors

1. **No Unsafe Code**: Avoid `unsafe` unless absolutely necessary
2. **Input Validation**: Validate all external input
3. **Error Handling**: Use `Result` types, avoid panics
4. **Dependency Review**: Carefully review new dependencies
5. **Audit Trail**: Document security-relevant changes

### Security Features

- ✅ No `unsafe` code in core library
- ✅ Dependency auditing via cargo-deny
- ✅ Minimal privilege principle
- ✅ Input validation
- ✅ Resource limits (throttling, depth limits)
- ✅ Error handling without panics

### Disclosure Policy

When a security issue is fixed:

1. **Private Fix**: Develop fix in private repository
2. **Advisory Draft**: Create security advisory
3. **Coordinated Release**: Release fix version
4. **Public Disclosure**: Publish advisory after users can update
5. **CVE**: Request CVE if applicable

### Security Updates

Security updates will be:
- Released ASAP for critical issues
- Backported to supported versions
- Announced via:
  - GitHub Security Advisories
  - CHANGELOG.md
  - Git tags

## Acknowledgments

We appreciate security researchers who responsibly disclose vulnerabilities. Contributors will be acknowledged in:
- Security advisory
- CHANGELOG.md
- GitHub release notes

## Questions?

For security-related questions (not vulnerabilities), you can:
- Open a public discussion on GitHub
- Check existing security advisories
- Review this document

Thank you for helping keep consola-rs secure!
