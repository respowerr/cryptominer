use std::{fs::write, path::Path};
use tokio::{process::Command, time::{sleep, Duration}};
use std::error::Error;

const XMR_REPO: &str      = "https://github.com/xmrig/xmrig.git";
const XMR_DIR: &str       = "/opt/xmrig";
const BUILD_DIR: &str     = "/opt/xmrig/build";
const XMR_WALLET: &str    = "Wallet-adresse";
const POOL_URL: &str      = "pool.supportxmr.com:3333";
const WORKER_NAME: &str   = "my-miner";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    install_dependencies().await?;
    cleanup_previous_install().await?;
    setup_xmrig().await?;
    create_systemd_service().await?;
    start_systemd_service().await?;
    println!("\n✔️  Tout est prêt !");

    Ok(())
}

async fn run_cmd(cmd: &str) -> Result<(), Box<dyn Error>> {
    println!("> {}", cmd);
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()?;
    let status = child.wait().await?;
    if !status.success() {
        Err(format!("La commande a échoué ({})", cmd))?
    }
    Ok(())
}

async fn install_dependencies() -> Result<(), Box<dyn Error>> {
    println!("[+] Installation des dépendances...");
    run_cmd("apt-get update").await?;
    run_cmd("apt-get install -y git build-essential cmake libuv1-dev libssl-dev libhwloc-dev cargo").await?;
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
Description=XMRig Miner Service
After=network.target

[Service]
Type=simple
ExecStart={build}/xmrig -o {pool} -u {wallet} -k --rig-id {worker}
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

    write("/etc/systemd/system/xmrig.service", service_content)?;
    Ok(())
}

async fn start_systemd_service() -> Result<(), Box<dyn Error>> {
    println!("[+] Activation et démarrage du service xmrig");
    run_cmd("systemctl daemon-reload").await?;
    run_cmd("systemctl enable xmrig").await?;
    run_cmd("systemctl start xmrig").await?;
    Ok(())
}
