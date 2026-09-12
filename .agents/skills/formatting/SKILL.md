---
name: formatting
description: Apply this repository's formatting rules when writing, editing, or reviewing Rust code and tests.
---

# Formatting

Apply to code you add or change, including tests.

- Leave one blank line between a completed block and the next statement or block.
- Leave one blank line before `if` statements, except at the start of a block.
- Add a one line above a return, break, or continue statement - this includes return statements where the `return` keyword isn't used. Don't do this if the line above is an opening for a scope.
- Split long sequences of variable declarations into smaller groups by purpose, with one blank line between groups.
- Tests should be inline at the bottom of the relevant file inside a tests module.
- only use `super` at the top of tests modules.
- Don't inline imports, ensure all imports are at the top of the file
- Comments should also ideally be succinct, and around 1 - 3 sentences. Longer comments may be allowed in rare scenarios such as doc comments for complex systems.
- Ensure comments are only added to explain missing / important context.
- Ensure comments use proper grammer, capital letters, full stops, and oxford commas.
- Don't use comment dividers (e.g. `/////////////\nSection Name\n/////////////`).
- Names must be 3 or more characters, except for lifetimes, axis (x, y, z, w), cx (context), match statement else branch (`_`), and when we are referencing an argument from a third party function/closure.
- Names for indexes should be `idx` (you may add a prefix to differentiate - `apple_idx` and `pear_idx`).
- Don't name things as a singular underscore (`_foo` instead of `_`).
- Use guard clauses as much as possible. Aim for no more than 3 levels of nestedness.
- Abstract repeated logic into helper functions. DRY.
- Ensure tests are broad. Don't have lots of near-identical tests with minor differences. Test behaviour, not code.
