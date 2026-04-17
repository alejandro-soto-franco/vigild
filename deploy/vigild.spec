Name:           vigild
Version:        0.1.0
Release:        1%{?dist}
Summary:        Multi-host systemd health daemon written in Rust

License:        MIT OR Apache-2.0
URL:            https://github.com/alejandrosotofranco/vigild
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  systemd-devel
BuildRequires:  dbus-devel
BuildRequires:  openssl-devel
BuildRequires:  systemd-rpm-macros

Requires:       systemd

%description
vigild is a multi-host Linux service health daemon that integrates with
systemd via D-Bus, tails journal logs per unit, gossips health state
across hosts over TCP, and exposes aggregated status on a Unix socket
as newline-delimited JSON.

%prep
%autosetup

%build
/usr/bin/cargo build --release -p vigild

%install
install -Dpm 0755 target/release/vigild %{buildroot}%{_bindir}/vigild
install -d %{buildroot}%{_rundir}/vigild
install -Dpm 0644 systemd/vigild.service \
    %{buildroot}%{_unitdir}/vigild.service
install -Dpm 0644 config/vigild.toml.example \
    %{buildroot}%{_sysconfdir}/vigild/config.toml.example

%post
%systemd_post vigild.service

%preun
%systemd_preun vigild.service

%postun
%systemd_postun_with_restart vigild.service

%files
%license LICENSE-MIT LICENSE-APACHE
%{_bindir}/vigild
%{_unitdir}/vigild.service
%config(noreplace) %{_sysconfdir}/vigild/config.toml.example

%changelog
* Fri Apr 17 2026 Alejandro Soto Franco <sotofranco.eng@gmail.com> - 0.1.0-1
- Initial release
