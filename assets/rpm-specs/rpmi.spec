%bcond check 1

# prevent library files from being installed
%global cargo_install_lib 0

Name:            rpmi
Version:         @@TAG@@
Release:         %autorelease
Summary:         Simple graphical utility that installs/upgrades/removes .rpm files built with Rust and EGUI.

License:         MIT

URL:             https://github.com/arabianq/rpmi
Source:          https://github.com/arabianq/rpmi/archive/refs/tags/%{version}.tar.gz

BuildRequires: rust
BuildRequires: cargo

%global _description %{expand:
Simple graphical utility that installs/upgrades/removes .rpm files built with Rust and EGUI.}

%description %{_description}

%prep
%autosetup -n rpmi-%{version} -p1

%build
cargo build --release --locked

%install
install -Dm755 target/release/rpmi %{buildroot}%{_bindir}/rpmi
install -Dm644 assets/rpmi.desktop %{buildroot}%{_datadir}/applications/rpmi.desktop


%files
%license LICENSE
%doc README.md
%{_bindir}/rpmi
%{_datadir}/applications/rpmi.desktop

%changelog
%autochangelog