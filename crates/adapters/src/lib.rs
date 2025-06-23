// Define trading adapters here

#[allow(unused)]
mod coinbase;
pub use coinbase::*;

#[allow(unused, clippy::too_many_arguments)]
pub mod uniswap_v4;
#[allow(unused)]
pub use uniswap_v4::*;
