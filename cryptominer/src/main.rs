use tokio::process::Command;
use std::fs::write;

const XMR_WALLET: &str = "Wallet-adresse";
const POOL_URL: &str = "pool.supportxmr.com:3333";
const WORKER_NAME: &str = "my-miner";

#[tokio::main]
async fn main() {
    install_dependencies().await;
    setup_xmrig().await;
    create_systemd_service().await;
    start_systemd_service().await;
}

async fn run_cmd(command: &str) {
    println!("> {}", command);
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .expect("Commande échouée.")
        .wait()
        .await
        .expect("Erreur pendant l'exécution.");
}

async fn install_dependencies() {
    println!("[+] Dépendances");
    run_cmd("apt-get update").await;
    run_cmd("apt-get install -y git build-essential cmake libuv1-dev libssl-dev libhwloc-dev cargo").await;
}

async fn setup_xmrig() {
    println!("[+] Setup xmrig");
    run_cmd("git clone https://github.com/xmrig/xmrig.git").await;
    run_cmd("mkdir -p xmrig/build && cd xmrig/build && cmake .. && make -j$(nproc)").await;
}

async fn create_systemd_service() {
    println!("[+] Création service");
    let content = format!(
        "[Unit]
Description=XMRig Miner
After=network.target

[Service]
ExecStart=/root/xmrig/build/xmrig -o {POOL} -u {WALLET} -k --rig-id {WORKER}
Restart=always

[Install]
WantedBy=multi-user.target
",
        POOL = POOL_URL,
        WALLET = XMR_WALLET,
        WORKER = WORKER_NAME
    );

    write("/etc/systemd/system/xmrig.service", content).expect("Écriture échouée");
}

async fn start_systemd_service() {
    println!("[+] Démarrage service");
    run_cmd("systemctl daemon-reexec").await;
    run_cmd("systemctl daemon-reload").await;
    run_cmd("systemctl enable xmrig").await;
    run_cmd("systemctl start xmrig").await;
}
