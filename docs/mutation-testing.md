# Mutation Testing Strategy

## Overview

Uses [cargo-mutants](https://github.com/sourcefrog/cargo-mutants) for mutation testing. Due to procedural macro limitations, mutation testing is informational only.

## Current Status

- **Target**: < 70% miss rate
- **Current**: ~63% miss rate (139 missed out of 221 mutants)
- **Test Count**: 181 tests

## Why Mutation Testing is Challenging for Procedural Macros

1. **Token Generation**: Much of the code generates Rust tokens. Mutations like changing `+=` to `-=` might generate invalid Rust that won't compile, but are counted as "missed" mutations.

2. **Exhaustive Matches**: The compiler ensures match statements are exhaustive, so "delete match arm" mutations often create code that won't compile.

3. **Context-Dependent Operations**: Bit arithmetic operations (`<<`, `>>`, `&`, `|`) have specific meanings in bit packing contexts that can't always be tested in isolation.

## CI Strategy

The GitHub Actions workflow runs mutation testing as an **informational job** that:
- Continues even if mutations are missed
- Generates a summary report showing miss rate
- Warns if miss rate exceeds 70%
- Uploads detailed results as artifacts

## Configuration

The `.cargo/mutants.toml` file excludes certain problematic mutations:
- Functions returning `Default::default()` in token generation
- Match arm deletions in type checking
- Arithmetic operator swaps in bit calculation code

## Improving Coverage

When adding new features:
1. Write integration tests that exercise the generated code
2. Add unit tests for pure functions (type checking, validation)
3. Focus on testing behavior, not implementation details

## Running Locally

```bash
# Quick run with timeout
cargo mutants --timeout 30

# Full run with detailed output
cargo mutants -vV

# Test specific file
cargo mutants --file bebytes_derive/src/structs.rs
```