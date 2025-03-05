# rust-cni 
This is the CNI plugin impl by rust for container-runtime create CNI network.



## requirements
* Install cni plugin in /opt/cni/bin
* Prepare cni config in /etc/cni/net.d

## Test
```
## Run test
should as root user
```bash
cargo test --test it_test --  --test-threads=1 --nocapture
```

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
    let mut cni = Libcni::default();
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


## License
This project is licensed under the Apache License 2.0. See the LICENSE file for details.

## references
* Containerd cni plugins （https://github.com/containerd/go-cni）
* cni-rs (https://github.com/divinerapier/cni-rs)


## Contributing
Contributions are welcome! Please open an issue or submit a pull request if you have any improvements or bug fixes.

For more detailed information, please refer to the source code and documentation.