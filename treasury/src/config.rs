#[cfg(test)]
pub const GROUP_CONTRACT: &str =
    "CABYTW7GMOYRDOEYUTFQOFTYGPEFUZOOGYDIJLSYLDP7XFWQ4A2TFXP2";

#[cfg(test)]
pub const USDC_ADDRESS: &str =
    "CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75";

#[cfg(all(not(test), feature = "testnet"))]
pub const GROUP_CONTRACT: &str =
    "CABYTW7GMOYRDOEYUTFQOFTYGPEFUZOOGYDIJLSYLDP7XFWQ4A2TFXP2";

#[cfg(all(not(test), feature = "testnet"))]
pub const USDC_ADDRESS: &str =
    "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";

#[cfg(all(not(test), feature = "mainnet"))]
pub const GROUP_CONTRACT: &str =
    "CA_MAINNET...";

#[cfg(all(not(test), feature = "mainnet"))]
pub const USDC_ADDRESS: &str =
    "CA_MAINNET...";

#[cfg(not(any(test, feature = "testnet", feature = "mainnet")))]
compile_error!("You must enable either 'testnet' or 'mainnet' feature");