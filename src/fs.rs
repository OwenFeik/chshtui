fn app_directory() -> std::path::PathBuf {
    let home = std::env::home_dir().expect("std::env::home_dir() was None");
    home.join(format!(".local/share/{}", crate::APP_NAME))
}

fn data_directory() -> std::path::PathBuf {
    app_directory().join("data")
}

pub fn read_data(name: &str) -> std::io::Result<impl std::io::Read> {
    std::fs::File::open(data_directory().join(name))
}

pub fn write_data(name: &str, data: impl AsRef<[u8]>) -> std::io::Result<()> {
    let dir = data_directory();
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(name), data)
}
