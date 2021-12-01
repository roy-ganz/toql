# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## 0.3.1 -

### Fixed
- Fix insert behaviour for joins that contain a key instead of a value.
- Fix missing TreeInsert dispatching for joined keys.
- Fix invalid code generation with optional merges.
- Fix bug with hidden fields

### Added
- MockDb, a mocking database backend.
- Unit tests.
- Integration tests.
- Fixed doc tests.

Tarpaulin reported test coverage is XX%.

### Changed
- Make guide a separate repository for better versioning.
- Change syntax of boolean attributes in Toql derive
- Complete rewrite of Toql derive code generation 

## 0.3.0 - 2021-10-21

### Changed
- Major overhaul of everything. Incompatible with version 0.2.

