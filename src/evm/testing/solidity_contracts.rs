use alloy::sol;

sol!(
    #[sol(rpc)]
    MockERC20,
    "src/evm/testing/MockERC20.sol/MockERC20.json",
);
