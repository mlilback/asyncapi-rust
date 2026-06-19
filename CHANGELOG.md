## [0.4.0] - 2026-06-19

### 🐛 Bug Fixes


- Replace HashMap with IndexMap for deterministic spec output (closes #9) ([#9](https://github.com/mlilback/asyncapi-rust/issues/9))


### 📚 Documentation


- Add 0.3.x → 0.4.0 migration guide and update version references




## [0.3.0] - 2026-06-19

### ⚙️ Miscellaneous


- Merge release/0.3.0 into main


### 📚 Documentation


- Update README for 0.3.0 and regenerate CHANGELOG


### 🧪 Testing


- Expand coverage to ~98% and add CI coverage reporting




## [0.3.0-beta.3] - 2026-06-19

### 🐛 Bug Fixes


- Use variant ident as message name; add message_name override; detect collisions (closes #8) (reported by @mlilback, [#8](https://github.com/mlilback/asyncapi-rust/issues/8))




## [0.3.0-beta.2] - 2026-06-19

### 🐛 Bug Fixes


- Hoist shared $defs into components.schemas; rewrite payload $refs (closes #7) ([#7](https://github.com/mlilback/asyncapi-rust/issues/7), [#6](https://github.com/mlilback/asyncapi-rust/issues/6))




## [0.3.0-beta.1] - 2026-06-19

### Style


- Apply rustfmt formatting to server_variables.rs


### ⚙️ Miscellaneous


- Fix rustfmt formatting issues

- Add reporter/contributor credit tokens to cliff changelog template


### ⚡ Features


- Add pre-commit hooks and enhanced release script


### 🐛 Bug Fixes


- Resolve clippy warnings in examples

- Bring Parameter object into AsyncAPI 3.0 compliance (closes #5) (reported by @pureions, [#5](https://github.com/mlilback/asyncapi-rust/issues/5))

- Handle open-ended JSON Schemas (e.g. serde_json::Value) without panic (closes #4) (reported by @csells, [#4](https://github.com/mlilback/asyncapi-rust/issues/4))

- Emit per-variant payload schema for tagged enums; embed $defs (closes #6, closes #2) ([#6](https://github.com/mlilback/asyncapi-rust/issues/6), [#2](https://github.com/mlilback/asyncapi-rust/issues/2), [#2](https://github.com/mlilback/asyncapi-rust/issues/2))


### 📚 Documentation


- Remove early development notice from README

- Update channel parameter docs to reflect AsyncAPI 3.0 Parameter fields




## [0.2.0] - 2025-11-07

### ⚙️ Miscellaneous


- Fix rustfmt formatting issues

- Add git-cliff for changelog generation


### ⚡ Features


- Add server variables and channel parameters support


### 🐛 Bug Fixes


- Support module paths in #[asyncapi_messages(...)]

- Support schemars 1.1 schema type arrays for optional fields


### 📚 Documentation


- Generate retroactive CHANGELOG.md for v0.1.0 and v0.1.1

- Add server_variables.rs example




## [0.1.1] - 2025-11-06

### ⚙️ Miscellaneous


- Add README to crates.io metadata (v0.1.1)




## [0.1.0] - 2025-11-06

### ⚙️ Miscellaneous


- Add .dev-notes.md to gitignore

- Gitignore personal development scripts


### ⚡ Features


- Add workspace structure and CI/CD pipeline

- Implement ToAsyncApiMessage derive macro

- Add JSON Schema generation with schemars integration

- Implement #[asyncapi(...)] helper attributes

- Add triggers_binary attribute for binary message support

- Implement #[derive(AsyncApi)] for spec generation

- Enhance AsyncApi derive with servers, channels, and operations

- Add message integration with #[asyncapi_messages(...)]


### 📚 Documentation


- Add working examples and update README

- Add spec file generation and rustdoc integration guide

- Add real-world framework integration examples

- Comprehensive rustdoc enhancements across all crates

- Update README with message integration feature




### New Contributors

* @mlilback made their first contribution



