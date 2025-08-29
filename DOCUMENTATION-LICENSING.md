# Kosmarium Documentation Licensing Strategy

## Overview

Kosmarium uses dual licensing to optimize both software development and educational/research use:

- **Source Code**: MIT License (permissive, industry-standard)
- **Documentation**: Creative Commons licenses (optimized for educational reuse)

## License Applications by Content Type

### Educational Materials → CC BY-SA 4.0
**Content Types:**
- Educational deep-dives and tutorials
- User guides and how-to documentation  
- CLI documentation and examples
- Learning materials and explanatory content

**License:** [Creative Commons Attribution-ShareAlike 4.0 International](https://creativecommons.org/licenses/by-sa/4.0/)

**Why ShareAlike:** Ensures educational improvements are shared back with the community, building collective educational resources.

### Research Documentation → CC BY 4.0
**Content Types:**
- Physics validation reports
- Mathematical analyses and proofs
- Scientific papers and research findings
- Experimental results and data analysis

**License:** [Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/)

**Why Attribution Only:** Maximum compatibility with academic publishing and research workflows.

### Technical Documentation → CC BY-SA 4.0
**Content Types:**
- Architecture Decision Records (ADRs)
- Implementation specifications
- Development workflows and standards
- System design documentation

**License:** [Creative Commons Attribution-ShareAlike 4.0 International](https://creativecommons.org/licenses/by-sa/4.0/)

**Why ShareAlike:** Encourages collaborative improvement while ensuring community benefits.

## Mixed Content Handling

### Documentation with Code Examples

When documentation contains code snippets:

```markdown
# Example Documentation Header

**Documentation License:** [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)

**Code Examples:** All code snippets in this document are licensed under the [MIT License](../LICENSE) and may be used independently of this documentation.

---

## Usage Example

The following code demonstrates basic usage:

```rust
// This code snippet is MIT-licensed
fn main() {
    println!("Hello, Kosmarium!");
}
```

### Source Files with Extensive Documentation

For source files with substantial documentation content:

```rust
// SPDX-License-Identifier: MIT
// Documentation portions: CC BY-SA 4.0
// Copyright (c) 2025 Jerry Snitselaar and contributors

//! # Kosmarium Physics Engine
//! 
//! **Documentation License:** CC BY-SA 4.0
//! **Code License:** MIT
//! 
//! This module implements planetary physics simulation with educational
//! explanations provided under Creative Commons licensing.
```

## Attribution Requirements

### For CC BY Content (Research Documentation)
When reusing research documentation:

```
This material includes content from "Kosmarium" by Jerry Snitselaar and contributors, 
licensed under CC BY 4.0. Available at: https://github.com/[repo-url]
```

### For CC BY-SA Content (Educational/Technical Documentation)
When reusing or adapting educational content:

```
This material is based on "Kosmarium Documentation" by Jerry Snitselaar and contributors, 
licensed under CC BY-SA 4.0. Adaptations are available under the same license.
Available at: https://github.com/[repo-url]
```

## Educational Use Benefits

### Classroom Integration
- **No permission required** for educational use
- **Students can modify and share** their adaptations
- **Instructors can create** derivative courses and materials
- **Institutional repositories** can freely include content

### Research Applications
- **Direct citation** in academic papers
- **Integration** into systematic reviews and meta-analyses
- **Adaptation** for different research contexts
- **No copyright clearance** required for academic use

### International Collaboration
- **Global compatibility** through CC 4.0 international licenses
- **Machine-readable licenses** for automated compliance
- **Standard attribution practices** reduce collaboration friction

## Implementation Guidelines

### Documentation File Headers

**Educational Content (CC BY-SA 4.0):**
```markdown
<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2025 Jerry Snitselaar and contributors
License: https://creativecommons.org/licenses/by-sa/4.0/
-->
```

**Research Content (CC BY 4.0):**
```markdown
<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2025 Jerry Snitselaar and contributors
License: https://creativecommons.org/licenses/by/4.0/
-->
```

### Directory Structure
```
docs/
├── LICENSE-CC-BY-4.0.txt         # Research documentation license
├── LICENSE-CC-BY-SA-4.0.txt      # Educational/technical documentation license
├── README.md                     # This file - licensing explanation
├── 00-project/                   # CC BY-SA 4.0 (educational)
├── 01-architecture/              # CC BY-SA 4.0 (technical)  
├── 02-design/                    # CC BY-SA 4.0 (technical)
├── 03-implementation/            # CC BY-SA 4.0 (technical)
├── 04-analysis/                  # CC BY 4.0 (research)
└── 05-process/                   # CC BY-SA 4.0 (educational)
```

## Contributor Guidelines

### For Documentation Contributors
1. **Understand the license** that applies to your content type
2. **Use provided templates** for attribution and headers
3. **Mark mixed content** clearly when including code examples
4. **Follow CC best practices** for attribution and derivative works

### For Reusers
1. **Check the specific license** for each piece of content
2. **Provide proper attribution** using our templates
3. **Respect ShareAlike requirements** for CC BY-SA content
4. **Separate code and documentation** licensing when reusing

## License Compatibility

### With Other Open Licenses
- **CC BY** is compatible with most other licenses
- **CC BY-SA** requires derivatives to use compatible licenses
- **MIT code** can be freely combined with CC documentation
- **Academic fair use** may apply for limited research use

### Upgrade Path
- **CC BY content** can be upgraded to CC BY-SA if desired
- **CC BY-SA content** must remain ShareAlike in derivatives
- **Version compatibility** - CC 4.0 is the current standard

## Questions and Support

For questions about licensing:

1. **Review** the [Creative Commons FAQ](https://creativecommons.org/faq/)
2. **Check** our [specific guidance](#attribution-requirements) above
3. **Contact** [maintainers] for project-specific questions
4. **Consult** legal counsel for complex commercial use cases

## Rationale

This dual licensing approach serves Kosmarium's core mission as an educational platform and research tool by:

- **Maximizing software adoption** through permissive MIT licensing
- **Optimizing educational reuse** through appropriate CC licensing
- **Supporting research workflows** with compatible attribution requirements
- **Encouraging collaborative improvement** through ShareAlike where beneficial
- **Maintaining industry standards** while serving educational goals

The approach balances practical software development needs with educational and research community benefits, positioning Kosmarium as a model for open educational/research software projects.