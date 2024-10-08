// Copyright (c) 2024 https://github.com/divinerapier/cni-rs
use std::{
    fs::{self, File},
    io::{BufReader, Read},
};

use super::{
    api::NetworkConfigList,
    types::{NetConf, NetworkConfig},
};

pub struct ConfigFile {}

impl ConfigFile {
    pub fn config_files(dir: String, extensions: Vec<String>) -> Result<Vec<String>, String> {
        let mut conf_files = Vec::default();
        let dir = fs::read_dir(dir).unwrap();

        for entry in dir {
            match entry {
                Ok(file) => {
                    let file_path = file.path();
                    let file_ext = file_path.extension().unwrap();
                    if extensions.contains(&file_ext.to_string_lossy().to_string()) {
                        conf_files.push(file.path().to_string_lossy().to_string());
                    }
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
        Ok(conf_files)
    }

    pub fn config_from_bytes(datas: &[u8]) -> Result<NetworkConfigList, String> {
        let ncmaps: serde_json::Value = serde_json::from_slice(datas).unwrap();
        let name = ncmaps.get("name").unwrap().as_str().unwrap().to_string();
        let version = ncmaps
            .get("cniVersion")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let mut disabel_check = false;
        if let Some(check) = ncmaps.get("disableCheck") {
            disabel_check = check.as_bool().unwrap();
        }

        let mut ncflist = NetworkConfigList::default();
        let mut all_plugins = Vec::new();
        if let Some(plugins) = ncmaps.get("plugins") {
            let plugins_arr = plugins.as_array().unwrap();
            for plugin in plugins_arr {
                let string_plugin = plugin.to_string();
                let plg_bytes = string_plugin.as_bytes().to_vec();
                let tmp: NetConf = serde_json::from_str(&string_plugin).unwrap();
                all_plugins.push(NetworkConfig {
                    network: tmp,
                    bytes: plg_bytes,
                })
            }
        }

        ncflist.name = name;
        ncflist.cni_version = version;
        ncflist.bytes = datas.to_vec();
        ncflist.disable_check = disabel_check;
        ncflist.plugins = all_plugins;
        Ok(ncflist)
    }

    pub fn read_configlist_file(file_path: String) -> Option<NetworkConfigList> {
        let file = File::open(file_path).unwrap();
        let mut file_bytes = Vec::default();
        let mut reader = BufReader::new(file);
        let _ = reader.read_to_end(&mut file_bytes);

        let ncflist = Self::config_from_bytes(&file_bytes).unwrap();
        Some(ncflist)
    }
}
