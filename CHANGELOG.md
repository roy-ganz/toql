# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## 0.4.1 - 2021-12-15

### Fixed
- Fix wrong QueryField naming for joins / merges.
- Fix regression with Rust names starting with `r#`

### Changed
- Improve string representation of SqlArg.

## 0.4.0 - 2021-12-04

### Fixed
- Fix insert behaviour for joins that contain a key instead of a value.
- Fix missing TreeInsert dispatching for joined keys.
- Fix invalid code generation with optional merges.
- Fix bug with hidden fields.
- Fix insert execution order: top entities come after joins.
- Fixed doc tests.

### Added
- MockDb, a mocking database backend.
- Unit tests.
- Integration tests.

Tarpaulin reported test coverage is 87%.

### Changed
- Make guide a separate repository for better versioning.
- Change syntax of boolean attributes in Toql derive. Incompatible with version 0.3.
- Complete rewrite of Toql derive code generation.
- String representation of role expressions.

## 0.3.0 - 2021-10-21

### Changed
- Major overhaul of everything. Incompatible with version 0.2.

