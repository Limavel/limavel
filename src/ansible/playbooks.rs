use include_dir::{include_dir, Dir};

static ANSIBLE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/provision/ansible");

pub fn write_all(dir: &std::path::Path) -> std::io::Result<()> {
    ANSIBLE_DIR.extract(dir)
}
