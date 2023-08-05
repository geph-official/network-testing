use std::process::{exit, Command};

pub fn install_on_machine(addr: &str) -> anyhow::Result<()> {
    if !check_binary_installed("geph4-client") {
        install_geph4_client();
    }

    check_and_install_dependencies();

    if let Some((ip, port)) = split_ip_port(addr) {
        transfer_geph4_client(&ip, &port);
    }

    Ok(())
}

fn split_ip_port(input: &str) -> Option<(String, String)> {
    input
        .split_once(':')
        .map(|(hostname, port)| (hostname.to_string(), port.to_string()))
}

fn check_binary_installed(binary: &str) -> bool {
    let output = Command::new("which")
        .arg(binary)
        .output()
        .expect("Failed to execute 'which' command.");

    let output_str = String::from_utf8_lossy(&output.stdout);
    !output_str.trim().is_empty()
}

fn install_geph4_client() {
    if !check_binary_installed("geph4-client") {
        let status = Command::new("cargo")
            .arg("install")
            .arg("--git")
            .arg("https://github.com/geph-official/geph4-client")
            .arg("--branch")
            .arg("master")
            .arg("--features")
            .arg("binary")
            .status()
            .expect("Failed to execute 'cargo install' command.");

        if !status.success() {
            eprintln!("Failed to install geph4-client.");
            exit(1);
        }
    }
}

fn check_and_install_dependencies() {
    let dependencies = vec!["git", "rsync"];

    for dependency in dependencies {
        if !check_binary_installed(dependency) {
            let status = Command::new("apt-get")
                .arg("install")
                .arg("-y")
                .arg(dependency)
                .status()
                .expect("Failed to execute 'apt-get install' command.");

            if !status.success() {
                eprintln!("Failed to install {}.", dependency);
                exit(1);
            }
        }
    }
}

fn transfer_geph4_client(ip: &str, ssh_port: &str) {
    let status = Command::new("rsync")
        .arg("-avz")
        .arg("--progress")
        .arg("-e")
        .arg(format!("'ssh -p {}'", ssh_port))
        .arg("./target/x86_64-unknown-linux-musl/release/geph4-client")
        .arg(format!("root@{}:/usr/local/bin", ip))
        .status()
        .expect("Failed to execute 'rsync' command.");

    if !status.success() {
        eprintln!(
            "Failed to transfer geph4-client binary to {}:{}.",
            ip, ssh_port
        );
        exit(1);
    }
}
