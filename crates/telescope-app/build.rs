fn main() {
    // GPUI loads the window and taskbar icon from resource ID 1 of the exe.
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=icons/icon.ico");
        winresource::WindowsResource::new()
            .set_icon("icons/icon.ico")
            .compile()
            .expect("failed to embed the Windows icon");
    }
}
