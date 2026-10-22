fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("rok_icon.ico");
        res.set("ProductName", "Rise of Kingdoms Loot Watcher & Analyzer");
        res.set("FileDescription", "Rise of Kingdoms - Motte, Fort & Barbarian Loot Watcher");
        res.set("LegalCopyright", "Copyright © 2026");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile windows resource: {}", e);
        }
    }
}
