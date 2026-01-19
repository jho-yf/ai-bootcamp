---
description: Perform deep code review for Rust and TypeScript codebases, analyzing architecture, design patterns, and code quality principles.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Context

This command performs a comprehensive, systematic code review for Rust and TypeScript codebases. It analyzes code across multiple dimensions:

1. **Architecture & Design** - Overall structure, patterns, and organization
2. **Code Principles** - DRY, YAGNI, SOLID, KISS
3. **Code Complexity** - Function size, parameter count, cyclomatic complexity
4. **Language-Specific Best Practices** - Rust ownership/borrowing, TypeScript typing
5. **Security & Performance** - Common vulnerabilities and optimization opportunities

## Outline

### Phase 1: Scope Analysis

1. **Parse review scope from user input**:
   - If `$ARGUMENTS` is empty: Review all modified files in the current git branch
   - If `$ARGUMENTS` contains file paths: Review only specified files
   - If `$ARGUMENTS` contains patterns: Use glob/grep to find matching files

2. **Identify codebase structure**:
   ```bash
   # Detect project type and structure
   find . -name "Cargo.toml" -o -name "package.json" -o -name "tsconfig.json" | head -20
   ```

3. **Classify files by language**:
   - Rust files: `*.rs`, `Cargo.toml`
   - TypeScript files: `*.ts`, `*.tsx`, `package.json`, `tsconfig.json`
   - Create separate review queues for each language

### Phase 2: Architecture & Design Review

#### A. Rust Architecture Review

For each Rust file/module, evaluate:

1. **Module Organization**:
   - Are modules logically organized with clear responsibilities?
   - Does the module tree follow the principle of least privilege?
   - Are public APIs carefully curated (re-exports, pub use)?
   - Check: `mod.rs` files, `lib.rs` structure, visibility modifiers

2. **Ownership & Borrowing Patterns**:
   - Are ownership transfers clear and intentional?
   - Is borrowing used appropriately (references vs. moved values)?
   - Are lifetimes annotated where needed without being excessive?
   - Check for: unnecessary clones, missing lifetimes, lifetime elision opportunities

3. **Type System Design**:
   - Are types used to encode invariants (newtype pattern, enum variants)?
   - Are error types comprehensive and informative?
   - Is the type system leveraged to prevent invalid states?
   - Check: custom types, Result types, enum design, trait bounds

4. **Trait Design & Usage**:
   - Are traits used for appropriate abstraction levels?
   - Do traits follow object safety guidelines where needed?
   - Are trait bounds clear and necessary?
   - Check: trait definitions, impl blocks, generic constraints

5. **Error Handling**:
   - Are errors handled explicitly with Result and Option?
   - Is error context preserved with `thiserror` or similar?
   - Are recoverable vs. unrecoverable errors distinguished?
   - Check: unwrap/expect usage, error propagation, custom error types

6. **Concurrency & Async**:
   - Are async/await used appropriately?
   - Is thread safety properly considered (Send, Sync)?
   - Are channels/mutexes used correctly?
   - Check: async functions, Arc/Mutex usage, spawn/.join

#### B. TypeScript Architecture Review

For each TypeScript file, evaluate:

1. **Type System Design**:
   - Are types explicitly defined (no implicit any)?
   - Are interfaces and types used appropriately?
   - Is strict mode enabled and utilized?
   - Check: type annotations, interface definitions, type inference

2. **Module System & Imports**:
   - Are imports organized and barrel exports used?
   - Is there circular dependency?
   - Are relative vs. absolute imports consistent?
   - Check: import statements, exports, module structure

3. **Component/Presentation Architecture**:
   - Are components/presentations properly separated?
   - Is prop drilling avoided with composition/context?
   - Are side effects isolated?
   - Check: component structure, hooks usage, context providers

4. **API Design**:
   - Are function signatures clear and type-safe?
   - Are generic types used appropriately?
   - Is return type inference avoided for public APIs?
   - Check: function signatures, type parameters, return types

### Phase 3: Code Principles Review

Evaluate code against fundamental principles:

#### A. DRY (Don't Repeat Yourself)

1. **Check for code duplication**:
   - Similar functions/methods that could be abstracted
   - Repeated logic patterns that could be extracted
   - Duplicate data structures or configurations

