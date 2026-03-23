# Test Suite

Test files and runner from https://github.com/munificent/craftinginterpreters (MIT license).

Upstream commit: see `UPSTREAM_COMMIT`

## Structure

- `test/` — Lox test files (`.lox`) organized by language feature
- `tool/` — Dart test runner

## Running tests

From the repo root:

```bash
./run_tests.sh jlox
```

Or filter by feature:

```bash
./run_tests.sh jlox scanning
```

## Prerequisites

- Dart SDK (for the test runner)
- Run `cd tests/tool && dart pub get` once to install dependencies
