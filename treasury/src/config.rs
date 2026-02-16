#[cfg(test)]
pub const GROUP_CONTRACT: &str =
    "CABYTW7GMOYRDOEYUTFQOFTYGPEFUZOOGYDIJLSYLDP7XFWQ4A2TFXP2";

#[cfg(test)]
pub const USDC_ADDRESS: &str =
    "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA";

#[cfg(all(not(test), feature = "testnet"))]
pub const GROUP_CONTRACT: &str =
    "CABYTW7GMOYRDOEYUTFQOFTYGPEFUZOOGYDIJLSYLDP7XFWQ4A2TFXP2";

#[cfg(all(not(test), feature = "testnet"))]
pub const USDC_ADDRESS: &str =
    "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA";

/// Replace with real mainnet contract IDs before deploying.
#[cfg(all(not(test), feature = "mainnet"))]
pub const GROUP_CONTRACT: &str = "CA_MAINNET...";

#[cfg(all(not(test), feature = "mainnet"))]
pub const USDC_ADDRESS: &str = "CA_MAINNET...";

#[cfg(not(any(test, feature = "testnet", feature = "mainnet")))]
compile_error!("You must enable either 'testnet' or 'mainnet' feature");