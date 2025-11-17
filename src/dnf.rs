use std::{
    io::{BufRead, BufReader},
    os::unix::process::CommandExt,
    process::{Command, Stdio},
    sync::mpsc::{Receiver, channel},
    thread::{self, JoinHandle},
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DNFAction {
    Install,
    Upgrade,
    Remove,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PackageState {
    NewPackage,
    OldVersion,
    NewVersion(PackageEntry),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
    pub arch: String,
    pub version: String,
    pub release: String,
}

pub fn get_package_state(pkg: &rpm::Package) -> PackageState {
    let pkg_name = pkg.metadata.get_name().unwrap_or_default();
    let pkg_arch = pkg.metadata.get_arch().unwrap_or_default();
    let pkg_version = pkg.metadata.get_version().unwrap_or_default();
    let pkg_release = pkg.metadata.get_release().unwrap_or_default();

    let mut child = Command::new("/usr/bin/dnf")
        .args(["list", "--installed"])
        .stdout(Stdio::piped())
        .process_group(0)
        .spawn()
        .expect("Couldn't spawn child thread");

    let child_stdout = child.stdout.take().expect("Couldn't take stdout");

    for line in BufReader::new(child_stdout).lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() != 3 {
                continue;
            }

            let name_splitted: Vec<&str> = parts[0].split(".").collect();
            let (name, arch) = (name_splitted[0], name_splitted[1]);

            let version_prepared = parts[1]
                .split_once(':')
                .map(|(_epoch, version)| version)
                .unwrap_or(parts[1]);
            let version_splitted: Vec<&str> = version_prepared.split("-").collect();
            let (version, release) = (version_splitted[0], version_splitted[1]);

            if pkg_name.eq(name) && pkg_arch.eq(arch) {
                child.kill().unwrap();
                child.wait().unwrap();

                let package_entry = PackageEntry {
                    name: name.to_string(),
                    arch: arch.to_string(),
                    version: version.to_string(),
                    release: release.to_string(),
                };

                if pkg_version.eq(version) && pkg_release.eq(release) {
                    return PackageState::OldVersion;
                }
                return PackageState::NewVersion(package_entry);
            }
        }
    }

    child.kill().ok();
    child.wait().ok();

    return PackageState::NewPackage;
}

pub fn dnf_start_action(
    package_path: &str,
    action_type: DNFAction,
) -> (JoinHandle<()>, Receiver<String>) {
    let (tx, rx) = channel();

    let package_path = package_path.to_string().clone();

    let action_thread = thread::spawn(move || {
        let mut child = Command::new("/usr/bin/pkexec")
            .arg("--disable-internal-agent")
            .arg("/usr/bin/dnf")
            .arg(match action_type {
                DNFAction::Install => "install",
                DNFAction::Remove => "remove",
                DNFAction::Upgrade => "upgrade",
            })
            .args(["-y", &package_path])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .expect("Couldn't spawn child thread");

        let child_stdout = child.stdout.take().expect("Couldn't take stdout");
        let child_stderr = child.stderr.take().expect("Couldn't take stdout");
        let (stdout_tx, stdout_rx) = channel();
        let (stderr_tx, stderr_rx) = channel();

        let stdout_thread = thread::spawn(move || {
            let stdout_lines = BufReader::new(child_stdout).lines();
            for line in stdout_lines {
                if let Ok(line) = line {
                    stdout_tx.send(line).ok();
                }
            }
        });

        let stderr_thread = thread::spawn(move || {
            let stderr_lines = BufReader::new(child_stderr).lines();
            for line in stderr_lines {
                if let Ok(line) = line {
                    stderr_tx.send(line).ok();
                }
            }
        });

        while let Ok(None) = child.try_wait() {
            if let Ok(msg) = stdout_rx.try_recv() {
                tx.send(msg).ok();
            }
            if let Ok(msg) = stderr_rx.try_recv() {
                tx.send(msg).ok();
            }
        }

        stdout_thread.join().ok();
        stderr_thread.join().ok();
        child.kill().ok();
        child.wait().ok();
    });

    return (action_thread, rx);
}
