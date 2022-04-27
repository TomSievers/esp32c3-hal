//! ESP32C3 Hardware Abstraction Layer.
#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg), feature(doc_cfg_hide))]
#![warn(missing_docs)]

pub use esp32c3 as pac;
pub use embedded_hal as hal;

pub mod gpio;
pub mod dma;

#[cfg(feature = "rt")]
pub use esp32c_rt;