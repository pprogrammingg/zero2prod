# Recommended Coding Guidelines

# IDE

- It is recommended to use `Intellij RustRover`

# Clippy

- Make sure `clippy` is installed: `cargo component install clippy`
- Go to

```
  File -> Settings -> Rust -> External Linters, 
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
    File -> Settings -> search for `RustFmt`. Select `Use Rustfmt instead of built-in formatter`.
```

Then

```
    Click the Configure actions on save link.
    Check Set the Reformat code checkbox.
```

## Cargo.toml

- keep package names alphabetical
- Omit patch numbers from dependency versions. Cargo will automatically find the latest patch number.

## Error Handling

- As a general rule, errors in `main.rs` should panic, however, anywhere else errors must be bubbled up and handled i.e.
  do not simply use `unwrap` on expressions that return `Result<T, E>`. Rather use proper error handling syntax sucha
  as `?`.

- If capturing specific messages from a lower level using, make sure to bubble it up in error types at higher levels.

Note: The example below specifically list `thiserror` library usage for errors but it can be applied to similar crates.
In the error below the value inside VerifyConnection is not used.

```rust
    #[error("connection could not be verified")]
VerifyConnection(String)
```

Change to include the error message inside its string variable:

```rust
    #[error("connection could not be verified: {0}")]
VerifyConnection(String),
```
# General Software Engineering Patterns
## Valid Inputs
- [Parse, don't validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
- Take advantage of the "New Type" pattern in Rust, so that types that could potentially be invalid can be converted to types
that are garaunteed to be valid after parsing them. Relat

## CD Pipeline
- try to use bare minimum images as the runtime for code
- stage and separate dependency and core code build (core code in a repo changes more often than dependencies).
Cache different stages of building, including dependencies.

  
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


