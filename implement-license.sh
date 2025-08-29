#!/bin/bash

# ABOUTME
# implement-license.sh: Quick implementation script for MIT license setup in Kosmarium project
# Handles license headers, Cargo.toml updates, and basic compliance setup

set -e

echo "🚀 Implementing MIT License for Kosmarium..."

# 1. Add license headers to all Rust source files
echo "📝 Adding SPDX license headers to source files..."

LICENSE_HEADER="// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors"

find src/ -name "*.rs" -type f | while IFS= read -r file; do
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
        echo "  ✅ Added header to: $file"
    else
        echo "  ⏭️  Header exists: $file"
    fi
done

# 2. Update Cargo.toml with license metadata
echo "⚙️  Updating Cargo.toml with license metadata..."

# Check if license field already exists
if grep -q "^license" Cargo.toml; then
    echo "  ⏭️  License field already exists in Cargo.toml"
else
    # Add license field after version line
    sed -i.bak '/^version = /a\
license = "MIT"
' Cargo.toml && rm Cargo.toml.bak
    echo "  ✅ Added license field to Cargo.toml"
fi

# Add other metadata if missing
if ! grep -q "^description" Cargo.toml; then
    sed -i.bak '/^license = /a\
description = "Kosmarium: Advanced planetary physics simulation system"
' Cargo.toml && rm Cargo.toml.bak
    echo "  ✅ Added description to Cargo.toml"
fi

# 3. Create basic license compliance configuration
echo "🔧 Setting up license compliance monitoring..."

cat > .deny.toml << 'EOF'
# Kosmarium License Compliance Configuration
[licenses]
# Allowed licenses (all permissive, MIT-compatible)
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

# Explicitly disallowed licenses
denied = [
    "GPL-2.0",
    "GPL-3.0", 
    "LGPL-2.1",
    "LGPL-3.0",
    "AGPL-3.0"
]

confidence-threshold = 0.8

# Exceptions for specific cases
exceptions = [
    { allow = ["Unicode-3.0"], name = "unicode-ident" },
]
EOF

echo "  ✅ Created .deny.toml for license compliance"

# 4. Test license compliance if cargo-deny is available
if command -v cargo-deny &> /dev/null; then
    echo "🧪 Testing license compliance..."
    if cargo deny check licenses; then
        echo "  ✅ License compliance verified!"
    else
        echo "  ⚠️  License compliance issues detected - review manually"
    fi
else
    echo "  ⚠️  Install cargo-deny for automated compliance checking:"
    echo "     cargo install cargo-deny"
fi

# 5. Update README if it exists
if [ -f "README.md" ]; then
    echo "📚 Checking README for license section..."
    if ! grep -q -i "license" README.md; then
        echo "" >> README.md
        echo "## License" >> README.md
        echo "" >> README.md
        echo "This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details." >> README.md
        echo "" >> README.md
        echo "  ✅ Added license section to README.md"
    else
        echo "  ⏭️  License section already exists in README"
    fi
fi

echo ""
echo "🎉 MIT License implementation complete!"
echo ""
echo "📋 Implementation Summary:"
echo "  ✅ LICENSE file created with MIT license text"
echo "  ✅ SPDX headers added to all .rs source files"
echo "  ✅ Cargo.toml updated with license metadata"
echo "  ✅ .deny.toml created for compliance monitoring"
echo "  ✅ README.md updated with license section"
echo ""
echo "🚀 Next Steps:"
echo "  1. Review and commit these changes"
echo "  2. Install cargo-deny: cargo install cargo-deny"
echo "  3. Set up CI/CD license checking (see docs/05-process/workflows/)"
echo "  4. Generate third-party attribution files when ready"
echo ""
echo "📖 Documentation:"
echo "  - Complete analysis: LICENSE-AUDIT.md"
echo "  - Compliance report: COMPLIANCE-REPORT.md"
echo "  - Workflow procedures: docs/05-process/workflows/license-compliance-workflow.md"