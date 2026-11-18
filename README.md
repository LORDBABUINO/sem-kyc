# Dever Cívico

An interactive talk about how the Bitcoin network works and how to transact with
privacy, played as a game. The audience joins through a QR code, each person runs a
small business in one of eight sectors and buys supplies from the others, paying in
bitcoin on an in-memory ledger. The speaker plays the DOE, the Departamento de Ordem Econômica of the República
Democrática de Nova Alvorada: it reads the chain, prints a newspaper called O Rastro
every round, assesses ICMS on every operation it can attribute to someone, and
explains, from the surveillance side, what not to do.

The DOE's view is the same set of heuristics that
[Stealth](https://github.com/stealth-bitcoin/stealth) runs for the wallet owner.
At the end of the talk every player receives their Stealth report.

## Layout

- `core/`: game rules. The ledger, the State's cluster view, the economy, the ICMS,
  and the bridge to `stealth-engine`.
- `server/`: HTTP and WebSocket transport, serves the three screens.
- `frontend/`: the player's phone, the projector and the presenter panel
  (React, Vite, CSS Modules).
- `roteiro.md`, `telas.md`, `economia/`: the talk script, the screen spec and the
  economy data the game loads.

## Running

```bash
cargo run -p dever-server
cd frontend && yarn && yarn dev
```

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd frontend && yarn test --run
```

Commits follow conventional commits (`feat(core): ...`). Tests come before the code
they cover.

## License

MIT.
