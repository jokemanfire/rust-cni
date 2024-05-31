use std::{fs::File, io};

use cni::cni::Libcni;

use netns_rs::NetNs;

use nix::sched::setns;

fn create_ns() -> Result<NetNs, String> {
    let pid = std::process::id();
    let ns = NetNs::new("ns_name").unwrap();
    let fd_name = format!("/proc/{}/ns/net", pid);
    let fd = File::open(fd_name).unwrap();
    let path_ns = ns.path();
    let _ = setns(fd, nix::sched::CloneFlags::CLONE_NEWNET);
    println!("{:?}", path_ns.to_string_lossy().to_string());
    Ok(ns)
}

fn main() {
    let ns = create_ns().unwrap();
    let mut cni = Libcni::new();
    cni.load_default_conf();
    let _ = cni.add_lo_network();

    let id = "test".to_string();
    let path = ns.path().to_string_lossy().to_string();
    let _ = cni.setup(id.clone(), path.clone());

    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("error");

    println!("try to remove --------------------");
    let _ = cni.remove(id.clone(), path.clone());
    let _ = ns.remove();
}
