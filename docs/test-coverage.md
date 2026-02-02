# Test Coverage Tracking

## Baseline (2026-02-02)
Command:
```
cargo llvm-cov --tests --workspace
```

Summary:
- Regions: 47.94%
- Functions: 32.39%
- Lines: 45.90%

Business-logic highlights:
- `src/portfolio/mod.rs`: Regions 97.71%, Functions 85.71%, Lines 97.94%
- `src/services/auth.rs`: Regions 90.92%, Functions 72.46%, Lines 92.94%
- `src/services/finance.rs`: Regions 68.18%, Functions 52.70%, Lines 72.88%
- `src/services/goal.rs`: Regions 88.57%, Functions 72.55%, Lines 86.58%
- `src/services/pocket.rs`: Regions 85.77%, Functions 71.05%, Lines 86.67%
- `src/services/subscription.rs`: Regions 70.43%, Functions 46.15%, Lines 75.28%
- `src/services/transaction.rs`: Regions 80.82%, Functions 49.41%, Lines 78.81%
- `src/services/user_subscription.rs`: Regions 43.88%, Functions 40.00%, Lines 34.60%

Notes:
- Unit tests are in-module for services and portfolio.
