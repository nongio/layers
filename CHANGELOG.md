# Changelog

All notable changes to this project will be documented in this file.

## [1.8.0] - 2026-04-04

### 🚀 Features

- Add live animation inspector to debugger
- Skip painting nodes outside the damage region
- Expose scene module and add find_layer_by_key API
- Keyframe animation timing (#16)

### ⚙️ Miscellaneous Tasks

- Upgrade skia-safe from 0.88 to 0.93
- Cargo fmt
- Bump version to 1.8.0
- Update changelog for 1.8.0

## [1.7.0] - 2026-03-19

### 🚀 Features

- Blur pipeline to work on scaled backdrop (#15)

### ⚙️ Miscellaneous Tasks

- Version bump

## [1.5.0] - 2026-03-09

### 🚀 Features

- Add IntoFuture for AnimationRef (#13)

### ⚙️ Miscellaneous Tasks

- *(version)* 1.5.0

## [1.4.3] - 2026-03-06

### 🐛 Bug Fixes

- Clean up stale layer handles on removal (#12)

## [1.4.2] - 2026-03-04

### 🐛 Bug Fixes

- Safe arena node access and pending transactions count API (#11)

## [1.4.0] - 2026-02-24

### ⚙️ Miscellaneous Tasks

- Bump v

## [1.3.0] - 2026-01-25

### ⚙️ Miscellaneous Tasks

- *(ci)* Add ci permission to write to ghpages
- *(agent)* Instruction for copilot reviews

## [1.2.0] - 2026-01-20

### 🚀 Features

- Ui update layers_inspector
- Add fractal noise overlay for BackgroundBlur blend mode
- Animation callbacks
- Adjust background blur noise alpha to 70

### 🐛 Bug Fixes

- Node removed checks + expose Spring timing
- Implement backdrop blur region bubbling for image-cached layers

### 📚 Documentation

- Update changelog for backdrop blur damage tracking

### ⚙️ Miscellaneous Tasks

- Bump version to 1.1.0
- Bump version to 1.2.0

## [1.0.0] - 2025-12-08

### 🚀 Features

- Layer content clip to bounds
- Layer clip children
- Debugger_viz
- Change anchor point preserving position
- LayerTree color filter
- View hooks
- Expose visibility in render_layer
- Layers_inspector 1.0
- Add hit_test_node_list cache for pointer hit-testing

### 🐛 Bug Fixes

- Debugger assets path
- Increase spring animation tolerance
- Engine.mark_for_delete
- Engine.trigger_callback deadlock
- Buildlayertree get or create child
- Engine update
- Update node
- Layer caching logic
- Rendering issues
- Noop change id conflict
- Damage tracking logic
- Engine update logic
- Hit-tracking
- Rendering problem with transparent parent nodes
- Removing subtrees
- Filters not working
- Propagate parent transforms to children
- Prevent crashes
- Prevent deadlocks
- Clean handlers on remove layer
- Prevent infinite recursion in as_content for nested replicated layers
- Re-enable content_cache to prevent recursion in draw_layer
- Color / image filters model changes ids
- Bounds with children calculations
- Update cbindgen and improve build error handling

### 🚜 Refactor

- Layer pointer event and propagation + examples

### 📚 Documentation

- RenderLayer attributes
- Add development workflow instructions
- Require doc comments for new methods
- Add damage tracking docs
- Update
- Layer followers
- Add documentation pages
- Mark damage tracking for mirrored layers as fixed

### 🧪 Testing

- Damage ut update
- Add tests for hidden parent pointer events, damage tracking, and replicate_node
- Update image_cache tests
- Ignore draw_multiple_children test in headless environments

### ⚙️ Miscellaneous Tasks

- [**breaking**] Rename library to lay-rs
- [**breaking**] Layer content caching api + export layer as content
- [**breaking**] Storage using tokio:rwlock
- Bump rust-skia
- C api refactor
- Expose LayerTree anchor point
- [**breaking**] Replicate layer api
- Improve error tracing
- [**breaking**] Svg rendering using resvg
- Refactor shared mutable engines
- Use nightly Rust for cargo-cache compatibility
- Fix nightly Rust version format
- Use nightly Rust for benchmark workflow
- Add test job to continuous integration
- Use nightly Rust for test job

### Api

- [**breaking**] Rename engine add_layer api

### Example

- Update hello-content

### Examples

- Cleanup
- Update animations

### Fmt

- Clippy

### Renderlayer

- Disable layer content caching

### Restore

- Layer.set_hidden

## [0.5.0] - 2024-10-27

### 🚀 Features

- Value change listener

### ⚙️ Miscellaneous Tasks

- Update cliff config for version bump

## [0.4.0] - 2024-10-27

### 🚀 Features

- Layer color filter
- Enable debugger on 0.0.0.0
- Spring animations
- Value change listener
- Layer content clip to bounds
- Layer clip children
- Layertree fmt::Debug
- Layer color filter
- Enable debugger on 0.0.0.0
- Spring animations

### 🐛 Bug Fixes

- [**breaking**] New layer apis and fix offscreen drawing
- Debugger assets path
- Increase spring animation tolerance
- Engine.mark_for_delete
- Engine.trigger_callback deadlock
- Buildlayertree get or create child
- [**breaking**] New layer apis and fix offscreen drawing

### 📚 Documentation

- Update documentation and imports
- Documentation update logo link
- Update documentation and imports
- Documentation update logo link

### 🧪 Testing

- Transaction_handlers rework

### ⚙️ Miscellaneous Tasks

- Add permission to publish pages
- Contents write
- Update cliff config for version bump
- [**breaking**] Rename library to lay-rs
- [**breaking**] Layer content caching api + export layer as content
- [**breaking**] Storage using tokio:rwlock
- Bump rust-skia
- C api refactor
- Expose LayerTree anchor point
- [**breaking**] Replicate layer api
- Improve error tracing
- [**breaking**] Svg rendering using resvg
- [**breaking**] Refactor Engine to handle Arc references
- Bench results

### ◀️ Revert

- Tokio::rwlock

### Api

- [**breaking**] Rename engine add_layer api

### Bugfix

- Cargo test --doc
- Build_layer logic / append new layers
- Update_nodes stage

### Examples

- Rename hello-spring hello-views

### Fmt

- Clippy

### Restore

- Layer.set_hidden

### Update

- Examples

<!-- generated by git-cliff -->
