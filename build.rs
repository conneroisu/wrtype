fn main() {
    // Re-run build script if protocol file changes
    println!("cargo:rerun-if-changed=wtype/protocol/virtual-keyboard-unstable-v1.xml");
}
