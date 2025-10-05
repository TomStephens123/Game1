# Game1 - 2.5D Rust Game Project

## Project Goals
1. **Create a fun 2.5D game** - Build an engaging game with compelling mechanics and visual appeal
2. **Learn Rust programming** - Use this project as a practical learning experience for Rust concepts

## Development Guidelines

### Rust Learning Focus
- Prioritize idiomatic Rust patterns over quick solutions
- Explain ownership, borrowing, and lifetime concepts as they arise
- Use this project to explore Rust's type system, error handling, and memory safety
- Experiment with Rust game development ecosystem (Bevy, ggez, etc.)

### Game Development Approach
- Start with core mechanics before polish
- Prototype quickly, then refactor for better Rust patterns
- Focus on 2.5D visual style (3D graphics with constrained movement/perspective)
- Keep scope manageable to ensure completion

### Code Quality Standards
- Use `cargo fmt` for consistent formatting
- Run `cargo clippy` for linting and Rust best practices
- When running the game always fix warnings as well as errors - unless they relate to a feature that is implemented but not extended yet (e.g. you've set up but not used one of the animation states yet)
- Write tests for core game logic
- Document complex algorithms and game mechanics

### Recommended Commands
- `cargo run` - Run the game in development mode
- `cargo test` - Run all tests
- `cargo clippy` - Check for common mistakes and improvements
- `cargo fmt` - Format code according to Rust standards

## Learning Checkpoints
Track Rust concepts learned through this project:
- [ ] Basic ownership and borrowing
- [ ] Pattern matching and enums
- [ ] Error handling with Result/Option
- [ ] Traits and generics
- [ ] Concurrency and async programming
- [ ] Game-specific patterns (ECS, state machines)

## Documentation
- Place all documentation in docs/
- Document key architectural decisions and their Rust-specific reasoning as the project develops.
- Refer to documentation if work relates to higher level architecture decisions