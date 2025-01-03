use libcni::{
    api::{CNIConfig, CNI},
    exec::RawExec,
    types::Config,
};
use std::sync::Arc;

use crate::{
    libcni,
    namespace::{Namespace, Network},
};

pub struct Libcni {
    config: Config,
    cni_interface: Arc<Box<dyn CNI + Send +Sync>>,
    network_count: i64,
    networks: Vec<Network>,
}
impl Default for Libcni {
    fn default() -> Self {
        Self::new()
    }
}

impl Libcni {
    pub fn load_default_conf(&mut self) {
        let extensions = vec![
            "conf".to_string(),
            "conflist".to_string(),
            "json".to_string(),
        ];
        let r =
            libcni::conf::ConfigFile::config_files(self.config.plugin_conf_dir.clone(), extensions)
                .unwrap();
        let mut networks = Vec::new();
        let mut cnt = 1;
        for configfile in r {
            if configfile.ends_with(".conflist") {
                let r = libcni::conf::ConfigFile::read_configlist_file(configfile).unwrap();
                networks.push(Network {
                    cni: self.cni_interface.clone(),
                    config: r,
                    ifname: self.config.prefix.clone() + &cnt.to_string(),
                });
            }
            cnt += 1;
            //todo! another file
        }
        self.networks = networks;
    }
    pub fn new() -> Self {
        Libcni {
            config: Config {
                plugin_dirs: vec!["/opt/cni/bin".to_string()],
                plugin_conf_dir: "/etc/cni/net.d".to_string(),
                plugin_max_conf_num: 1,
                prefix: "vethcni".to_string(),
            },
            cni_interface: Arc::new(Box::new(CNIConfig {
                path: vec!["/opt/cni/bin".to_string()],
                exec: RawExec::default(),
                cache_dir: String::default(),
            })),
            network_count: 1,
            networks: Vec::default(),
        }
    }

    pub fn load() -> Result<(), String> {
        todo!()
    }

    pub fn add_lo_network(&mut self) -> Result<(), String> {
        let datas = " {\"cniVersion\": \"0.3.1\",
            \"name\": \"cni-loopback\",
            \"plugins\": [{
              \"type\": \"loopback\"
            }] 
        }"
        .to_string();

        if let Ok(loconfig) = libcni::conf::ConfigFile::config_from_bytes(datas.as_bytes()) {
            self.networks.push(Network {
                cni: self.cni_interface.clone(),
                config: loconfig,
                ifname: "lo".to_string(),
            });
            return Ok(());
        }
        Err("can't add lo network".to_string())
    }
    pub fn status(&self) -> Result<(), String> {
        if self.networks.len() < self.network_count as usize {
            return Err("Initial error".to_string());
        }
        Ok(())
    }
    pub fn networks() {}
    pub fn setup(&self, id: String, path: String) -> Result<(), String> {
        //get status
        let status = self.status();
        if status.is_err() {
            return Err("error get status".to_string());
        }
        // get namespace
        let namespace = Namespace::new(id, path);
        // do attach networks
        self.attach_networks(&namespace);
        Ok(())
    }

    pub fn remove(&self, id: String, path: String) -> Result<(), String> {
        let status = self.status();
        if status.is_err() {
            return Err("error get status".to_string());
        }
        let namespace = Namespace::new(id, path);
        self.networks.iter().for_each(|net| {
            let _ = net.remove(&namespace);
        });
        Ok(())
    }

    pub fn getconfig() {}

    pub fn check() {}

    fn attach_networks(&self, ns: &Namespace) {
        self.networks.iter().for_each(|n| {
            let _ = n.attach(ns);
        });
    }
}
