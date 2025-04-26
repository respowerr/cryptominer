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



