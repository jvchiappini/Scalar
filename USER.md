# User Profile

## Preferences
- Language: Spanish (communicates in Spanish)
- Code style: Clean Rust, explicit types, prefers simple solutions
- Workflow: Iterative — reports issues after testing, expects quick fixes
- Communication: Direct, points out problems concisely

## Working Patterns
- Tests features by downloading the code and running on **Windows PC** — never run the CLI binary in the development workspace (it has no GPU/display server)
- Reports visual issues (rendering artifacts, animation glitches) with specific descriptions
- Prefers animated effects to have smooth easing curves
- Likes visual polish (bounce, elastic easings)

## Current Priorities
- Building a pro-level animation system that surpasses Manim
- **Implement user-defined functions (`fn`)** — the foundation for abstraction and reuse
- **Implement `Wait()` + `Play()`** — professional choreography sequencing
- Adding missing objects: Arrow, Dot, NumberPlane, Brace
- Fixing `Grow` animation (currently a no-op)
- Polish morph system: per-glyph text morph working, test on Windows

## Project Values
- Code must compile cleanly (no warnings from our crates)
- Wiki must be kept in sync with code changes
- Tests must pass before considering work complete
