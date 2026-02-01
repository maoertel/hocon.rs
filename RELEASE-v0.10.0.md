# Release v0.10.0

This release brings a major modernization of the crate, updating all dependencies to their latest versions and delivering **up to 308x performance improvement** on large configuration files.

## Highlights

- **Massive performance gains**: 10-308x faster parsing on large configs
- **Modern dependencies**: All dependencies updated to latest versions
- **nom 8.0**: Migrated from nom 4.2 to nom 8.0
- **Rust 2024 edition**: Updated from Rust 2018 to 2024 edition
- **CI/CD**: Added GitHub Actions workflow
- **Fully backwards compatible**: Byte-identical output verified

## Pull Requests

| PR | Title | Description |
|----|-------|-------------|
| [#1](../../pull/1) | Update nom 4 to 7 | Initial parser migration from nom 4.2 macro-based API to nom 7.1 function-based API |
| [#2](../../pull/2) | GitHub Actions CI workflow | Added CI with fmt, clippy, tests, and HOCON test suite validation |
| [#3](../../pull/3) | Update safe dependencies | Updated patch versions of all compatible dependencies |
| [#4](../../pull/4) | Update dependencies with breaking changes | Updated thiserror 1.0→2.0, reqwest 0.11→0.13, java-properties 1.4→2.0 |
| [#5](../../pull/5) | Update remaining dependencies | Updated aho-corasick 0.7→1.1, criterion 0.4→0.8, rand 0.8→0.9; replaced lazy_static with std::sync::OnceLock |
| [#6](../../pull/6) | Update nom 7.1 to 8.0 | Final parser migration to nom 8.0 with modern Parser trait API |

## Dependency Updates

| Dependency | Old Version | New Version | Notes |
|------------|-------------|-------------|-------|
| nom | 4.2 | **8.0** | Major parser rewrite |
| thiserror | 1.0 | 2.0 | |
| reqwest | 0.11 | 0.13 | TLS feature renamed |
| java-properties | 1.4 | 2.0 | |
| aho-corasick | 0.7 | 1.1 | New PatternID API |
| criterion | 0.4 | 0.8 | |
| rand | 0.8 | 0.9 | |
| insta | 1.34 | 1.46 | |
| lazy_static | 1.4 | *removed* | Replaced with std::sync::OnceLock |

## Rust Edition

Updated from **Rust 2018** to **Rust 2024** edition, taking advantage of:
- New `let` chains in `if` expressions
- Updated formatting rules
- `std::env::set_var` now requires `unsafe` blocks

## Performance

Benchmarked using [hoconvert](https://github.com/maoertel/hoconvert) CLI with [hyperfine](https://github.com/sharkdp/hyperfine).

### Large Configuration Files

| Config | Size | v0.9.0 (nom 4.2) | v0.10.0 (nom 8.0) | Improvement |
|--------|------|------------------|-------------------|-------------|
| large-nested.conf | 90KB, 6K lines | 2.19s | 18.7ms | **117x faster** |
| large-arrays.conf | 120KB, 7.5K lines | 2.37s | 66.6ms | **36x faster** |
| deep-nesting.conf | 45KB, 400 lines | 141ms | 13.8ms | **10x faster** |
| xlarge.conf | 416KB, 22K lines | 41.8s | 135.8ms | **308x faster** |

### Small Configuration Files

| Config | v0.9.0 (nom 4.2) | v0.10.0 (nom 8.0) |
|--------|------------------|-------------------|
| test01.conf | 2.8ms | 3.1ms |
| basic.conf | 2.8ms | 3.0ms |

*Note: Small file benchmarks are dominated by process startup overhead (~2.5ms), not parsing time. The apparent "regression" disappears when measuring actual parsing workloads.*

### Why the Dramatic Improvement?

The nom 4.2 parser had **O(n²) or worse complexity** due to macro-generated code patterns. The nom 7/8 function-based parsers achieve proper **O(n) linear complexity**, which becomes dramatically apparent on larger inputs:

```
Config Size     nom 4.2 Time    nom 8.0 Time    Ratio
45KB            141ms           13.8ms          10x
90KB            2.19s           18.7ms          117x
120KB           2.37s           66.6ms          36x
416KB           41.8s           135.8ms         308x
```

## Compatibility

- **Output**: Byte-identical JSON/YAML output verified across all versions (SHA256 checksums match)
- **API**: No breaking changes to public API
- **Edition**: Rust 2024 (requires Rust 1.85+)

## Verification

All changes verified with:
- `cargo fmt --all -- --check`
- `cargo clippy -- -D warnings`
- `cargo test --all-features` (28 tests)
- `cargo test --no-default-features`
- HOCON test suite (24/24 passing)
- Comprehensive A/B/C performance testing
- Byte-level output comparison (60+ config files)

## Contributors

- [@mockersf](https://github.com/mockersf) (original author)
- [@maoertel](https://github.com/maoertel)

## Full Changelog

https://github.com/maoertel/hocon.rs/compare/v0.9.0...v0.10.0
