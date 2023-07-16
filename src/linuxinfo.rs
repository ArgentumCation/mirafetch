#![cfg(target_family = "unix")]
use crate::traits::OSInfo;
pub struct UnixInfo {}
impl OSInfo for UnixInfo {
    fn new() -> Self {
        Self {}
    }
}
