pub mod cni;
pub mod libcni;
pub mod namespace;
#[cfg(test)]
pub mod test {

    use nix::unistd::{setuid, Uid};

    use crate::cni;

    #[test]
    fn test_cni_add_remove() {
        // run_as_root();
        let mut cni = cni::Libcni::new();
        cni.load_default_conf();

        let pid = std::process::id();
        let path = format!("/proc/{}/ns/net", pid);
        let id = "test".to_string();
        let _ = cni.setup(id.clone(), path.clone());

        let _ = cni.remove(id.clone(), path.clone());
    }
}
