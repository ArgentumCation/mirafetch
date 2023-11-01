use arcstr::ArcStr;

use crate::info::OSInfo;

pub struct IosInfo {}
impl IosInfo {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}
impl OSInfo for IosInfo {
    fn sys_font(&self) -> Option<ArcStr> {
        None
    }

    fn cursor(&self) -> Option<ArcStr> {
        None
    }

    fn terminal(&self) -> Option<ArcStr> {
        None
    }

    fn term_font(&self) -> Option<ArcStr> {
        None
    }

    fn gpus(&self) -> Vec<arcstr::ArcStr> {
        Vec::new()
    }

    fn memory(&self) -> Option<ArcStr> {
        None
    }

    fn disks(&self) -> Vec<(ArcStr, ArcStr)> {
        Vec::new()
    }

    fn battery(&self) -> Option<ArcStr> {
        None
    }

    fn locale(&self) -> Option<ArcStr> {
        None
    }

    fn theme(&self) -> Option<ArcStr> {
        None
    }

    fn icons(&self) -> Option<ArcStr> {
        None
    }

    fn os(&self) -> Option<ArcStr> {
        None
    }

    fn displays(&self) -> Vec<arcstr::ArcStr> {
        Vec::new()
    }

    fn machine(&self) -> Option<ArcStr> {
        None
    }

    fn kernel(&self) -> Option<ArcStr> {
        None
    }

    fn wm(&self) -> Option<ArcStr> {
        None
    }

    fn de(&self) -> Option<ArcStr> {
        None
    }

    fn shell(&self) -> Option<ArcStr> {
        None
    }

    fn cpu(&self) -> Option<ArcStr> {
        None
    }

    fn username(&self) -> Option<arcstr::ArcStr> {
        None
    }

    fn id(&self) -> arcstr::ArcStr {
        todo!()
    }

    fn uptime(&self) -> Option<ArcStr> {
        todo!()
    }

    fn ip(&self) -> Vec<arcstr::ArcStr> {
        todo!()
    }

    fn hostname(&self) -> Option<arcstr::ArcStr> {
        todo!()
    }
}
