# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `@` filter (regular expression switch).
- Alternative way to write range of substring filters as `start+length`.

### Changed

- `l` filter (to uppercase) renamed to `v`.
- `L` filter (to lowercase) renamed to `^`.
- `0` is now a valid filter a no longer considered error.
- Simplified error message for an invalid range.
- Simplified output of `--help-pattern` and `--help-filters` flags.
- Output of `-h, --help` flag is organized into sections.
- Output of `-h, --help` flag uses more colors in descriptions.
- Regular expression `-e. --regex` / `-E. --regex-filename` is now called *global* instead of *external*.

### Fixed

- `--help-filters` flag displays correct name of `i` / `I` filters.

## [0.1.0] - 2020-12-13

- Initial release.

[Unreleased]: https://github.com/jpikl/compare/v1.0.0...HEAD
[0.1.0]: https://github.com/jpikl/rew/releases/tag/v0.1.0
