use std::env;
use std::fs::{create_dir, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

pub fn setup_cert() -> Result<Vec<u8>> {
    let cert_loc = get_appdata_dir().join("verm-cert.pfx");
    if !Path::exists(&cert_loc) {
        generate_cert(cert_loc.clone())
    }
    let mut file = File::open(cert_loc)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn generate_cert(path: PathBuf) {
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(generate_shell_script(path))
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        println!("Script executed successfully.");
    } else {
        eprintln!("Script execution failed.");
        eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn generate_shell_script(path: PathBuf) -> String {
    format!(r#"Add-Type -AssemblyName System.Windows.Forms
# Remove old certificate from store if it exists
Get-ChildItem Cert:\LocalMachine\Root | Where-Object {{ $_.Subject -match 'Vermintwitch' }} | Remove-Item

# Generate a self-signed certificate and move it to trusted root certificate store
$cert = New-SelfSignedCertificate -Subject 'Vermintwitch' -DnsName 'api.twitch.tv' -CertStoreLocation 'Cert:\LocalMachine\MY'
Move-Item -Path ('Cert:\LocalMachine\My\' + $cert.Thumbprint) -Destination 'Cert:\LocalMachine\Root'

# Export private key to data dir
$pwd = ConvertTo-SecureString -String 'Testing' -Force -AsPlainText
Export-PfxCertificate -Cert $cert -Password $pwd -FilePath {}
"#, path.to_str().unwrap())
}

fn get_appdata_dir() -> PathBuf {
    let dir = env::var("APPDATA").expect("APPDATA environment variable not set.");
    let path = PathBuf::from(dir).join("Vermintwitch");
    if !Path::exists(&path) {
        create_dir(&path).expect("Could not create vermintwitch data dir.");
    }
    path
}