use serde::{Deserialize, Serialize};
/* Started by AICoder, pid:3f64b657d3k8d1a1407d0b393078c9075391fe43 */

use super::CNIError;
/* Ended by AICoder, pid:3f64b657d3k8d1a1407d0b393078c9075391fe43 */
use super::exec::RawExec;
use crate::libcni::exec::{Exec, ExecArgs};
use crate::libcni::result::result100;
use crate::libcni::result::{APIResult, ResultCNI};
use crate::libcni::types::NetworkConfig;
use std::collections::HashMap;
pub trait CNI {
    fn add_network_list(
        &self,
        net: NetworkConfigList,
        rt: RuntimeConf,
    ) -> ResultCNI<Box<dyn APIResult>>;

    fn check_network_list(&self, net: NetworkConfigList, rt: RuntimeConf) -> ResultCNI<()>;

    fn delete_network_list(&self, net: NetworkConfigList, rt: RuntimeConf) -> ResultCNI<()>;

    fn get_network_list_cached_result(
        &self,
        net: NetworkConfigList,
        rt: RuntimeConf,
    ) -> ResultCNI<Box<dyn APIResult>>;

    fn add_network(
        &self,
        name: String,
        cni_version: String,
        net: NetworkConfig,
        prev_result: Option<Box<dyn APIResult>>,
        rt: RuntimeConf,
    ) -> ResultCNI<Box<dyn APIResult>>;

    fn check_network(&self, net: NetworkConfigList, rt: RuntimeConf) -> ResultCNI<()>;
    fn delete_network(
        &self,
        name: String,
        cni_version: String,
        net: NetworkConfig,
        rt: RuntimeConf,
    ) -> ResultCNI<()>;

    fn get_network_cached_result(
        &self,
        net: NetworkConfig,
        rt: RuntimeConf,
    ) -> ResultCNI<Box<dyn APIResult>>;

    fn get_network_cached_config(
        &self,
        net: NetworkConfig,
        rt: RuntimeConf,
    ) -> ResultCNI<(Vec<u8>, RuntimeConf)>;

    fn validate_network_list(&self, net: NetworkConfigList) -> ResultCNI<Vec<String>>;

    fn validate_network(&self, net: NetworkConfig) -> ResultCNI<Vec<String>>;
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct NetworkConfigList {
    pub name: String,
    pub cni_version: String,
    pub disable_check: bool,
    pub plugins: Vec<NetworkConfig>,
    pub bytes: Vec<u8>,
}

#[derive(Default, Clone)]
pub struct RuntimeConf {
    pub container_id: String,
    pub net_ns: String,
    pub if_name: String,
    pub args: Vec<[String; 2]>,
    pub capability_args: HashMap<String, String>,
    pub cache_dir: String,
}
#[derive(Default)]
pub struct CNIConfig {
    pub path: Vec<String>,
    pub exec: RawExec,
    pub cache_dir: String,
}

impl CNIConfig {
    // fn cache_add(
    //     &self,
    //     type_result: Box<dyn APIResult>,
    //     config: Vec<u8>,
    //     netname: String,
    //     rt: &RuntimeConf,
    // ) -> Result<(), String> {
    //     Ok(())
    // }

    fn build_new_config(
        &self,
        name: String,
        cni_version: String,
        orig: &NetworkConfig,
        prev_result: Option<Box<dyn APIResult>>,
        _rt: &RuntimeConf,
    ) -> Result<NetworkConfig, String> {
        let mut json_object = json::parse(String::from_utf8_lossy(&orig.bytes).as_ref()).unwrap();

        json_object.insert("name", name).unwrap();
        json_object.insert("cniVersion", cni_version).unwrap();

        if let Some(prev_result) = prev_result {
            json_object
                .insert("prevResult", prev_result.get_json())
                .unwrap();
        }
        let new_bytes = json_object.dump().as_bytes().to_vec();
        //need to update RutimeConfig
        Ok(NetworkConfig {
            network: serde_json::from_slice(&new_bytes).unwrap(),
            bytes: new_bytes,
        })
    }
}

impl CNI for CNIConfig {
    fn add_network_list(
        &self,
        net: NetworkConfigList,
        rt: RuntimeConf,
    ) -> ResultCNI<Box<dyn APIResult>> {
        let mut r = None;

        for x in net.plugins {
            let r1 = self.add_network(
                net.name.clone(),
                net.cni_version.clone(),
                x.clone(),
                r,
                rt.clone(),
            )?;
            r = Some(r1);
            //add r to cache
            // self.cacheAdd();
        }
        match r {
            Some(r) => Ok(r),
            None => Err(Box::new(CNIError::ExecuteError("()".to_string()))),
        }
    }

