# RPMI: Graphical Utility for Managing RPM Packages

![rpmi-screenshot](./assets/screenshot.png)

RPMI is a simple graphical utility, developed in Rust using EGUI, designed for installing, upgrading, and removing RPM packages on Linux operating systems. It provides an intuitive interface for interacting with packages, using `dnf` as its backend.

## Features

*   **Install RPM Packages:** Easily install new RPM packages.
*   **Upgrade RPM Packages:** Upgrade existing packages to newer versions.
*   **Remove RPM Packages:** Remove unwanted packages from your system.
*   **Graphical User Interface:** Intuitive user interface built with EGUI.
*   **Detailed Package Information:** View package name, version, architecture, size, summary, URL, and description before installation.
*   **Process Logging:** Monitor the progress of `dnf` operations in real-time.

## Installation

### Cargo
You can install `egui_rpm_installer` directly from crates.io using `cargo`:

```bash
cargo install egui_rpm_installer
```

### COPR
For Fedora and RHEL-based systems, you can install RPMI from the COPR repository:

```bash
sudo dnf copr enable arabianq/rpmi
sudo dnf install rpmi
```

### Pre-built Binaries
You can download pre-built binaries for various platforms from the [GitHub Releases page](https://github.com/arabianq/rpmi/releases).

## Build and Run

### Dependencies

To build and run RPMI, you will need:

*   Rust (version 1.70 or higher)
*   `dnf` (for package management)
*   `pkexec` (for executing commands with elevated privileges)

### Build

Clone the repository and build the project:

```bash
git clone https://github.com/arabianq/rpmi.git
cd rpmi
cargo build --release
```

The executable will be located at `target/release/rpmi`.

### Run

You can run RPMI by passing the path to one or more RPM files:

```bash
./target/release/rpmi /path/to/your/package.rpm
```

If you pass multiple files, a separate RPMI instance will be launched for each.

## Usage

1.  **Launch:** Start RPMI by specifying the path to an RPM file.
2.  **Package Information:** The RPMI window will display detailed information about the package.
3.  **Action:** Depending on the package's state (new, older version, newer version of installed), you will be offered "Install", "Upgrade", or "Remove" buttons.
4.  **Execute:** Click the appropriate button to begin the process. The execution log will be displayed in the window.
5.  **Completion:** After the process is complete, you can close the application.

## License

This project is distributed under the MIT License. See the `LICENSE` file for details.