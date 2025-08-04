# Agent Interaction Patterns

ABOUTME: Documented patterns for effective agent collaboration and troubleshooting
ABOUTME: Records what works, what fails, and best practices for agent communication

## Effective Agent Communication Patterns

### File Output Pattern ✅ PROVEN EFFECTIVE

**When to Use**: Agent gives "Tool ran without output or errors" or silent failures
**Pattern**: 
```
IMPORTANT: Write your analysis directly to a file at `docs/[topic].md` as you work. 
Keep each section concise (2-3 paragraphs max).
Use the Write tool to create the analysis file. Be concise and practical.
```

**Why It Works**: Bypasses response length/context limits while preserving full analysis
**Example Success**: social-systems-designer multi-scale analysis (72 lines successfully captured)

### Sequential vs Parallel Execution

**Learned**: Sequential agent execution more reliable than parallel for complex tasks
**Pattern**: Launch one agent, wait for completion, then launch next
**Avoid**: Multiple complex agent tasks simultaneously

### Prompt Complexity Management

**Short, Focused Prompts**: Work better for agents with technical issues
**File-Based Detailed Work**: When you need comprehensive analysis
**Iterative Refinement**: Better than one large complex prompt

## Agent-Specific Behavior Patterns

### social-systems-designer
- **Issue**: Complex prompts cause silent failures
- **Solution**: File output with concise section requirements
- **Tools**: Has Write/Edit tools, use them proactively

### simulation-designer
- **Behavior**: Handles complex technical prompts well
- **Strength**: Architectural analysis and system design

### systems-architect  
- **Behavior**: Builds well on existing codebase patterns
- **Strength**: Integration with current architecture

## Troubleshooting Checklist

When agent fails:
1. Try shorter, more focused prompt
2. Use file output pattern if complex analysis needed
3. Check agent tool permissions (some have limited toolsets)
4. Try sequential instead of parallel execution
5. Break complex tasks into smaller increments

---

**Last Updated**: 2025-08-01  
**Context**: Lessons from pitch experiment and multi-scale architecture analysis