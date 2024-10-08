// Copyright (c) 2024 https://github.com/divinerapier/cni-rs
use serde::{Deserialize, Serialize};

use crate::libcni::result::ResultCNI;
use crate::libcni::CNIError;
use std::process::Stdio;
use std::{collections::HashMap, io::Write};

#[derive(Default, Serialize, Deserialize)]
pub struct ExecArgs {
    pub(crate) command: String,
    pub(crate) containerd_id: String,
    pub(crate) netns: String,
    pub(crate) plugin_args: Vec<[String; 2]>,
    pub(crate) plugin_args_str: String,
    pub(crate) ifname: String,
    pub(crate) path: String,
}

impl ExecArgs {
    pub fn to_env(&self) -> Vec<String> {
        let mut result_env = Vec::default();
        //get ose env
        std::env::set_var("CNI_COMMAND", self.command.clone());
        std::env::set_var("CNI_CONTAINERID", self.containerd_id.clone());
        std::env::set_var("CNI_NETNS", self.netns.clone());
        std::env::set_var("CNI_ARGS", self.plugin_args_str.clone());
        std::env::set_var("CNI_IFNAME", self.ifname.clone());
        std::env::set_var("CNI_PATH", self.path.clone());

        for (k, v) in std::env::vars() {
            result_env.push((k + "=" + &v).to_string());
        }
        result_env
    }
}
pub trait Exec {
    fn exec_plugins(
        &self,
        plugin_path: String,
        stdin_data: &[u8],
        environ: Vec<String>,
    ) -> super::ResultCNI<Vec<u8>>;
    fn find_in_path(&self, plugin: String, paths: Vec<String>) -> ResultCNI<String>;
    fn decode(&self, data: &[u8]) -> ResultCNI<()>;
}

#[derive(Default)]
pub struct RawExec {}

impl Exec for RawExec {
    fn exec_plugins(
        &self,
        plugin_path: String,
        stdin_data: &[u8],
        environ: Vec<String>,
    ) -> ResultCNI<Vec<u8>> {
        let envs: HashMap<String, String> = environ
            .iter()
            .map(|key| key.split('=').collect::<Vec<&str>>())
            .map(|kv| (kv[0].to_string(), kv[1].to_string()))
            .collect();

        println!(
            "send plugin to {:?}\n\n",
            String::from_utf8(stdin_data.to_vec()).unwrap()
        );

        let mut plugin_cmd = std::process::Command::new(plugin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(envs)
            .spawn()
            .expect("start plugin error");

        let mut stdin = plugin_cmd.stdin.take().unwrap();
        stdin.write_all(stdin_data).unwrap();
        //stdin over close fifo
        drop(stdin);

        let output = plugin_cmd.wait_with_output().unwrap();

        // println!("{:?}",output.stdout);
        let std_out_json: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_default();
        if std_out_json.get("code").is_some() {
            let msg = String::from_utf8_lossy(&output.stdout.clone()).to_string();
            println!("error:{}", msg);
            return Err(Box::new(CNIError::ExecuteError(msg)));
        }

        Ok(output.stdout)
    }

    fn find_in_path(&self, plugin: String, paths: Vec<String>) -> ResultCNI<String> {
        if !paths.is_empty() {
            return Ok(paths[0].clone() + "/" + &plugin);
        }

        Ok(String::default())
    }

    fn decode(&self, _data: &[u8]) -> ResultCNI<()> {
        todo!()
    }
}

// struct BufferedStdin<'a> {
//     buf: &'a [u8],
// }

// impl<'a> BufferedStdin<'a> {
//     fn new(buf: &'a [u8]) -> BufferedStdin {
//         BufferedStdin { buf }
//     }
// }

// impl<'a> Into<Stdio> for BufferedStdin<'a> {
//     fn into(self) -> Stdio {
//         todo!()
//     }
// }
