#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{process::Command, sync::Once};

    use lazy_static::lazy_static;
    use rustc_hash::FxHashMap;

    use crate::Info;
    use crate::OSInfo;
    lazy_static! {
        static ref FF_INFO: HashMap<String, String> = {
            let mut ff_tmp = HashMap::new();
            let output: String = String::from_utf8_lossy(
                Command::new("fastfetch")
                    .args(["--pipe"])
                    .output()
                    .unwrap()
                    .stdout
                    .as_slice(),
            )
            .to_string();
            let mut lines = output.lines();
            let (x, y) = lines.next().unwrap().split_once("@").unwrap();
            ff_tmp.insert("username".to_string(), x.into());
            ff_tmp.insert("hostname".to_string(), x.into());
            lines.next();
            for line in lines {
                if let Some((x, y)) = line.split_once(": ") {
                    ff_tmp.insert(x.into(), y.into());
                }
            }
            ff_tmp
        };
        static ref MIRA_INFO: Info = Info::new();
    }
    #[test]
    fn test_username() {
        assert_eq!(
            &AsRef::<str>::as_ref(FF_INFO.get("username").unwrap()),
            &(MIRA_INFO.username.as_ref().unwrap().as_ref())
        );
    }
}
