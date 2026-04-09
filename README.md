# Anantham Game Engine (AGE)

## 1. Core Identity and Philosophy

- **Concept:** An "opinionated game with an absurdly extensive API" (the
  Half-Life/Roblox model). It is not a blank-slate engine. The base game
  provides unbreakable systemic primitives, a distinct visual identity, and
  establishes the culture, while delegating infinite mechanic expansion to a
  heavily integrated modding ecosystem.
- **Visual Identity:** A macro-micro voxel system. The base building unit is a
  1-meter grid to maintain structural identity and accessibility (low floor),
  but blocks can be sub-divided into micro-voxel grids (e.g., 16^3 or 32^3) for
  high-fidelity carving and scripting (infinite ceiling).
- **Scale and Immersion:** Massive render distances with continuous horizons.
  High ambition utilizing modern hardware over legacy low-spec compatibility.

## 2. Technical Architecture & Ownership

- **Language:** Rust.
- **The Core Pattern (App & Plugins):** A strict separation between the
  Immutable Core and Official/Community Plugins.
    - **Main World:** Owns gameplay state, the ECS database (Bevy/hecs style),
      the Wasm runtime state, and macro-grids. Runs on a fixed deterministic
      timestep.
    - **Extraction Phase:** A brief, synchronous bridge. The Main World securely
      copies data (transforms, grid diffs, animation states) to the Render
      World.
    - **Render World:** Strictly owns the Vulkan 1.4 `ash::Device`, GPU
      allocators, Swapchain, and SSBOs. Variables cannot be accidentally mutated
      by game logic.
- **Rendering Pipeline:** - No discrete LODs. Utilizes hardware tessellation,
  mesh shaders, and continuous edge-aware decimation.
    - Sparse Voxel Octrees (SVO) and Greedy Meshing for micro-voxels.
    - Voxel Global Illumination (VXGI) and Virtual Shadow Maps (VSM).
    - **The Render Graph:** The renderer is locked in the core to preserve
      visual identity, but exposes an Extensible Render Graph. Native mods
      cannot overwrite the renderer via hacky injections; they inject strictly
      scheduled `RenderPassNodes` (e.g., custom post-processing or compute
      shaders).

## 3. API-First Design (Eating Our Own Dogfood)

To guarantee ultimate compatibility, the vanilla base game is simply "Mod #1".
It relies entirely on the API exposed to the community.

- **Facade Physics:** The core API defines raw data components (`RigidBody`,
  `Collider`). The official Physics Crate implements the logic. If a modder
  wants realistic physics, they fork the Physics Crate. Other mods (e.g., a
  custom crossbow mod) automatically inherit the new physics because they share
  the same API components.
- **Relational Inventory:** Inventories are not fixed OOP arrays. They are
  relational databases in the ECS. A modder can add a "Bauble" slot by simply
  spawning a new Entity tagged as a `SlotType::Modded`, making the UI infinitely
  and safely expandable without conflicts.

## 4. The Two-Tiered Modding Architecture

- **Tier 1: Native Rust Crates (Compile-Time Modding)**
    - Used for deep engine alterations, custom Render Graph compute shaders, and
      core system replacements.
    - **Distribution:** Distributed via a custom Cargo registry. The Hub
      launcher utilizes a hermetic (bundled) Rust toolchain to silently run
      `cargo build --release` on the user's machine.
    - **Benefit:** Zero FFI overhead, Link-Time Optimization (LTO), and
      compile-time conflict resolution (Cargo handles dependencies, preventing
      runtime crashes). Requires open-source auditing.
- **Tier 2: WebAssembly (Wasm) Plugins (Content & Logic)**
    - Used for adding items, entities, UI, and event scripting.
    - **Polyglot & Sandboxed:** Defined via `.wit` bindings. Modders can write
      in Rust, C, or Python. Executes in a completely secure, natively sandboxed
      Wasm runtime (Wasmtime).
    - **The Command Queue:** Wasm scripts _cannot_ directly mutate the ECS. They
      push requests to an API Command Queue (e.g.,
      `[Despawn(Entity), SpawnExplosion(X,Y,Z)]`), which the native Rust core
      processes synchronously, ensuring memory safety.

## 5. Business and Distribution Model (Open Hook + Closed Content)

- **The Engine (Free/Open):** The raw Rust core and native Tier 1 API hooks are
  open-source (under a strong copyleft license like GPL/AGPL). Anyone can
  compile and play offline for free.
- **The Hub (Paid/Proprietary):** The monetization vector. A proprietary
  launcher that requires a paid authentication token. It functions as the game's
  main menu, handling seamless background compilation, 1-click mod installs,
  server browsing, and cloud saves.
- **Proprietary Modding Support:** A studio can write an open-source Native Rust
  Crate for custom engine hooks, but keep their massive Wasm content package
  (quests, AI, dialogue) closed-source and monetized on the Hub.
- **Mod Creator Monetization:**
    - _Playtime-Weighted Pool:_ Revenue from hub sales is pooled and distributed
      monthly based on user playtime. Incentivizes creators to make their mods
      free to maximize engagement and playtime share.
    - _Direct Sales:_ "Honesty DRM." Creators can sell premium Wasm plugins
      outright via the hub for a standard platform cut. No subscription models
      allowed.

## 6. Legal and Security Framework

- **DMCA Safe Harbor:** The platform operates as a service provider.
- **Escrow and Clawbacks:** Revenue from the playtime pool is held in a 60-day
  escrow. In the event of a DMCA takedown or malicious code flag, the mod is
  delisted, and pending funds are redistributed.
- **Security Scans:** The custom Cargo registry runs automated CI/CD pipelines
  (`cargo audit`, unsafe AST scanning) for Native Crates. Wasm plugins are
  inherently protected by the runtime sandbox.
- **Indemnification:** Creators sign agreements legally assuming liability for
  their uploads.

## 7. Immediate Milestone: Strong Foundation and MVP Demo

- **Goal:** A 5-minute technical demo to secure funding and community/creator
  buy-in via a closed Mod Jam.
- **Focus:** Establish the Wasm boundary, the Render World/Main World extraction
  phase, and the base Voxel/VXGI pipeline.
- **Sequence:**
    1. _Wasm Command Queue:_ Implement the `hecs` boundary and test live-loaded
       `.wasm` logic injection.
    2. _Macro:_ Procedural heightmap rendering massive distant horizons via the
       Vulkan 1.4 Render Graph.
    3. _Transition:_ Fly down to the surface, demonstrating continuous LOD.
    4. _Micro:_ Player chisels a 1-meter stone block into a micro-voxel lantern
       housing.
    5. _Logic/Lighting:_ Player places a glowing crystal inside. The crystal's
       behavior is driven by the Wasm Command Queue, casting dynamic VXGI light
       inside the carved geometry.
