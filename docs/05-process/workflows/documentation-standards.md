# Desert Island Games Documentation Standards

<!-- ABOUTME: Standardized documentation organization system for Desert Island Games projects -->
<!-- PURPOSE: Information architecture framework that scales from simple to complex technical projects -->

## Overview

This specification defines a standardized documentation organization system for Desert Island Games projects, designed to transform documentation chaos into systematic, maintainable, and discoverable knowledge architecture.

## Current Problem Analysis

Alpha Prime's current docs/ directory contains 80+ files with no clear organization:
- Everything in one flat directory creates cognitive overload
- No separation between living documents vs point-in-time reports  
- Hard to distinguish specifications vs implementation reports vs research
- No clear naming conventions or version control
- Difficult to find current/authoritative versions

## Standardized Directory Structure

```
docs/
├── 00-project/           # Project-level documents (mandatory)
├── 01-architecture/      # Technical specifications and ADRs (mandatory)
├── 02-design/           # Game design, UX, visual specs (optional)
├── 03-implementation/   # Sprint reports, code reviews (mandatory)
├── 04-analysis/         # Research, mathematical analysis (optional)
├── 05-process/          # Workflows, agent definitions (mandatory)
├── assets/              # Images, data files, supporting materials (optional)
└── archive/             # Completed/obsolete documents (created as needed)
```

### 00-project/ - Project Management Documents

**Purpose:** Living project status and coordination documents
**Audience:** All project stakeholders
**Update Frequency:** Continuous

```
00-project/
├── README.md           # Project overview and onboarding
├── status.md           # Current session handoff (replaces session-handoff.md)
├── roadmap.md          # Implementation milestones and progress
└── team.md            # Team member roles and responsibilities
```

**Required Documents:**
- `status.md` - Current implementation status, blocking issues, next steps
- `roadmap.md` - Milestones, progress tracking, completion metrics
- `README.md` - Project overview, setup instructions, key concepts

### 01-architecture/ - Technical Specifications

**Purpose:** Permanent technical decisions and specifications
**Audience:** Technical implementers and future maintainers  
**Update Frequency:** Major versions only, archive old versions

```
01-architecture/
├── README.md              # Architecture overview and index
├── specifications/        # Technical specifications
│   ├── language-spec.md
│   ├── vm-spec.md
│   └── api-spec.md
├── adr/                   # Architecture Decision Records
│   ├── adr-001-vm-design.md
│   ├── adr-002-language-syntax.md
│   └── template.md
└── data-models/          # Schema definitions and data architecture
    ├── data-architecture.md
    └── configuration-schema.md
```

**Naming Conventions:**
- ADRs: `adr-NNN-descriptive-title.md` (numbered sequentially)
- Specifications: `component-spec.md` pattern
- All technical specs are living documents with version history

### 02-design/ - Game and UX Design (Optional)

**Purpose:** Design specifications and creative direction
**Audience:** Designers, UX specialists, product stakeholders
**Update Frequency:** Version-controlled with major design changes

```
02-design/
├── README.md
├── game-design/          # Game mechanics and balance
│   ├── game-design-spec.md
│   ├── balance-framework.md
│   └── victory-conditions.md
├── ux-design/           # Interface and user experience
│   ├── user-flows.md
│   ├── interface-spec.md
│   └── interaction-patterns.md
├── visual-design/       # Art direction and visual specifications
│   ├── visual-spec.md
│   └── style-guide.md
└── data/               # Design data and configurations
    ├── balance-data.json
    └── configuration-defaults.md
```

### 03-implementation/ - Implementation Records (Mandatory)

**Purpose:** Time-bound implementation progress and reviews
**Audience:** Development team, project tracking
**Update Frequency:** Per sprint/milestone, never modify after creation

```
03-implementation/
├── README.md
├── sprints/             # Sprint-based organization
│   ├── sprint-1-1/      # Phase-Sprint format
│   │   ├── planning.md
│   │   ├── completion-report.md
│   │   └── retrospective.md
│   └── sprint-1-2/
├── code-reviews/        # Systematic code review records  
│   ├── 2025-01-15-authentication-review.md
│   └── 2025-01-20-vm-optimization-review.md
├── bug-reports/         # Debugging and issue resolution
│   ├── systematic-debugging/
│   └── resolution-reports/
└── integration-tests/   # Integration testing reports
```

**Naming Conventions:**
- Sprints: `sprint-P-S/` (Phase-Sprint)
- Reviews: `YYYY-MM-DD-feature-review.md`
- All documents are point-in-time records, never modified after creation

### 04-analysis/ - Research and Analysis (Optional)

**Purpose:** Research, mathematical analysis, strategic studies
**Audience:** Researchers, analysts, strategic decision makers
**Update Frequency:** Per study completion, preserve all versions

