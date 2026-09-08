fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        // Fixed ID so the tray icon can load this exact embedded resource at
        // runtime (see platform::windows::load_frog_icon) instead of depending
        // on a loose .ico file shipped next to the exe.
        res.set_icon_with_id("assets/frog.ico", "1");
        res.compile().expect("failed to embed exe icon resource");
    }
}
