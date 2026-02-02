# Test Coverage Tracking

## Baseline (2026-02-02)
Command:
```
cargo llvm-cov --tests --workspace
```

Summary:
- Regions: 3.40%
- Functions: 1.40%
- Lines: 2.92%

Business-logic highlights:
- `src/portfolio/mod.rs`: Regions 97.71%, Functions 85.71%, Lines 97.94%
- `src/services.rs`: 0% coverage
- `src/auth.rs`: 0% coverage
- `src/investments.rs`: 0% coverage
- `src/schemas.rs`: 0% coverage
- `src/repository.rs`: 0% coverage

Notes:
- Only `src/portfolio/tests.rs` exists (10 passing tests).
