<span align="center">

# 🔧🚧⚠️👷 **Work In Progress** 👷⚠️🚧🔨

</span>

# Graft

Generate GraphQL client Typescript **_bLaZiNgLy_** fast with Rust!

Inspired by [`graphql-code-generator`](https://github.com/dotansimha/graphql-code-generator); I just don't think it's usage of Typescript is up to scratch, so naturally I had to make a whole other version in Rust.

## (Planned) Features

- [x] Generate basic GraphQL types
- [x] Option to generate a GraphQL AST file of the schema
- [ ] ~Everything that `graphql-code-generator` can do~ Eh, maybe just the stuff that I need
- [ ] User can arbitrarily extend the generated types with a `*.config.js`-like file instead of plugins

## Build

As a native Rust binary

```
cargo build
```

As an executable NPM package

```
yarn build
```

## Installation

Hopefully soon

```sh
# NPM package
yarn add -D @teajey/graft

# Rust binary
cargo install teajey-graft
```

But for now

```sh
# NPM package
yarn add -D @teajey/graft@https://github.com/Teajey/graft#workspace=@teajey/graft

# Rust binary
cargo install teajey-graft --git https://github.com/Teajey/graft
```

Yarn (or your package manager of choice) may hang for a while as the Rust is compiled; i.e.:

```
➤ YN0007: │ @teajey/graft@https://github.com/Teajey/graft.git#workspace=%40teajey%2Fgraft&commit=b7aa83a082f10be8df25f3ac48d622c3b575c9cf must be built because it never has been before or the last one failed
```

## Configuration

Example config

```yml
# .graft.yml
generates:
  mySchema:
    schema:
      url: "{{MY_DOMAIN}}" # environment variable interpolation supported
      no_ssl: true
      out:
        ast: schema.graphql
        json: schema.json
    typescript:
      ast: schema.graphql
      documents: document.graphql
      out: generated.ts
```

## Usage

```
yarn run graft
```
