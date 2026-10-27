fn main() {
    println!("cargo:rerun-if-env-changed=ROK_SUPABASE_URL");
    println!("cargo:rerun-if-env-changed=ROK_SUPABASE_ANON_KEY");
    println!("cargo:rerun-if-changed=.env");

    // Load .env file automatically at build time if present
    if let Ok(content) = std::fs::read_to_string(".env") {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').trim_matches('\'');
                if key == "ROK_SUPABASE_URL" || key == "ROK_SUPABASE_ANON_KEY" {
                    println!("cargo:rustc-env={}={}", key, val);
                }
            }
        }
    }

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