```
04-analysis/
├── README.md
├── research/            # Mathematical and technical research
│   ├── mathematical-analysis/
│   ├── performance-studies/
│   └── feasibility-studies/
├── audits/             # Balance audits, technical debt reviews
│   ├── balance-audits/
│   ├── security-audits/
│   └── architecture-reviews/
└── experiments/        # Proof-of-concept and experimental work
    ├── prototypes/
    └── validation-studies/
```

### 05-process/ - Development Process (Mandatory)

**Purpose:** Project-specific workflows, standards, and automation
**Audience:** Development team, CI/CD systems
**Update Frequency:** As processes evolve, version control changes

```
05-process/
├── README.md
├── workflows/           # Development workflows and standards
│   ├── development-workflow.md
│   ├── testing-standards.md
│   ├── code-review-process.md
│   └── documentation-standards.md
├── agents/             # AI agent definitions and standards
│   ├── agent-definitions/
│   └── agent-workflows.md
└── templates/          # Document templates and checklists
    ├── adr-template.md
    ├── sprint-planning-template.md
    └── code-review-checklist.md
```

## Naming Conventions

### File Naming Standards
- **Kebab-case for all files and folders:** `user-authentication-spec.md`
- **Date prefixes for temporal documents:** `2025-01-15-sprint-completion.md`
- **Sequential numbering for ordered series:** `adr-001-vm-design.md`
- **Descriptive names indicating content and purpose:** `tournament-system-ux-design.md`

### Version Control
- **Living documents:** Update in place, archive old versions in `archive/`
- **Point-in-time records:** Never modify after creation
- **Version history:** Use git for version control, not filename versions

## Mandatory vs Optional Categories

### Mandatory (All Projects)
- **00-project/:** Project status and roadmap tracking
- **01-architecture/:** Technical decisions and specifications
- **03-implementation/:** Implementation progress and reviews  
- **05-process/:** Project-specific processes and standards

### Optional (Project-Dependent)
- **02-design/:** Only for projects with significant design components
- **04-analysis/:** Only for research-heavy projects
- **assets/:** Only when supporting materials are needed
- **archive/:** Created as needed when documents become obsolete

## Document Lifecycle Management

### Living Documents
- **Update in place** with clear change tracking
- **Archive old versions** when major revisions occur
- **Maintain "last updated" timestamps**
- **Use consistent front-matter for metadata**

### Point-in-Time Records
- **Never modify after creation** - they represent historical state
- **Store chronologically** within appropriate sprint/date folders
- **Reference by creation date** for clear temporal ordering

### Document Status Indicators
- **Draft:** Work in progress, subject to major changes
- **Review:** Complete but awaiting review/approval
- **Final:** Approved and stable
- **Deprecated:** Superseded by newer documents
- **Archived:** Historical record, no longer current

## Quality Gates and Standards

### Required Elements
- **ABOUTME headers:** All documents must have 2-line ABOUTME comments for greppability
- **Purpose statements:** Clear explanation of document's role and audience
- **Last updated timestamps:** For living documents
- **Cross-reference standards:** Use relative paths for portability

### Directory Organization
- **README.md in each directory** explaining contents and organization
- **Consistent folder structure** across all projects
- **Index files** linking to key documents in each category

### Automation Support
- **Consistent metadata** for automated processing
- **Standardized terminology** for search and cross-referencing
- **Template-based creation** for common document types

## Implementation Guidelines

### Migration Strategy
1. **Create new directory structure** alongside existing docs/
2. **Categorize existing documents** into appropriate folders
3. **Rename files** following naming conventions
4. **Create missing README.md files** for each directory
5. **Update cross-references** to use new paths
6. **Archive obsolete documents** appropriately

### Maintenance Process
1. **Regular audits** of document organization and relevance
2. **Archive completed sprints** and obsolete documents
3. **Update living documents** with clear change tracking
4. **Maintain cross-references** as documents move or change

### Success Metrics
- **Reduced time to find relevant documents**
- **Clear separation of concerns** between document types
- **Consistent organization** across Desert Island Games projects
- **Scalability** from simple to complex project needs

## Tools and Automation

### Search and Discovery
- **Consistent ABOUTME headers** enable grep-based search
- **Standardized terminology** improves findability
- **Cross-reference systems** connect related documents

### Index Generation
- **Automated README.md updates** with document listings
- **Table of contents generation** for major documents
- **Cross-reference validation** to catch broken links

### Template System
- **Document templates** for common formats (ADRs, sprint reports, etc.)
- **Consistent front-matter** for metadata tracking
- **Automated document creation** from templates

This documentation standards system transforms chaos into systematic knowledge architecture that scales with project complexity while maintaining discoverability and maintainability.