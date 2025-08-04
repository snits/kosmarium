# Journal Integration Protocol Template

## Add this section to all agent .md files:

```markdown
## Journal Integration Protocol

### MANDATORY: Task Start - Query Journal
BEFORE beginning any analysis or implementation:
1. **Search for relevant experience**: Use `mcp__private-journal__search_journal` with queries like:
   - "{agent_name} + {domain_keywords}" 
   - "lessons about {current_task_type}"
   - "{specific_technology_stack} insights"
2. **Load context**: Review relevant past experiences to avoid repeating mistakes
3. **Build on previous work**: Reference successful patterns and failed approaches

### MANDATORY: Task End - Update Journal  
BEFORE completing task and returning results:
1. **Document insights**: Use `mcp__private-journal__process_thoughts`
2. **Include agent identification**: Set `agent_id` parameter to "{agent_name}"
3. **Capture key learnings**:
   - Technical discoveries and breakthrough moments
   - Failed approaches and why they didn't work
   - Successful patterns worth repeating
   - Domain-specific insights and gotchas
   - User preferences and collaboration patterns observed
4. **Tag for searchability**: Include project context and key terms for future retrieval

### Journal Entry Guidelines
- **Be specific**: "React hook dependency issue with useEffect" not "hook problem"
- **Include context**: What was the goal, what went wrong, how was it fixed
- **Note user patterns**: Communication preferences, decision-making style, technical preferences
- **Document domain insights**: Architecture decisions, performance discoveries, debugging techniques
```

## Implementation Notes
- Add this section near the end of each agent file, before any closing sections
- Maintain each agent's unique voice and domain expertise
- The journal integration should enhance, not replace, domain-specific workflows