use alloy::sol;

#[allow(clippy::too_many_arguments)]
pub mod uniswap_v4 {
    use super::*;

    sol!(
        #[derive(Debug)]
        #[sol(rpc)]
        UniswapV4,
        "abis/StateView.json"
    );
}
