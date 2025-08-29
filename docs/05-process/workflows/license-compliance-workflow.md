# License Compliance Workflow

## ABOUTME
// license-compliance-workflow.md: Automated workflows and procedures for maintaining open source license compliance in Kosmarium
// Establishes systematic processes for dependency monitoring, license validation, and compliance maintenance

## Overview

Systematic workflow for maintaining MIT license compliance and monitoring dependency license compatibility in the Kosmarium project.

## Dependency License Monitoring

### Pre-commit License Validation

#### Setup cargo-deny Configuration
Create `.deny.toml` in project root:
```toml
[licenses]
# List of licenses that are allowed to be used
allowed = [
    "MIT",
    "Apache-2.0", 
    "MIT OR Apache-2.0",
    "Apache-2.0/MIT",
    "BSD-2-Clause",
    "BSD-2-Clause OR Apache-2.0 OR MIT",
    "Unlicense",
    "Unlicense OR MIT", 
    "Zlib",
    "MIT OR Zlib OR Apache-2.0",
    "Unicode-3.0",
    "(MIT OR Apache-2.0) AND Unicode-3.0",
    "0BSD",
    "0BSD OR MIT OR Apache-2.0",
    "BSL-1.0",
    "Apache-2.0 OR BSL-1.0"
]

# List of licenses that are explicitly disallowed
denied = [
    "GPL-2.0",
    "GPL-3.0", 
    "LGPL-2.1",
    "LGPL-3.0",
    "AGPL-3.0",
    "CC-BY-SA-*",
    "EUPL-1.2"
]

# Confidence threshold for detecting a license from a license text.
confidence-threshold = 0.8

# Allow 1 or more licenses on a single dependency
exceptions = [
    # Allow Unicode license for unicode-related crates
    { allow = ["Unicode-3.0"], name = "unicode-ident" },
]
```

#### Git Pre-commit Hook
Create `.git/hooks/pre-commit`:
```bash
#!/bin/bash
# License compliance check before commit

echo "🔍 Checking license compliance..."

# Check for cargo-deny installation
if ! command -v cargo-deny &> /dev/null; then
    echo "⚠️  Installing cargo-deny for license compliance..."
    cargo install cargo-deny
fi

# Run license compliance check
if ! cargo deny check licenses; then
    echo "❌ License compliance check failed"
    echo "   Review dependencies or update .deny.toml configuration"
    exit 1
fi

echo "✅ License compliance verified"
```

### Automated CI/CD Integration

#### GitHub Actions Workflow
Create `.github/workflows/license-compliance.yml`:
```yaml
name: License Compliance

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  license-compliance:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Install cargo-deny
      run: cargo install cargo-deny
      
    - name: Check license compliance
      run: cargo deny check licenses
      
    - name: Generate license report
      run: cargo deny list --format json > license-report.json
      
    - name: Upload license report
      uses: actions/upload-artifact@v3
      with:
        name: license-report
        path: license-report.json
```

## Source File License Management

### Automated Header Addition

#### Script for Adding License Headers
Create `scripts/add-license-headers.sh`:
```bash
#!/bin/bash
# Add SPDX license headers to all Rust source files

LICENSE_HEADER="// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors"

find src/ -name "*.rs" | while read -r file; do
    # Check if file already has SPDX header
    if ! grep -q "SPDX-License-Identifier" "$file"; then
        # Create temporary file with header + original content
        {
            echo "$LICENSE_HEADER"
            echo ""
            cat "$file"
        } > "$file.tmp"
        
        # Replace original file
        mv "$file.tmp" "$file"
        echo "Added license header to: $file"
    fi
done

echo "✅ License headers updated"
```

### Header Validation

#### Check for Missing Headers
Create `scripts/validate-license-headers.sh`:
```bash
#!/bin/bash
# Validate all Rust files have proper license headers

MISSING_HEADERS=()

find src/ -name "*.rs" | while read -r file; do
    if ! grep -q "SPDX-License-Identifier: MIT" "$file"; then
        echo "❌ Missing license header: $file"
        MISSING_HEADERS+=("$file")
    fi
done

if [ ${#MISSING_HEADERS[@]} -gt 0 ]; then
    echo ""
    echo "Files missing license headers:"
    printf '%s\n' "${MISSING_HEADERS[@]}"
    echo ""
    echo "Run: ./scripts/add-license-headers.sh"
    exit 1
fi

echo "✅ All source files have proper license headers"
```

