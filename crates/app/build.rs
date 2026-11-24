fn main() {
    println!("cargo:rerun-if-changed=../../packaging/windows");
    #[cfg(windows)]
    embed_resource::compile("../../packaging/windows/flexolotl.rc", embed_resource::NONE)
        .manifest_optional()
        .expect("the icon can be embedded");
}
