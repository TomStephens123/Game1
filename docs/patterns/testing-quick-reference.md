# Unit Testing Quick Reference

## Overview
This guide covers how to run and maintain unit tests in the Game1 project.

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test
```bash
cargo test test_name
```

### Run Tests in a Specific Module
```bash
cargo test module_name::
```

### Run Tests with Output
By default, `cargo test` captures stdout. To see print statements:
```bash
cargo test -- --nocapture
```

### Run Tests with Backtrace
For detailed error information on panics:
```bash
RUST_BACKTRACE=1 cargo test
```

## Test Organization

Tests are located in the same files as the code they test, within `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test code here
        assert_eq!(actual, expected);
    }
}
```

## Current Test Status

### Modules with Tests
- **collision** (5 tests) - AABB collision detection
- **combat** (8 tests) - Damage calculation and combat mechanics
- **render** (2 tests) - Depth sorting
- **sprite** (3 tests) - Animation frame handling
- **stats** (8 tests) - Health and stat modifier systems
- **the_entity** (4 tests) - Entity state and behavior
- **ui::floating_text** (2 tests) - Floating text rendering
- **ui::health_bar** (3 tests) - Health bar UI

### Known Failing Tests (as of 2025-10-11)

1. **collision::tests::test_calculate_overlap_horizontal**
   - Issue: Sign error in overlap calculation (expected 32, got -32)
   - Location: src/collision.rs:387

2. **collision::tests::test_calculate_overlap_vertical**
   - Issue: Sign error in overlap calculation (expected 32, got -32)
   - Location: src/collision.rs:399

3. **stats::tests::test_stat_modifiers_flat**
   - Issue: Flat modifiers not applying correctly (expected 15.0, got 8.0)
   - Location: src/stats.rs:420

4. **stats::tests::test_stat_modifiers_percentage**
   - Issue: Percentage modifiers not applying correctly (expected 15.0, got 4.5)
   - Location: src/stats.rs:434

5. **stats::tests::test_stat_modifiers_stacking**
   - Issue: Multiple modifiers not stacking correctly (expected 22.5, got 12.0)
   - Location: src/stats.rs:457

6. **ui::health_bar::tests::test_default_health_bar_style**
   - Issue: Border width mismatch (expected 4, got 6)
   - Location: src/ui/health_bar.rs:273

## Common Test Patterns

### Assertion Macros
```rust
assert!(condition);                    // Assert true
assert_eq!(left, right);              // Assert equality
assert_ne!(left, right);              // Assert inequality
assert!(result.is_ok());              // Assert Result is Ok
assert!(option.is_some());            // Assert Option is Some
```

### Testing Panics
```rust
#[test]
#[should_panic]
fn test_panic_condition() {
    // Code that should panic
}

#[test]
#[should_panic(expected = "error message")]
fn test_specific_panic() {
    // Code that should panic with specific message
}
```

### Testing Results
```rust
#[test]
fn test_result() -> Result<(), String> {
    let value = some_function()?;
    assert_eq!(value, expected);
    Ok(())
}
```

## Best Practices

1. **Run tests before committing**: Always run `cargo test` before committing changes
2. **Fix warnings**: The project policy is to fix warnings as well as errors
3. **Test coverage**: Add tests for new functionality as you build features
4. **Clear test names**: Use descriptive names like `test_<functionality>_<scenario>`
5. **Isolated tests**: Each test should be independent and not rely on other tests

## Integration with CI/CD

The test suite is designed to be run in continuous integration. All tests must pass before merging to main.

## Debugging Failed Tests

1. **Run the specific failing test**:
   ```bash
   cargo test test_name -- --nocapture
   ```

2. **Get full backtrace**:
   ```bash
   RUST_BACKTRACE=full cargo test test_name
   ```

3. **Check the test assertion**: The error message shows what was expected vs. what was received

4. **Review recent changes**: If a test suddenly fails, review recent changes to related code

## Performance Testing

For performance-critical code:
```rust
#[test]
#[ignore]  // Ignored by default
fn bench_performance() {
    use std::time::Instant;
    let start = Instant::now();

    // Code to benchmark

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
}
```

Run with: `cargo test --ignored`
