use base64::Engine;

const BASE_URL: &str =
    "https://api.github.com/repos/foundryvtt/pf2e/contents/packs/spells";

#[derive(Debug)]
struct Spell {
    name: String,
    rank: i8,
    description: String,
}

async fn list_folder(
    client: &mut reqwest::Client,
    url: &str,
    type_filter: &str,
) -> reqwest::Result<Vec<String>> {
    #[derive(Debug, serde::Deserialize)]
    struct FileEntry {
        #[serde(rename = "type")]
        ty: String,
        name: String,
    }

    let resp = client.get(url).send().await?;
    let data: Vec<FileEntry> = resp.json().await?;
    let dirs = data
        .into_iter()
        .filter(|entry| entry.ty == type_filter)
        .map(|entry| entry.name)
        .collect();
    Ok(dirs)
}

async fn list_spell_folders(
    client: &mut reqwest::Client,
) -> reqwest::Result<Vec<String>> {
    list_folder(client, BASE_URL, "dir").await
}

async fn list_spells_in_folder(
    client: &mut reqwest::Client,
    folder: &str,
) -> reqwest::Result<Vec<String>> {
    list_folder(client, &format!("{BASE_URL}/{folder}"), "file").await
}

async fn download_spell(
    client: &mut reqwest::Client,
    folder: &str,
    name: &str,
) -> Result<Spell, String> {
    #[derive(Debug, serde::Deserialize)]
    struct Response {
        /// Base64 encoded text content of the file.
        content: String,
        encoding: String,
    }

    #[derive(Debug, serde::Deserialize)]
    struct RespSpellVal<T> {
        value: T,
    }

    #[derive(Debug, serde::Deserialize)]
    struct RespSpellDuration {
        sustained: String,
        value: String,
    }

    #[derive(Debug, serde::Deserialize)]
    struct RespSpellTraits {
        rarity: String,
        traditions: Vec<String>,
        value: Vec<String>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct RespSpellSystem {
        description: RespSpellVal<String>,
        level: RespSpellVal<i8>,
        range: RespSpellVal<String>,
        target: RespSpellVal<String>,
        time: RespSpellVal<String>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct RespSpell {
        name: String,
        system: RespSpellSystem,

        #[serde(rename = "type")]
        ty: String,
    }

    let resp: Response = client
        .get(format!("{BASE_URL}/{folder}/{name}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    debug_assert_eq!(resp.encoding, "base64");

    let file_bytes = base64::prelude::BASE64_STANDARD
        .decode(resp.content.replace("\n", ""))
        .map_err(|e| e.to_string())?;

    let spell: RespSpell =
        serde_json::de::from_slice(&file_bytes).map_err(|e| e.to_string())?;

    Ok(Spell {
        name: spell.name,
        rank: spell.system.level.value,
        description: spell.system.description.value,
    })
}

#[tokio::test]
async fn test() {
    let mut client = reqwest::Client::builder()
        .user_agent("chshtui/0.0.1")
        .build()
        .unwrap();
    let folders = list_spell_folders(&mut client).await.unwrap();
    let folder = &folders[5];
    let spells = list_spells_in_folder(&mut client, folder).await.unwrap();
    let spell = &spells[7];
    println!(
        "{:?}",
        download_spell(&mut client, folder, spell).await.unwrap()
    );
    panic!();
}
