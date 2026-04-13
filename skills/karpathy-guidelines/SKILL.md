---
name: karpathy-guidelines
description: Use when writing, reviewing, or refactoring code to avoid overcomplication, surface assumptions, make surgical changes, and define verifiable success criteria.
---

# Karpathy Guidelines

Vendored from `forrestchang/andrej-karpathy-skills` and adapted as a project-local reference.

## 1. Think Before Coding

Do not assume. Do not hide confusion. Surface tradeoffs.

- State assumptions explicitly
- If multiple interpretations exist, present them
- If something is unclear, stop and ask
- If a simpler approach exists, say so

## 2. Simplicity First

Write the minimum code that solves the problem.

- No speculative features
- No abstractions for one-off code
- No configurability that was not requested
- No unnecessary error handling for impossible scenarios
- If the solution looks bloated, simplify it

## 3. Surgical Changes

Touch only what you must.

- Do not improve adjacent code unless required by the task
- Do not reformat unrelated code
- Match existing style and patterns where sensible
- Remove only the dead code your own change created

## 4. Goal-Driven Execution

Work toward verifiable outcomes.

- Translate tasks into explicit success criteria
- Prefer tests and objective checks over intuition
- For multi-step changes, define brief steps and verification for each

## Project Fit

These principles align with this repository's rules:

- hexagonal architecture
- small focused files
- minimal diffs
- strong verification before completion

## Source

Original upstream:

- `https://github.com/forrestchang/andrej-karpathy-skills`
