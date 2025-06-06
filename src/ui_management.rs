pub fn add_gamesave() -> Option<(String, String)> {
    match rfd::FileDialog::new().set_directory(".").pick_file() {
        Some(data) => Some((data.file_name()?.to_str()?.to_string(), data.as_path().to_str().unwrap().to_string())),
        _ => None
    }
}