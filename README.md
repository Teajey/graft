<span align="center">

# 🔧🚧⚠️👷 **Work In Progress** 👷⚠️🚧🔨

</span>

# Accord

Generate GraphQL client Typescript **_bLaZiNgLy_** fast with Rust!

Inspired by [graphql-code-generator](https://github.com/dotansimha/graphql-code-generator); I just don't think it's usage of Typescript is up to scratch, so naturally I had to make a whole other version in Rust.

## Installation

Hopefully soon

```
yarn add -D @teajey/accord
```

But for now

```
yarn add -D accord@https://github.com/Teajey/accord#workspace=@teajey/accord
```

Note: Rust's `cargo` build tool must be installed.

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
