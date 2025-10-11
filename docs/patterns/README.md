# Game Development Patterns

**Practical guides for building Game1**

This directory contains step-by-step patterns and best practices for common development tasks in Game1.

## Available Patterns

### [Entity Pattern](./entity-pattern.md) ⭐

**When to use**: Adding any new entity (characters, objects, items) to the game

**What it covers**:
- ✅ Anchor-based positioning system
- ✅ Rendering with depth sorting
- ✅ Collision systems (environmental + damage)
- ✅ Save/load integration
- ✅ Complete examples (treasure chest, goblin, door)
- ✅ Common pitfalls and solutions

**Quick start**: Full checklist and step-by-step guide from struct to integration

---

## Pattern Philosophy

These patterns are designed to:

1. **Be Practical**: Real code you can copy and adapt
2. **Explain Why**: Understand the reasoning, not just the how
3. **Teach Rust**: Learn Rust concepts through game development
4. **Stay Consistent**: Build a coherent, maintainable codebase
5. **Prevent Bugs**: Avoid common mistakes before they happen

## Pattern Template

When adding new patterns to this directory, follow this structure:

```markdown
# Pattern Name

**Quick description of what this pattern solves**

## Table of Contents
- Quick Reference (checklist)
- The Problem (what you're trying to achieve)
- The Solution (step-by-step implementation)
- Common Pitfalls (what not to do)
- Examples (real working code)
- Related Documentation
```

## How to Use These Patterns

1. **Starting a new feature?** Check if there's a pattern for it
2. **Follow the checklist** at the top of each pattern
3. **Copy the examples** and adapt to your needs
4. **Refer back** when stuck or debugging
5. **Contribute improvements** when you find better approaches

## Contributing Patterns

Found a better way to do something? Create a new pattern or improve an existing one!

**Good candidates for new patterns**:
- UI components (menus, dialogs, HUD elements)
- Animation systems (sprite sheets, state machines)
- AI behaviors (pathfinding, state machines)
- World generation (tiles, rooms, procedural content)
- Combat systems (attacks, damage, effects)

## Related Documentation

- **Systems** (`docs/systems/`): Architecture and design of core systems
- **Features** (`docs/features/`): Specific feature implementations
- **Patterns** (this dir): Reusable development patterns

---

**Remember**: These patterns are living documents. Update them as the project evolves!
