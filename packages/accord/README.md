<span align="center">

# 🔧🚧⚠️👷 **Work In Progress** 👷⚠️🚧🔨

</span>

# Accord

Generate GraphQL client Typescript **_bLaZiNgLy_** fast with Rust!

Inspired by [`graphql-code-generator`](https://github.com/dotansimha/graphql-code-generator); I just don't think it's usage of Typescript is up to scratch, so naturally I had to make a whole other version in Rust.

## (Planned) Features

- [x] Generate basic GraphQL types
- [ ] ~Everything that `graphql-code-generator` can do~ Eh, maybe just the stuff that I need
- [ ] Option to generate a GraphQL AST file of the schema
- [ ] User can arbitrarily extend the generated types with a `*.config.js`-like file instead of plugins

## Build

As a native Rust binary

```
cargo build --features native
```

As an executable NPM package

```
yarn build
```

## Installation

Hopefully soon

```sh
# NPM package
yarn add -D @teajey/accord

# Rust binary
cargo install teajey-accord
```

But for now

```sh
# NPM package
yarn add -D @teajey/accord@https://github.com/Teajey/accord#workspace=@teajey/accord

# Rust binary
cargo install teajey-accord --git https://github.com/Teajey/accord
```

Yarn (or your package manager of choice) may hang for a while as the Rust is compiled; i.e.:

```
➤ YN0007: │ @teajey/accord@https://github.com/Teajey/accord.git#workspace=%40teajey%2Faccord&commit=b7aa83a082f10be8df25f3ac48d622c3b575c9cf must be built because it never has been before or the last one failed
```

## Configuration

Example config

```yml
# .accord.yml
schema: https://localhost:9443/trivia/graphql
no_ssl: true
document: document.graphql
```

## Usage

```
yarn run accord
```
