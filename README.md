Following [Zero to Production in Rust](https://www.zero2prod.com/index.html?country_code=CA) the repo demonstrates a basic newsletter subscription system.

It includes:
-  Actix-web based routes
-  Integration test
-  CI and CD pipelines
-  Tracing and logging


Run or test locally:
Pre-req: docker, postgres CLI to interact with Postgres DB

1. bring up the database:
`./scripts/init_db.sh`

2. run the Rust app:
```
  RUST_LOG=<info, trace, etc.> cargo run
  # change to `cargo test` for testing
```

Note: `./configuration` folder is used for basic, local and prod env variables.
