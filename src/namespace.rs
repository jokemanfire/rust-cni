use std::{collections::HashMap, sync::Arc};

use crate::libcni::{
    self,
    api::{RuntimeConf, CNI},
};
pub struct Network {
    pub cni: Arc<Box<dyn CNI + Send + Sync>>,
    pub config: libcni::api::NetworkConfigList,
    pub ifname: String,
}

impl Network {
    pub fn attach(&self, ns: &Namespace) -> Result<(), String> {
        let _ = self
            .cni
            .add_network_list(self.config.clone(), ns.config(self.ifname.clone()));
        Ok(())
    }
    pub fn remove(&self, ns: &Namespace) -> Result<(), String> {
        let _ = self
            .cni
            .delete_network_list(self.config.clone(), ns.config(self.ifname.clone()));
        Ok(())
    }
    pub fn check(&self, ns: &Namespace) -> Result<(), String> {
        let _ = self
            .cni
            .check_network_list(self.config.clone(), ns.config(self.ifname.clone()));
        Ok(())
    }
}
#[derive(Clone, Default)]
pub struct Namespace {
    id: String,
    path: String,
    capability_args: HashMap<String, String>,
    args: HashMap<String, String>,
}

impl Namespace {
    pub fn new(id: String, path: String) -> Self {
        Self {
            id,
            path,
            capability_args: HashMap::default(),
            args: HashMap::default(),
        }
    }

    pub fn config(&self, ifname: String) -> libcni::api::RuntimeConf {
        let args = self
            .args
            .iter()
            .map(|(key, val)| [key.clone(), val.clone()])
            .collect();
        RuntimeConf {
            container_id: self.id.clone(),
            net_ns: self.path.clone(),
            if_name: ifname,
            args,
            capability_args: self.capability_args.clone(),
            cache_dir: String::default(),
        }
    }
}
