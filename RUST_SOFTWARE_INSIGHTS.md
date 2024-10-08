# Recommended Coding Guidelines

# IDE

- It is recommended to use `Intellij RustRover`. The following instructions pertain to RustRover.

# Clippy

- Make sure `clippy` is installed: `rustup component add clippy-preview`
- Go to

```
  File -> Manage IDE Settings -> Settings Sync ... -> Rust -> External Linters, 
  select Clippy & enable "Run external linter on the fly"
```

- Make warnings fail linting:
  `cargo clippy -- -D warnings`

- Note: one can disable a particular clippy warning if believed not to be correct via annotating
  the offending code with

```rust
    #[allow(clippy::lintname)]
```

# Rustfmt

- Enable RustFmt in RustRover

```
    File -> Manage IDE Settings -> Settings Sync ...  -> search for `RustFmt`. Select `Use Rustfmt instead of built-in formatter`.
```

Then

```
    Click the Configure actions on save link.
    Check Set the Reformat code checkbox.
```

# Install Nightly Rust

Nightly Rust is used to format the code in CI. To use it locally and be close to how CI works, need to install it.

Install nightly Rust:

```
 rustup toolchain install nightly
```

Note: In Clippy setting above, go to the same `ExternalLinter` settings and select channel at `nightly` to run clippy
also via nightly channel

# Cargo.toml

- keep package names alphabetical
- Omit patch numbers from dependency versions. Cargo will automatically find the latest patch number.

# Error Handling

- Error should help operators debug and troubleshoot. Users can still just receive 500 Internal Server error as
  they do not have a mental model of the internals; however, in cases such wrong input to the system, they shouldg
  get a helpful message.

- Errors can be internal (use enum variants, fields, methods to control flow and ultimately end in log/traces)
  or at the edge at API level (rely on status code and are propagated in response body)

- As a general rule, errors in `main.rs` should panic, however, anywhere else errors must be bubbled up and handled i.e.
  do not simply use `unwrap` on expressions that return `Result<T, E>`. Rather use proper error handling syntax sucha
  as `?`.

- In `actix_web` foreign errors can be wrapped in a local type as `newType` and then have `ResponseError` trait
  implemented on them. This way the foreign error type can use `into()` to get converted to `actix_web::Error` type to
  be propagated up the chain.

- Rust `std::error::Error` trait

```rust
pub trait Error: Debug + Display {
    /// The lower-level source of this error, if any.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
```

Returns `Option` of `Trait Object` to keep underlying error type opaque. `std::error::Error` trait is a good way to
standardize erorrs so that components importing your package, can get `Debug`, `Display` amd `source` for free.

- If capturing specific messages from a lower level using, make sure to bubble it up in error types at higher levels.
  Note: The example below specifically list `thiserror` library usage for errors but it can be applied to similar
  crates. In the error below the value inside VerifyConnection is not used.

```rust
    #[error("connection could not be verified")]
VerifyConnection(String)
```

Change to include the error message inside its string variable:

```rust
    #[error("connection could not be verified: {0}")]
VerifyConnection(String)
```

- `thiserror` takes a way a lot of boilderplate code via procedural macros:
    - [#from] : does a `From` conversion from the underlying error type to the top level
    - [#source] : delegates `Source` method from underlying error
    - [#transparent]: delegtes `Source` and `Display` from the underlying error type

- `anyhow`
  anyhow::Error is a wrapper around a dynamic error type. anyhow::Error works a lot like Box<dyn
  std::error::Error>, but with these differences:
  • anyhow::Error requires that the error is Send, Sync, and 'static.
  • anyhow::Error guarantees that a backtrace is available, even if the underlying error type does
  not provide one.
  • anyhow::Error is represented as a narrow pointer — exactly one word in size instead of two.
- anyhow::Error provides the capability to enrich an error with additional context out of the box.
  like adding descriptive strings (using `.context("describtive error")`)
  `context` does 2 things:
  • it converts the error returned by our methods into an `anyhow::Error`;
  • it enriches it with additional context around the intentions of the caller.
- `anyhow` or `thiserror`
  Read chapter 8.4.2 of `Zerto to Prod in Rust`; basically if you do not need so much programmatic
  handling of various error enums and error is just for operator, then use opaque error using
  `anyhow` or `eyre`.

- Rule of error handling
  `errors should be logged when they are handled.`
-

# Tests

Sample test to filter out noise and only allow `error` and `info` to show as well as enable `TEST_LOG`.

```bash
export RUST_LOG="sqlx=error,info"
export TEST_LOG=true
cargo t subscribe_fails_if_there_is_a_fatal_database_error | bunyan
```

- Mock
    - `mount` vs `mount_as_scoped`, the latter returns MockGaurd which has custom drop and gets dropped when test
      exists, appropriate when mocking only in specifically cases. Such as loxal to a test helper. Expectation is
      eagerly
      checked before going being dropped.
    -

# General Software Engineering Patterns

## Database

- Use all or nothing approach using `transactions` to not leave the DB in dirty state

## Parse, don't validate

- Instead of using validate functions everywhere to return a bool, define a parse pattern that takes user input
  and returns a data structure that guarantees to have valid
  fields[Parse, don't validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)

## Socket exhaustion due to multiple network  connections

- Don't create a brand-new connection each time a new request needs to be made
  [Socket Exhaustion](https://www.aspnetmonsters.com/2016/08/2016-08-27-httpclientwrong/)
  Note: most HTTP client (including `reqwest` in Rust re-use the existing connection as long as the same
  client instance is used - even if `Client::clone()` is used, because `clone()` in this case just creates)
  a pointer to the underlying client.

## Timeouts and IO Operation

- Any time performing an IO operation, always set a timeout!

## CD Pipeline

- try to use bare minimum images as the runtime for code
- stage and separate dependency and core code build (core code in a repo changes more often than dependencies).
  Cache different stages of building, including dependencies.
- Run Sqlx migration against remote DB:

```bash
DATABASE_URL=<REMOTE_DB_ADDRESS> sqlx migrate run
```

# General Team Way of Work (WoW) (sprints, user stories, etc.)

## User Stories

Capture requirements using "User Story" format:

```
As a..., I want to..., So that ...
```

E.g.

```
As a DeFi user, I want to stake my Hehe token, So that I can accumulate 7% staking APY.
```

## PRs

- Try to limit the scope of the PR to 1 ticket at a time. For example, focus on exposing the API and API error handling,
  or focus on domain level implementation. This makes the PR more concise, which means you can focus on a single
  deliverable.
- Avoid adding unused or dead code, if the task doesn't require it (e.g. common_config.rs). This also means: don't add
  impl blocks or derive macros without knowing that they are necessary. Avoid copy and paste.
- Do a self-review on your PRs before sending them for review. A lot of issues can be caught by doing a thorough review
  yourself.
- PR review should not focus on formatting, this should be taken care of via agreed-upon automatic formatting in
  dev's local machine
- PRs messages can follow a standard set of guidelines. For example, use of the words `feature`, `chore`, etc. better
  clarifies the intent of the PR.
  see [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0-beta.2/)

## Iterative Approach

- Do not go too deep on one story, rather iterate by first introducing essential functionalities
  and then adding other things. For example, subscription ability first, then adding fault-tolerant behaviour





