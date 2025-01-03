# This is the CNI plugin impl by rust for container create CNI network



## todo
* Need Cached in CNI
* Need Tests
* Need Validate in CNI
* Wrap as lib

## example

```Rust
use std::{fs::File, io};
use rust_cni::cni::Libcni;
use netns_rs::NetNs;
use nix::sched::setns;

fn create_ns() -> Result<NetNs, String> {
    let ns = NetNs::new("ns_name").unwrap();
    println!("{:?}", ns.path());
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


```


## ref
* Containerd cni plugins （https://github.com/containerd/go-cni）
* cni-rs (https://github.com/divinerapier/cni-rs)
