fn main() {
    println!("cargo:rerun-if-changed=capslang.rc");
    println!("cargo:rerun-if-changed=assets/capslang.ico");
    // The resource script gives the .exe its Explorer icon and version info.
    // Only MSVC ships a resource compiler we can rely on; the tray icon is
    // embedded with include_bytes! either way, so GNU builds lose nothing but
    // the file icon.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        embed_resource::compile("capslang.rc", embed_resource::NONE);
    }
}
