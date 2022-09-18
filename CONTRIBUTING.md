# Contributing

We are open to, and grateful for, any contributions made by the community. By contributing to axios, you agree to abide by the [code of conduct](https://github.com/mikesposito/glue/blob/main/CODE_OF_CONDUCT.md).

### Code Style

Please follow the [node style guide](https://github.com/felixge/node-style-guide).

### Commit Messages

Commit messages should be verb based, using the following pattern:

type(scope): Short description...

**type** can be one of feat, fix, tests, chore, docs etc..

**scope** should immediately indicate the file, component or package involved in the commit (more detailed is better)

### Testing

Please update the tests to reflect your code changes. Pull requests will not be accepted if they are failing.

### Documentation

Please update the [docs](README.md) accordingly so that there are no discrepancies between the API and the documentation.

### Developing

- `cargo run -- <COMMAND>` development run
- `cargo build --release` build glue
