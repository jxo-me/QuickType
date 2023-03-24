use crate::{
    app::{fs_extra},
    utils,
  };
  use log::{error, info};
  use regex::Regex;
  use std::{collections::HashMap, fs, path::PathBuf, vec};
  use tauri::{command};
  use walkdir::WalkDir;
  
  
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FileMetadata {
  pub name: String,
  pub ext: String,
  pub created: u64,
  pub id: String,
}

#[tauri::command]
pub fn get_download_list(pathname: &str) -> (Vec<serde_json::Value>, PathBuf) {
  info!("get_download_list: {}", pathname);
  let download_path = utils::app_root().join(PathBuf::from(pathname));
  let content = fs::read_to_string(&download_path).unwrap_or_else(|err| {
    error!("download_list: {}", err);
    fs::write(&download_path, "[]").unwrap();
    "[]".to_string()
  });
  let list = serde_json::from_str::<Vec<serde_json::Value>>(&content).unwrap_or_else(|err| {
    error!("download_list_parse: {}", err);
    vec![]
  });

  (list, download_path)
}

#[command]
pub fn download_list(pathname: &str, dir: &str, filename: Option<String>, id: Option<String>) {
  info!("download_list: {}", pathname);
  let data = get_download_list(pathname);
  let mut list = vec![];
  let mut idmap = HashMap::new();
  utils::vec_to_hashmap(data.0.into_iter(), "id", &mut idmap);

  for entry in WalkDir::new(utils::app_root().join(dir))
    .into_iter()
    .filter_entry(|e| !utils::is_hidden(e))
    .filter_map(|e| e.ok())
  {
    let metadata = entry.metadata().unwrap();
    if metadata.is_file() {
      let file_path = entry.path().display().to_string();
      let re = Regex::new(r"(?P<id>[\d\w]+).(?P<ext>\w+)$").unwrap();
      let caps = re.captures(&file_path).unwrap();
      let fid = &caps["id"];
      let fext = &caps["ext"];

      let mut file_data = FileMetadata {
        name: fid.to_string(),
        id: fid.to_string(),
        ext: fext.to_string(),
        created: fs_extra::system_time_to_ms(metadata.created()),
      };

      if idmap.get(fid).is_some() {
        let name = idmap.get(fid).unwrap().get("name").unwrap().clone();
        match name {
          serde_json::Value::String(v) => {
            file_data.name = v.clone();
            v
          }
          _ => "".to_string(),
        };
      }

      if filename.is_some() && id.is_some() {
        if let Some(ref v) = id {
          if fid == v {
            if let Some(ref v2) = filename {
              file_data.name = v2.to_string();
            }
          }
        }
      }
      list.push(serde_json::to_value(file_data).unwrap());
    }
  }

  // dbg!(&list);
  list.sort_by(|a, b| {
    let a1 = a.get("created").unwrap().as_u64().unwrap();
    let b1 = b.get("created").unwrap().as_u64().unwrap();
    a1.cmp(&b1).reverse()
  });

  fs::write(data.1, serde_json::to_string_pretty(&list).unwrap()).unwrap();
}