2. **Identify violations**:
   ```bash
   # Use structural similarity detection (tools like jscpd, or manual inspection)
   grep -rn "similar patterns" src/
   ```

3. **Recommend refactoring**:
   - Extract common logic into functions/methods
   - Use macros (Rust) or generics/templates appropriately
   - Create shared utilities/constants

#### B. YAGNI (You Aren't Gonna Need It)

1. **Check for unnecessary complexity**:
   - Unused functions, types, or modules
   - Over-engineered abstractions
   - Premature optimization
   - "Just in case" code

2. **Identify violations**:
   - Dead code not referenced anywhere
   - Complex patterns for simple problems
   - Unused dependencies

3. **Recommend simplification**:
   - Remove dead code
   - Simplify over-engineered solutions
   - Defer features until actually needed

#### C. SOLID Principles

1. **Single Responsibility Principle (SRP)**:
   - Does each module/function/class have one reason to change?
   - Are concerns properly separated?
   - Check: functions doing multiple unrelated things

2. **Open/Closed Principle (OCP)**:
   - Is code open for extension but closed for modification?
   - Are abstractions used for extensibility?
   - Check: traits/interfaces, polymorphism

3. **Liskov Substitution Principle (LSP)**:
   - Are subtypes properly substitutable for base types?
   - Do trait implementations honor contracts?
   - Check: trait implementations, class inheritance

4. **Interface Segregation Principle (ISP)**:
   - Are interfaces/trait focused and minimal?
   - Are clients not forced to depend on unused methods?
   - Check: trait size, interface composition

5. **Dependency Inversion Principle (DIP)**:
   - Do high-level modules not depend on low-level modules?
   - Are dependencies inverted through abstractions?
   - Check: dependency injection, trait objects

#### D. KISS (Keep It Simple, Stupid)

1. **Check for unnecessary complexity**:
   - Clever code that's hard to understand
   - Overly complex control flow
   - Nested conditionals and loops

2. **Identify violations**:
   - Functions with high cyclomatic complexity
   - Deep nesting (more than 3-4 levels)
   - Convoluted logic that could be simplified

3. **Recommend simplification**:
   - Early returns to reduce nesting
   - Extract complex conditions into named variables
   - Break complex functions into smaller ones

### Phase 4: Code Complexity Review

#### A. Function Size Analysis

For each function/method:

1. **Count lines of code**:
   - Maximum: 150 lines (strict threshold)
   - Warning: 100 lines
   - Ideal: 50 lines or less

2. **Check violations**:
   ```bash
   # For Rust
   grep -rn "^fn " src/ --include="*.rs" | while read line; do
     # Extract function and analyze length
   done

   # For TypeScript
   grep -rn "function\|=>\|class" src/ --include="*.ts" --include="*.tsx"
   ```

3. **Report violations** with:
   - File path and line number
   - Function name
   - Actual line count
   - Recommendation for splitting

#### B. Parameter Count Analysis

For each function/method:

1. **Count parameters**:
   - Maximum: 7 parameters (strict threshold)
   - Warning: 5 parameters
   - Ideal: 3 parameters or less

2. **Check violations**:
   - Functions with >7 parameters
   - Constructor functions with many parameters

3. **Recommend refactoring**:
   - **Use Builder pattern** (especially for Rust)
   - Group related parameters into structs/objects
   - Use options/configuration objects
   - Consider method chaining

#### C. Builder Pattern Evaluation

When many parameters are needed, evaluate Builder pattern usage:

**Rust Builder Pattern Checklist**:
- Builder derives `Debug` and `Clone` where appropriate
- Required parameters are enforced at compile time (build method or new)
- Optional parameters use `Option<T>` or sensible defaults
- Builder methods return `Self` for chaining
- `build()` method returns `Result<T, E>` for validation
- Example structure:
  ```rust
  pub struct Builder {
      required: RequiredType,
      optional1: Option<Type1>,
      optional2: Type2,  // with Default
  }

  impl Builder {
      pub fn new(required: RequiredType) -> Self { /* ... */ }
      pub fn optional1(mut self, val: Type1) -> Self { /* ... */ }
      pub fn build(self) -> Result<Target, Error> { /* ... */ }
  }
  ```

