#[cfg(target_os = "macos")]
use arcstr::ArcStr;

#[cfg(target_os = "macos")]
use sysctl::Sysctl;

use crate::info::OSInfo;

pub struct MacInfo {}

impl MacInfo {
    pub fn new() -> Self {
      MacInfo {}
    }
}

impl OSInfo for MacInfo {
    fn os(&self) -> Option<ArcStr> {
       None 
    }

    fn hostname(&self) -> Option<ArcStr> {
       Some(ArcStr::from(whoami::hostname()))
    }

    fn displays(&self) -> Vec<ArcStr> {
       vec![]
    }

    fn machine(&self) -> Option<ArcStr> {
       None 
    }

    fn kernel(&self) -> Option<ArcStr> {
       None 
    }

    #[allow(clippy::similar_names)]
    fn gpus(&self) -> Vec<ArcStr> {
       vec![]
    }

    // TODO
    fn theme(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn wm(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn de(&self) -> Option<ArcStr> {
        None
    }

    fn shell(&self) -> Option<ArcStr> {
       None 
    }

    fn cpu(&self) -> Option<ArcStr> {
       let model = sysctl::Ctl::new("machdep.cpu.brand_string").unwrap().value_string().unwrap();
       let core_count = sysctl::Ctl::new("machdep.cpu.core_count").unwrap().value_string().unwrap();

       
       Some(arcstr::format!(
          "{} ({})",
          model,
          core_count
       ))
    }

    fn username(&self) -> Option<ArcStr> {
       Some(ArcStr::from(whoami::username()))
    }

    // TODO
    fn sys_font(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn cursor(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn terminal(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn term_font(&self) -> Option<ArcStr> {
        None
    }

    fn memory(&self) -> Option<ArcStr> {
       None 
    }

    fn ip(&self) -> Vec<ArcStr> {
      	vec![] 
    }

    fn disks(&self) -> Vec<(ArcStr, ArcStr)> { 
       vec![]
    }

    fn battery(&self) -> Option<ArcStr> {
       None 
    }

    fn locale(&self) -> Option<ArcStr> {
        std::env::var("LANG")
            .ok()
            .filter(|x| !x.is_empty())
            .or_else(|| std::env::var("LC_ALL").ok().filter(|x| !x.is_empty()))
            .or_else(|| std::env::var("LC_MESSAGES").ok().filter(|x| !x.is_empty()))
            .map(ArcStr::from)
    }

    fn uptime(&self) -> Option<ArcStr> {
       None 
    }

    // TODO
    fn icons(&self) -> Option<ArcStr> {
        None
    }

    fn id(&self) -> ArcStr {
       ArcStr::from("macos") 
    }
}
