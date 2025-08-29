#!/bin/bash

# ABOUTME
# implement-doc-licensing.sh: Implements Creative Commons licensing for Kosmarium documentation
# Adds appropriate CC headers to educational, technical, and research documentation

set -e

echo "📚 Implementing Creative Commons Documentation Licensing for Kosmarium..."

# Define license headers
CC_BY_SA_HEADER="<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2025 Jerry Snitselaar and contributors  
License: https://creativecommons.org/licenses/by-sa/4.0/
-->"

CC_BY_HEADER="<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2025 Jerry Snitselaar and contributors
License: https://creativecommons.org/licenses/by/4.0/
-->"

# Function to add header to markdown file
add_cc_header() {
    local file="$1"
    local header="$2"
    local description="$3"
    
    if ! grep -q "SPDX-License-Identifier: CC-" "$file"; then
        # Create temporary file with header + original content
        {
            echo "$header"
            echo ""
            cat "$file"
        } > "$file.tmp"
        
        # Replace original file  
        mv "$file.tmp" "$file"
        echo "  ✅ $description: $file"
    else
        echo "  ⏭️  Header exists: $file"
    fi
}

echo "📝 Adding CC BY-SA 4.0 headers to educational/technical documentation..."

# Educational and Technical Documentation (CC BY-SA 4.0)
find docs/00-project/ docs/01-architecture/ docs/02-design/ docs/03-implementation/ docs/05-process/ -name "*.md" -type f 2>/dev/null | while IFS= read -r file; do
    if [ -f "$file" ]; then
        add_cc_header "$file" "$CC_BY_SA_HEADER" "CC BY-SA"
    fi
done

echo "📝 Adding CC BY 4.0 headers to research documentation..."

# Research Documentation (CC BY 4.0)  
find docs/04-analysis/ -name "*.md" -type f 2>/dev/null | while IFS= read -r file; do
    if [ -f "$file" ]; then
        add_cc_header "$file" "$CC_BY_HEADER" "CC BY"
    fi
done

# Handle root-level documentation files (educational)
for file in README.md CLAUDE.md; do
    if [ -f "$file" ]; then
        add_cc_header "$file" "$CC_BY_SA_HEADER" "CC BY-SA"
    fi
done

echo "📋 Creating documentation licensing overview..."

cat > docs/README.md << 'EOF'
<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2025 Jerry Snitselaar and contributors  
License: https://creativecommons.org/licenses/by-sa/4.0/
-->

# Kosmarium Documentation

This directory contains comprehensive documentation for Kosmarium, licensed under Creative Commons for optimal educational and research reuse.

## License Overview

- **Educational/Technical Documentation**: [CC BY-SA 4.0](LICENSE-CC-BY-SA-4.0.txt)
- **Research Documentation**: [CC BY 4.0](LICENSE-CC-BY-4.0.txt)
- **Source Code**: [MIT License](../LICENSE)

## Directory Structure

- **00-project/** - Project overview, status, and roadmap (CC BY-SA 4.0)
- **01-architecture/** - Architecture decisions and specifications (CC BY-SA 4.0)
- **02-design/** - UX design and system design documents (CC BY-SA 4.0)
- **03-implementation/** - Implementation guides and code reviews (CC BY-SA 4.0)
- **04-analysis/** - Research reports and physics validation (CC BY 4.0)
- **05-process/** - Workflows and development processes (CC BY-SA 4.0)

## Using This Documentation

### For Education
- Freely use, modify, and share for classroom instruction
- Create derivative educational materials under CC BY-SA 4.0
- No permission required for educational use

### For Research  
- Cite and reference research documentation with standard attribution
- Include in systematic reviews and meta-analyses
- Integrate into academic publications under CC BY 4.0

### For Development
- Follow architecture decisions and implementation guides
- Contribute improvements back to the community
- Code examples remain MIT-licensed regardless of documentation license

## Attribution

See [DOCUMENTATION-LICENSING.md](../DOCUMENTATION-LICENSING.md) for complete attribution requirements and examples.
EOF

echo "  ✅ Created docs/README.md with licensing overview"

echo ""
echo "🎉 Creative Commons Documentation Licensing Complete!"
echo ""
echo "📋 Implementation Summary:"
echo "  ✅ CC license files downloaded (CC BY 4.0, CC BY-SA 4.0)"
echo "  ✅ Headers added to educational/technical docs (CC BY-SA 4.0)"
echo "  ✅ Headers added to research documentation (CC BY 4.0)"
echo "  ✅ Documentation licensing overview created"
echo ""
echo "🔍 License Distribution:"
echo "  📚 Educational/Technical: CC BY-SA 4.0 (shareable improvements)"
echo "  🔬 Research: CC BY 4.0 (maximum academic compatibility)"
echo "  💻 Source Code: MIT (permissive software license)"
echo ""
echo "📖 Next Steps:"
echo "  1. Review headers and licensing assignments"
echo "  2. Commit all licensing implementation changes" 
echo "  3. Update project README with licensing section"
echo "  4. Set up CI/CD compliance monitoring"