**TypeScript Builder Pattern Checklist**:
- Builder class provides fluent interface
- Required parameters enforced in constructor
- Optional parameters have defaults or setters
- `build()` method validates and returns constructed object
- Example structure:
  ```typescript
  class Builder {
    private required: RequiredType;
    private optional1?: Type1;

    constructor(required: RequiredType) { /* ... */ }
    withOptional1(val: Type1): this { /* ... */; return this; }
    build(): Target { /* ... */ }
  }
  ```

### Phase 5: Language-Specific Best Practices

#### A. Rust Best Practices

1. **Idiomatic Rust Patterns**:
   - Use `Iterator` methods instead of manual loops where appropriate
   - Prefer `match` over nested `if-else`
   - Use `Option` and `Result` combinators (`map`, `and_then`, `unwrap_or`)
   - Leverage `?` operator for error propagation

2. **Performance Considerations**:
   - Avoid unnecessary allocations and clones
   - Use `Cow<[T]>` for conditional ownership
   - Prefer `&str` over `String` for function parameters
   - Use `vec![]` with capacity pre-allocation
   - Check for: `to_string()`, `clone()` in hot paths

3. **Memory Safety**:
   - No unsafe code without good reason and documentation
   - Interior mutability used appropriately (`Cell`, `RefCell`, `Mutex`)
   - Pin and futures correctly handled

4. **Testing**:
   - Unit tests present in module files (`#[cfg(test)]`)
   - Integration tests in `tests/` directory
   - Property-based tests with `proptest` for complex logic

