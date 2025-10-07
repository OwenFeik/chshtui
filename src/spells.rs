const BASE_URL: &str =
    "https://api.github.com/repos/foundryvtt/pf2e/contents/packs/spells";

fn download_spell_folders() -> attohttpc::Result<Vec<String>> {
    #[derive(serde::Deserialize)]
    struct FileEntry {
        #[serde(rename = "type")]
        ty: String,
        name: String,
    }

    let data: Vec<FileEntry> = attohttpc::get(BASE_URL).send()?.json()?;
    Ok(data.into_iter().map(|entry| entry.name).collect())
}

#[test]
fn test() {
    dbg!(download_spell_folders());
    panic!();
}
