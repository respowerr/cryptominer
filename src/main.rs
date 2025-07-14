use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::io::ErrorKind;
use tokio::process::Command;

const WALLET: &str = "4A2wLHnkMELKS4wWtNA6aPcAEpK3ZDPooZnmAW4sViq11JSus46Brngem55keyfKwcKG3udFiHvWjYvt3Y7F9aw629qTsja";
const NODE: &str = "121.98.149.60:18081";

const BINARY_URL: &str = "https://github.com/xmrig/xmrig/releases/download/v6.21.0/xmrig-6.21.0-linux-x64.tar.gz";
const ARCHIVE_NAME: &str = "xmrig.tar.gz";
const INSTALL_DIR: &str = "/opt/xmrig";
const BUILD_DIR: &str = "/opt/xmrig/build";
const BINARY_PATH: &str = "/opt/xmrig/build/xmrig";
const SERVICE_PATH: &str = "/etc/systemd/system/monero_miner.service";

#[tokio::main]
async fn main() {
    println!("[INFO] Initialisation du setup du mineur...");
    let result = full_setup().await;
    if result.is_err() {
        eprintln!("[ERREUR] Le setup a échoué.");
        std::process::exit(1);
    }
    println!("[INFO] Setup terminé avec succès.");

    println!("[INFO] Vérification que xmrig fonctionne...");
    if check_and_restart_xmrig().await.is_err() {
        eprintln!("[ERREUR] Échec lors de la vérification ou redémarrage de xmrig.");
    } else {
        println!("[INFO] xmrig est actif.");
    }
}

async fn full_setup() -> Result<(), ()> {
    println!("[INFO] Détection de la distribution Linux...");
    let distro = detect_distro().await?;
    println!("[INFO] Distribution détectée : {}", distro);

    println!("[INFO] Mise à jour du système...");
    update_system(&distro).await?;

    println!("[INFO] Installation des paquets requis...");
    install_packages(&distro).await?;

    println!("[INFO] Préparation de l'environnement...");
    prepare_environment().await?;

    println!("[INFO] Téléchargement du binaire précompilé de xmrig...");
    download_and_extract_binary().await?;

    println!("[INFO] Vérification du binaire...");
    confirm_binary().await?;

    println!("[INFO] Création du fichier de service systemd...");
    create_service_file().await?;

    println!("[INFO] Rechargement de systemd...");
    reload_systemd().await?;

    println!("[INFO] Activation du service...");
    enable_service().await?;

    println!("[INFO] Démarrage du service...");
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
        "debian" => run("apt-get install -y curl tar").await,
        "alpine" => run("apk add curl tar").await,
        "arch" => run("pacman -S --noconfirm curl tar").await,
        "rhel" => run("yum install -y curl tar").await,
        _ => Err(()),
    }
}

async fn prepare_environment() -> Result<(), ()> {
    if Path::new(INSTALL_DIR).exists() {
        run(&format!("rm -rf {}", INSTALL_DIR)).await?;
    }
    run(&format!("mkdir -p {}", BUILD_DIR)).await?;
    Ok(())
}

async fn download_and_extract_binary() -> Result<(), ()> {
    run(&format!("curl -L {} -o {}/{}", BINARY_URL, BUILD_DIR, ARCHIVE_NAME)).await?;
    run(&format!("tar -xvf {}/{} -C {} --strip-components=1", BUILD_DIR, ARCHIVE_NAME, BUILD_DIR)).await?;
    Ok(())
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
        Ok(s) if s.success() => Ok(()),
        _ => {
            println!("[INFO] xmrig est inactif. Redémarrage...");
            run("systemctl restart monero_miner.service").await
        }
    }
}

async fn run(cmd: &str) -> Result<(), ()> {
    println!("[CMD] Exécution : {}", cmd);
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            eprintln!("[ERREUR] Commande échouée : {}", cmd);
            Err(())
        },
    }
}