5. **Documentation**:
   - Public APIs documented with `///` or `//!`
   - Examples in doc comments (`/// ````)
   - Panic conditions documented

#### B. TypeScript Best Practices

1. **Type Safety**:
   - Enable `strict` mode in `tsconfig.json`
   - Avoid `any` type - use `unknown` for truly dynamic data
   - Prefer `interface` for object shapes, `type` for unions/unions
   - Use discriminated unions instead of loose types
   - Use `as const` for readonly literals

2. **Modern TypeScript**:
   - Use optional chaining `?.` and nullish coalescing `??`
   - Leverage template literal types for type-level string manipulation
   - Use utility types (`Partial<T>`, `Required<T>`, `Pick<T>`)
   - Consider `satisfies` operator for type constraints

3. **React/Next.js Specific** (if applicable):
   - Functional components with hooks
   - Proper dependency arrays in `useEffect`, `useMemo`
   - Custom hooks for reusable logic
   - Avoid prop drilling with context

4. **Error Handling**:
   - Explicit error types, not catching `any`
   - Distinguish between expected and unexpected errors
   - Proper error logging with context

5. **Testing**:
   - Unit tests for pure functions
   - Integration tests for components/modules
   - Type-level testing with `tsd`

### Phase 6: Security & Performance Review

#### A. Security Review

1. **Common Rust Security Issues**:
   - Unchecked `unwrap()`/`expect()` that could panic
   - Integer overflow/underflow (in debug builds)
   - Unsafe blocks without safety justification
   - Unvalidated user input
   - Path traversal vulnerabilities
   - SQL/Command injection

2. **Common TypeScript Security Issues**:
   - XSS vulnerabilities (unescaped HTML)
   - Prototype pollution
   - Unsafe eval/Function
   - Insecure random number generation
   - Missing input validation
   - Sensitive data in logs/error messages

#### B. Performance Review

1. **Rust Performance**:
   - Unnecessary allocations in loops
   - Inefficient data structures (Vec vs HashMap)
   - Missing `#[inline]` for small, hot functions
   - Excessive cloning
   - Blocking async code

2. **TypeScript Performance**:
   - Unnecessary re-renders (React)
   - Large bundle sizes
   - Inefficient algorithms (O(n²) where O(n) possible)
   - Memory leaks (unclosed subscriptions, event listeners)

### Phase 7: Reporting

Generate a comprehensive review report in markdown format with the following structure:

```markdown
# Code Review Report

**Date**: [CURRENT DATE]
**Branch**: [CURRENT BRANCH]
**Files Reviewed**: [COUNT]
**Languages**: Rust | TypeScript | Both

## Summary

- **Total Issues Found**: [X Critical, Y Major, Z Minor]
- **Overall Score**: [A/B/C/D/F]
- **Review Status**: [PASSED/NEEDS ATTENTION/FAILED]

---

## Critical Issues (Must Fix)

[Issues that are blockers: security vulnerabilities, panics, data loss risks]

## Major Issues (Should Fix)

[Issues that significantly impact code quality, maintainability, or performance]

## Minor Issues (Nice to Fix)

[Style issues, minor improvements, optimizations]

---

## Detailed Findings

### Architecture & Design

#### Rust Architecture
- [Findings about module organization, ownership patterns, type design]

#### TypeScript Architecture
- [Findings about type system, module structure, component design]

### Code Principles Violations

#### DRY Violations
| File | Lines | Issue | Recommendation |
|------|-------|-------|----------------|
| ...  | ...   | ...   | ...            |

#### YAGNI Violations
[Dead code, over-engineering found]

#### SOLID Violations
[Specific principle violations with examples]

### Code Complexity Issues

#### Function Size Violations (>150 lines)
| File | Function | Lines | Recommendation |
|------|----------|-------|----------------|
| ...  | ...      | ...   | ...            |

#### Parameter Count Violations (>7 parameters)
| File | Function | Parameters | Recommendation |
|------|----------|------------|----------------|
| ...  | ...      | ...        | ...            |

**Builder Pattern Recommendations**:
[Specific places where Builder pattern should be applied]

### Language-Specific Issues

#### Rust Specific
- [Ownership/borrowing issues]
- [Error handling improvements]
- [Unsafe code review]
- [Performance optimizations]

#### TypeScript Specific
- [Type safety issues]
- [Modern syntax opportunities]
- [React/hook issues if applicable]

### Security & Performance

#### Security Issues
| Severity | File | Issue | Fix |
|----------|------|-------|-----|
| ...      | ...  | ...   | ... |

#### Performance Issues
| Impact | File | Issue | Optimization |
|--------|------|-------|--------------|
| ...    | ...  | ...   | ...          |

---

## Recommendations

### Immediate Actions
1. [Critical issues to fix now]
2. [Blocking changes for merge]

### Short-term Improvements
1. [Refactoring priorities]
2. [Technical debt to address]

### Long-term Considerations
1. [Architectural improvements]
2. [Better patterns to adopt]

---

## Positive Findings

[What the codebase does well - acknowledge good practices seen]

## Metrics

- **Average Function Size**: [X] lines
- **Largest Function**: [X] lines in `[file:function]`
- **Average Parameter Count**: [X]
- **Code Duplication**: [X]% estimated
- **Test Coverage**: [if available]
- **Unsafe Code Blocks**: [X] (Rust)
- **Any Types**: [X] (TypeScript)
```

### Phase 8: Execution Flow

1. **Parse input** and determine scope
2. **Analyze codebase** structure and classify files
3. **Run reviews** for each category:
   - Architecture & Design
   - Code Principles (DRY, YAGNI, SOLID, KISS)
   - Complexity (size, parameters)
   - Language-specific best practices
   - Security & Performance
4. **Aggregate findings** and categorize by severity
5. **Generate report** as markdown file
6. **Output summary** to user with report location

### Phase 9: Severity Classification

- **Critical**: Security vulnerabilities, potential panics (Rust), data loss risks
- **Major**: Breaking code principles, high complexity, performance issues
- **Minor**: Style issues, minor improvements, missing optimizations

## General Guidelines

### Review Philosophy

- **Constructive, not critical**: Focus on improvement, not criticism
- **Explain the "why"**: Don't just point out problems, explain the impact
- **Provide examples**: Show how to fix issues, not just what's wrong
- **Acknowledge good code**: Highlight what's done well
- **Be pragmatic**: Perfect is the enemy of good - prioritize impactful changes

### When to Use This Command

Use `/code-review`:

- Before merging a PR
- After completing a feature implementation
- During refactoring efforts
- As part of regular code quality maintenance
- When onboarding to a new codebase

### Output Format

- Report saved to: `.claude/reviews/code-review-<timestamp>.md`
- Summary printed to console
- Categorized findings with file:line references
- Actionable recommendations prioritized by severity
