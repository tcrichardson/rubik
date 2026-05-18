# Java Language Support Design

**Date:** 2026-05-18
**Status:** Approved

## Overview

Add Java language support to lede, enabling analysis of `.java` source files for cyclomatic complexity, nesting depth, and Halstead metrics.

## Architecture

Follow the existing language analyzer pattern. Create `src/language/java.rs` implementing `LanguageAnalyzer` trait, register it in `src/language/mod.rs` and `src/analyzer.rs`.

## File Changes

### 1. `Cargo.toml`

Add dependency:
```toml
tree-sitter-java = "0.23"
```

### 2. `src/language/java.rs` (new)

#### Function Kinds
- `method_declaration` — standard method declarations
- `constructor_declaration` — class constructors
- `compact_constructor_declaration` — Java record constructors
- `lambda_expression` — lambda expressions (closure)

#### Closure Kinds
- `lambda_expression`

#### Decision Kinds
- `if_statement` — if statements
- `for_statement` — traditional for loops
- `enhanced_for_statement` — for-each loops
- `while_statement` — while loops
- `do_statement` — do-while loops
- `switch_expression` — switch statements/expressions
- `switch_label` — case and default labels
- `catch_clause` — catch blocks
- `ternary_expression` — ternary operator

#### Operator Kinds (for Halstead metrics)
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!`
- Assignment: `=`, `+=`, `-=`, `*=`, `/=`, `%=`
- Bitwise: `&`, `|`, `^`, `<<`, `>>`, `>>>`, `~`
- Access: `.`, `::`
- Control: `return_statement`, `break_statement`, `continue_statement`, `throw_statement`, `yield_statement`
- Type: `instanceof_expression`

#### Operand Kinds
- `identifier`
- `decimal_integer_literal`, `hex_integer_literal`, `octal_integer_literal`, `binary_integer_literal`
- `decimal_floating_point_literal`, `hex_floating_point_literal`
- `string_literal`, `character_literal`
- `true`, `false`, `null_literal`
- `this`, `super`

#### Name Extraction
- For `method_declaration`: extract from `identifier` child
- For `constructor_declaration`: extract from `identifier` child
- For `compact_constructor_declaration`: extract from `identifier` child
- For `lambda_expression`: return `<lambda>@line N`

### 3. `src/language/mod.rs`

Add: `pub mod java;`

### 4. `src/analyzer.rs`

Import `JavaAnalyzer` and add to `ANALYZERS` array.

### 5. `README.md`

Update:
- Features list: add "Java"
- Supported File Extensions table: add `.java`
- Decision Point table: add Java column

## Testing

Unit tests in `java.rs` covering:
1. Simple method with if statement
2. If-else chain
3. Traditional for loop
4. Enhanced for-each loop
5. While and do-while loops
6. Switch with multiple cases
7. Try/catch/finally
8. Lambda included (with `include_closures=true`)
9. Lambda excluded by default
10. Ternary expression
11. Boolean operators (`&&`, `||`)
12. Constructor name extraction
13. Parse error handling
14. `can_analyze` for `.java` extension

## Java-Specific Decisions

- **Constructors** count as functions, consistent with other analyzers
- **Compact constructors** (Java records) are included as functions
- **Lambda expressions** are treated as closures, excluded by default
- **Switch labels** each `case` and `default` counts as one decision point
- **File extension**: `.java` only
