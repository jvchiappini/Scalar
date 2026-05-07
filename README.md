# Scalar & Ferrous Engine 📐⚙️

**Scalar** is a high-performance visual computing platform and motion graphics suite, purpose-built for mathematical animation, procedural design, and technical visualization. 

At the very core of this ecosystem lies **`ferrous_engine`**—a highly optimized, general-purpose 2D/3D rendering SDK written entirely in Rust. While currently powering the Scalar DSL, `ferrous_engine` is designed from the ground up to be a universal graphics library capable of driving video games, cross-platform app builders, NLE video editors, and UI framework backends (such as `egui` integrations).

---

## 🏛️ Architecture Philosophy

The development of this repository is governed by four non-negotiable core tenets:

1. **Strict Modularity (Anti-God Files):** 
   Codebases rot when files grow too large. We strictly enforce the Single Responsibility Principle (SRP). Monolithic files ("God Files") handling multiple domains are strictly prohibited. If a module grows beyond a highly cohesive scope, it *must* be split into specialized submodules.
2. **Impeccable Code Documentation (Docstrings):**
   Code without context is technical debt. **Every** public `struct`, `enum`, `trait`, and `fn` must be accompanied by flawless, comprehensive Rustdoc comments (`///`). Where applicable, docstrings must include minimal usage examples. Undocumented public APIs will be rejected.
3. **Absolute Determinism:** 
   The graphics engine (`ferrous_engine`) must guarantee that frame `N` always renders exactly the same way, isolating the rendering loop from real-world CPU wall-clock time, while maintaining the ability to process real-time delta loops for video game integrations.
4. **Unidirectional Dependency Flow & Engine Agnosticism:**
   The layers of the ecosystem are strictly isolated. **`ferrous_engine` must never contain logic specific to Scalar, math animations, or timelines.** It is an agnostic library. All cross-domain communication occurs strictly through the FFI/Binding layer (`scalar_bridge`).

---

## 📦 Workspace Topology

The ecosystem is divided into the following highly specialized crates:

- **`ferrous_engine`**: The Universal 2D/3D rendering SDK (WGPU, internal ECS, PBR, Shadows, Vector Tessellation). 
  > *Strategic Vision: This crate is engineered to be published as a standalone library for the wider Rust ecosystem. It is intended to power future projects including game engines, UI renderers, and app creators. It must remain pure and agnostic.*
- **`scalar_lang`**: The core language runtime. Includes the Lexer, combinatory Parser, AST, and the dynamic interpreter.
- **`scalar_bridge`**: The FFI/Binding layer. Maps the memory and native functions of the `scalar_lang` virtual machine to the ECS of `ferrous_engine`.
- **`scalar_cli`**: The Command-Line Interface. Instantiates the ecosystem in *Headless* mode, orchestrates the timeline, and pipes VRAM frames directly to FFmpeg via IPC (`stdin`) to compile `.mp4` video files.

---

## 📖 The Internal Wiki (`/wiki`)

The `/wiki` directory is the living, hierarchical wiki and the definitive source of truth for this project. It is **not** a flat collection of files; it is a meticulously structured library designed to scale.

Contributors (human or automated agents) **must** consult this directory before proposing architectural changes, and are required to update it in the same commit if APIs or grammars are modified. 

To maintain order as the project grows, contributors are fully authorized and encouraged to:
- **Create subdirectories** to group related concepts (e.g., `/wiki/engine/`, `/wiki/lang/`).
- **Create index files** (e.g., `_index.md` or `README.md` inside folders) to map out sub-domains.
- **Develop utility scripts** (e.g., Python tools like `search_docs.py` or `validate_ast_docs.py`) within the `/wiki/tools/` directory to help traverse, index, or maintain the wiki's integrity.

**Core Directory Structure Example:**
- `/wiki/engine/architecture.md` - Wgpu pipeline, shaders, and ECS lifecycle.
- `/wiki/lang/grammar_spec.md` - AST definitions, grammar rules, and tokens.
- `/wiki/bindings/memory_map.md` - FFI mappings between the virtual machine and the engine.
- `/wiki/roadmap/_index.md` - Short, medium, and long-term milestones.

---

## 🛠️ Contribution Standards

All Pull Requests and automated code generations must strictly adhere to the following rules:
1. **Context Separation:** Parsing logic cannot interact the ECS; graphics logic must not parse text files.
2. **No Hacks or Bypasses:** Temporary workarounds are unacceptable. If the Rust type system presents a challenge (e.g., lifetimes, shared mutability), it must be solved architecturally (using patterns like `Rc<RefCell<T>>`, channels, or ECS restructuring).
3. **Strict Typing:** Avoid "stringly-typed" logic or loose primitives. Use strictly typed Enums and Structs to represent states and configurations.