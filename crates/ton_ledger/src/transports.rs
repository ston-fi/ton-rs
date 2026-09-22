//! Native transports; neither feature is required for custom transports.
#[cfg(test)]
mod _test_framing;
#[cfg(feature = "ble")]
pub mod ble;
#[cfg(any(feature = "hid", feature = "ble", test))]
mod framing;
#[cfg(feature = "hid")]
pub mod hid;
