use archive::{archive_flags, archive_icons, FLAG_FILE, ICON_FILE};
use std::{env, fs, path::Path};

fn main() {
    let out_dir: &Path = &(Path::new(&env::var("OUT_DIR").unwrap()).join("../../../")); //todo: see if there's a less hacky way to do this
    println!("cargo:rerun-if-changed=data/{FLAG_FILE}");
    println!("cargo:rerun-if-changed=data/{ICON_FILE}");

    fs::DirBuilder::new().create(out_dir).ok();
    fs::copy(Path::new("./data").join(ICON_FILE), out_dir.join(ICON_FILE)).unwrap();
    fs::copy(Path::new("./data").join(FLAG_FILE), out_dir.join(FLAG_FILE)).unwrap();
    archive_flags(&out_dir.join(FLAG_FILE)).unwrap();
    archive_icons(&out_dir.join(ICON_FILE)).unwrap();
}
