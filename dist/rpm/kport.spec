%global debug_package %{nil}
%global __os_install_post %{nil}

Name:           kport
Version:        0.2.1
Release:        1%{?dist}
Summary:        ⚡ Ultra-fast CLI & TUI to hunt down and kill processes hogging network ports

License:        MIT OR Apache-2.0
URL:            https://neuralshyam.github.io/kport/
Source0:        https://github.com/neuralshyam/kport/releases/download/v%{version}/port-killer-x86_64-unknown-linux-gnu.tar.gz

ExclusiveArch:  x86_64

%description
kport is a blazing-fast, cross-platform CLI and interactive TUI written in Rust
to inspect, probe, and slay network port conflicts with zero friction.

%prep
%setup -q -c

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir}
install -m 0755 port-killer %{buildroot}%{_bindir}/kport
ln -sf kport %{buildroot}%{_bindir}/port-killer

%files
%{_bindir}/kport
%{_bindir}/port-killer

%changelog
* Fri Sep 11 2026 Shyam Charan Das <neuralshyam@gmail.com> - 0.2.0-1
- Initial public release v0.2.0