    fn check_network_list(&self, net: NetworkConfigList, _rt: RuntimeConf) -> ResultCNI<()> {
        net.plugins.into_iter().for_each(|_x| {});
        Ok(())
    }

    fn delete_network_list(&self, net: NetworkConfigList, rt: RuntimeConf) -> ResultCNI<()> {
        net.plugins.into_iter().try_for_each(|x| {
            self.delete_network(net.name.clone(), net.cni_version.clone(), x, rt.clone())
        })?;
        Ok(())
    }

    fn get_network_list_cached_result(
        &self,
        _net: NetworkConfigList,
        _rt: RuntimeConf,
    ) -> ResultCNI<Box<dyn APIResult>> {
        todo!()
    }

    fn add_network(
        &self,
        name: String,
        cni_version: String,
        net: NetworkConfig,
        prev_result: Option<Box<dyn APIResult>>,
        rt: RuntimeConf,
    ) -> ResultCNI<Box<dyn APIResult>> {
        //ensureexec todo!()
        let plugin_path = self
            .exec
            .find_in_path(net.network._type.clone(), self.path.clone())
            .unwrap();

        let environ = ExecArgs {
            command: "ADD".to_string(),
            containerd_id: rt.container_id.clone(),
            netns: rt.net_ns.clone(),
            plugin_args: rt.args.clone(),
            plugin_args_str: String::default(),
            ifname: rt.if_name.clone(),
            path: self.path[0].clone(),
        };

        let new_conf = self.build_new_config(name, cni_version, &net, prev_result, &rt);
        if let Ok(new_conf) = new_conf {
            let r_result =
                self.exec
                    .exec_plugins(plugin_path, &new_conf.bytes, environ.to_env())?;
            let pre_result_json: result100::Result = serde_json::from_slice(&r_result)
                .map_err(|e| e.to_string())
                .unwrap_or_else(|e| {
                    println!("Failed to parse JSON: {}", e);
                    result100::Result::default()
                });
            println!("cni_result {}", pre_result_json.get_json());
            return Ok(Box::new(pre_result_json));
        }
        Err(Box::new(CNIError::ExecuteError("()".to_string())))
    }

    fn check_network(&self, _net: NetworkConfigList, _rt: RuntimeConf) -> ResultCNI<()> {
        todo!()
    }
    fn delete_network(
        &self,
        name: String,
        cni_version: String,
        net: NetworkConfig,
        rt: RuntimeConf,
    ) -> ResultCNI<()> {
        let environ = ExecArgs {
            command: "DEL".to_string(),
            containerd_id: rt.container_id,
            netns: rt.net_ns,
            plugin_args: rt.args,
            plugin_args_str: String::default(),
            ifname: rt.if_name,
            path: self.path[0].clone(),
        };

        let plugin_path = self
            .exec
            .find_in_path(net.network._type, self.path.clone())
            .unwrap();

        // add network name and version
        let mut json_object = json::parse(String::from_utf8_lossy(&net.bytes).as_ref()).unwrap();
        json_object.insert("name", name).unwrap();
        json_object.insert("cniVersion", cni_version).unwrap();
        let new_stdin_data = json_object.dump().as_bytes().to_vec();

        self.exec
            .exec_plugins(plugin_path, &new_stdin_data, environ.to_env())?;

        Ok(())
    }

    fn get_network_cached_result(
        &self,
        _net: NetworkConfig,
        _rt: RuntimeConf,
    ) -> ResultCNI<Box<dyn APIResult>> {
        // let net_name = net.network.name.clone();

        todo!()
    }

    fn get_network_cached_config(
        &self,
        _net: NetworkConfig,
        _rt: RuntimeConf,
    ) -> ResultCNI<(Vec<u8>, RuntimeConf)> {
        todo!()
    }

    fn validate_network_list(&self, _net: NetworkConfigList) -> ResultCNI<Vec<String>> {
        todo!()
    }

    fn validate_network(&self, _net: NetworkConfig) -> ResultCNI<Vec<String>> {
        todo!()
    }
}
