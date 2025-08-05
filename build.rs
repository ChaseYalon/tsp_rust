extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let icon_path = "assets/globe.ico";

        if std::path::Path::new(icon_path).exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon(icon_path);

            if let Err(e) = res.compile() {
                println!("cargo:warning=winres compile failed: {:?}", e);
            }
        } else {
            println!("cargo:warning=Icon file not found: {}", icon_path);
        }
    }
}
