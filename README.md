# srvcs-nthroot

The nth-root orchestrator of the srvcs.cloud distributed standard library.

Its single concern: **arithmetic: nth root of value (alias of root).** It is a
thin alias of [`srvcs-root`](https://github.com/srvcs/root): it owns the
*control flow* but does no arithmetic of its own. It forwards `{"value", "n"}`
to `srvcs-root` and returns its `result`.

```
nthroot(value, n):
    return root(value, n).result
```

The result is a float (an f64 that may be fractional), e.g.
`nthroot(27, 3) ~= 3.0`.

Validation is not handled here. This service never calls `srvcs-isnumber`
directly; instead `srvcs-root` validates its own operands, and any `422` it
raises is forwarded verbatim.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity, concern, and dependency list |
| `POST` | `/` | Compute the nth root of `value` |
| `GET` | `/healthz` `/readyz` `/metrics` `/openapi.json` | srvcs service standard surface |

```sh
curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"value": 27, "n": 3}'
# {"value":27,"n":3,"result":3.0}
```

Responses:

- `200 {"value": value, "n": n, "result": r}` — evaluated; `result` is a float.
- `422` — `srvcs-root` rejected the input, forwarded verbatim.
- `500` — a reachable dependency returned a `200` without a usable result.
- `503` — `srvcs-root` is unavailable.

## Dependencies

- [`srvcs-root`](https://github.com/srvcs/root)

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ROOT_URL` | `http://127.0.0.1:8120` | Base URL of `srvcs-root` |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |

## Local checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Orchestration tests stand up a *computing* mock `srvcs-root` service in-process
— it reads the request body and returns the real `value.powf(1 / n)`, so the
composition is genuinely exercised against the asserted cases (compared
approximately, within `1e-9`). See
[`srvcs/platform`](https://github.com/srvcs/platform) for the shared standard.

> Note: the `cargoHash` in `flake.nix` is inherited from the template and must be
> refreshed with a `nix build` before the Nix gates pass.
