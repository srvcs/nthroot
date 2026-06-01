# srvcs-nthroot

## Name

| Field | Value |
| --- | --- |
| Service | `srvcs-nthroot` |
| Slug | `nthroot` |
| Repository | `srvcs/nthroot` |
| Package | `srvcs-nthroot` |
| Kind | `orchestrator` |

## Function

arithmetic: nth root of value (alias of root)

## Dependencies

| Dependency | Repository |
| --- | --- |
| `srvcs-root` | [srvcs/root](https://github.com/srvcs/root) |

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity |
| `POST` | `/` | Evaluate the service function |
| `GET` | `/healthz` | Liveness probe |
| `GET` | `/readyz` | Readiness probe |
| `GET` | `/metrics` | Prometheus metrics |
| `GET` | `/openapi.json` | OpenAPI document |

## Inputs

| Name | Type | Required |
| --- | --- | --- |
| `value` | `json` | yes |
| `n` | `json` | yes |

## Outputs

| Name | Type |
| --- | --- |
| `value` | `json` |
| `n` | `json` |
| `result` | `number` |

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |
| `SRVCS_ROOT_URL` | `http://127.0.0.1:8120` | Base URL for srvcs-root |

## Error Behavior

- `422` means the request could not be evaluated for the documented input shape.
- `503` means a required dependency was unavailable or returned an unexpected response.
- Dependency validation errors are forwarded when this service delegates validation.

## Local Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

See the [srvcs service standard](https://github.com/srvcs/platform/blob/main/STANDARD.md) for the full operational contract.

## Metadata

Machine-readable service metadata lives in `srvcs.yaml`. Keep it aligned with this README when the service contract changes.
