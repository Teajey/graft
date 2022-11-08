#[cfg(feature = "node")]
mod cli;
#[cfg(feature = "node")]
mod config;
#[cfg(feature = "node")]
mod gen;
#[cfg(feature = "node")]
mod introspection;
#[cfg(feature = "node")]
mod node;
#[cfg(feature = "node")]
mod typescript;
#[cfg(feature = "node")]
mod util;

#[cfg(feature = "node")]
pub use node::main::node_main;