## Third-Party Attribution Management

### Generate Attribution Files

#### Using cargo-about
Install and configure:
```bash
# Install cargo-about
cargo install cargo-about

# Generate attribution template
cargo about init

# Customize about.toml for project needs
```

#### Custom about.toml Configuration
```toml
accepted = [
    "MIT",
    "Apache-2.0", 
    "MIT OR Apache-2.0",
    "BSD-2-Clause",
    "Unlicense",
    "Zlib",
    "Unicode-3.0",
    "0BSD",
    "BSL-1.0"
]

[template]
name = "attribution"
template = "templates/LICENSES.hbs"
output = "LICENSES.md"
```

#### Attribution Template
Create `templates/LICENSES.hbs`:
```handlebars
# Third-Party Licenses

This document contains the licenses of all third-party dependencies used in Kosmarium.

{{#each crates}}
## {{name}} {{version}}

**License:** {{license}}  
**Repository:** {{repository}}  
**Authors:** {{authors}}

```
{{license_text}}
```

---

{{/each}}
```

## Compliance Monitoring Procedures

### Weekly License Review

#### Automated Weekly Report
Create GitHub Action for weekly reports:
```yaml
name: Weekly License Report

on:
  schedule:
    - cron: '0 9 * * 1'  # Every Monday at 9 AM
    
jobs:
  license-report:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Generate weekly license report
      run: |
        cargo deny list --format json > weekly-license-report.json
        # Compare with previous week's report
        # Alert on changes
```

### New Dependency Workflow

#### Before Adding Dependencies
1. **Research license compatibility:**
   ```bash
   # Check potential dependency license
   cargo search <dependency-name>
   # Review on crates.io for license information
   ```

2. **Test license compliance:**
   ```bash
   # Add dependency to Cargo.toml
   cargo deny check licenses
   ```

3. **Document decision:**
   - Add to dependency decision log if non-standard license
   - Update .deny.toml if new license type approved

## Emergency Procedures

### License Violation Response

#### Immediate Actions
1. **Stop distribution** of affected versions
2. **Assess impact scope** (which versions affected)
3. **Identify resolution path:**
   - Remove problematic dependency
   - Find compatible alternative
   - Seek license exception/clarification

#### Resolution Workflow
1. **Document incident** in license-incidents.md
2. **Implement fix** (dependency replacement/removal)
3. **Validate compliance** with updated dependencies
4. **Update documentation** and compliance procedures
5. **Release corrected version** with clear changelog

### License Change Response

#### When Dependency Changes License
1. **Evaluate new license compatibility**
2. **Update .deny.toml if still compatible**  
3. **Find alternative if incompatible**
4. **Document change** in project changelog

## Documentation Maintenance

### Required Documentation Updates

#### When Adding Dependencies
- [ ] Update dependency license analysis if new license type
- [ ] Regenerate attribution files using cargo-about
- [ ] Update compliance documentation if needed

#### Quarterly Reviews
- [ ] Full dependency license re-analysis  
- [ ] Attribution file updates
- [ ] Compliance procedure review
- [ ] Tool updates (cargo-deny, cargo-about)

## Tools and Resources

### Essential Tools
- **cargo-deny**: License compliance checking
- **cargo-about**: Attribution file generation  
- **cargo-license**: License information extraction
- **SPDX identifiers**: Standard license identification

### Reference Resources
- [SPDX License List](https://spdx.org/licenses/)
- [Choose a License](https://choosealicense.com/)
- [Cargo Book - License Field](https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields)
- [FSF License Compatibility](https://www.gnu.org/licenses/license-compatibility.html)

---
**Last Updated:** August 28, 2025  
**Review Schedule:** Quarterly or when major dependencies change  
**Owner:** Project maintainer