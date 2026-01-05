## 1. Implementation
- [x] 1.1 Update DNS provider row parsing to avoid silent JSON parse failures; log with provider id and field and return a contextual error or explicit fallback.
- [x] 1.2 Update legacy DNS migration timestamp/config parsing to log malformed data and either propagate or document fallback with warnings.
- [x] 1.3 Log best-effort cleanup failures in secret metadata deletion and secret resolution checks.
- [x] 1.4 Log config_json parse failures when deriving zone overrides in commands and issuance flow.
- [x] 1.5 Add or update tests as needed to cover parsing/logging behavior changes.
