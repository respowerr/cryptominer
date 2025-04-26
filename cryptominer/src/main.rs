use tokio::process::Command;

#[tokio::main]
async fn main() {}

async fn run(command: &str) {
    Command::new("sh")
    .arg("-c")
    .arg(command)
    .spawn()
    .expect("Commande échouée.")
    .wait()
    .await
    .expect("La commande a échoué pendant l'attente.");
    }

async fn install_dependencies() {
    println!("== Installation des dépendances en cours... ==");
    run_cmd("apt-get update").await;
    run_cmd(&[
        "apt-get",
        "install",
        "-y",
        "git",
        "build-essential",
        "cmake",
        "libuv1-dev",
        "libssl-dev",
        "libhwloc-dev",
        "cargo"
    ]).await;
}


