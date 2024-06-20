pub mod driver;
pub use driver::*;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "web")]
pub type Dvr = web::Driver;

#[cfg(not(feature = "web"))]
pub mod native;

#[cfg(not(feature = "web"))]
pub type Dvr = native::Driver;
