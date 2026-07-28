Name:           vigild
Version:        0.2.0
Release:        2%{?dist}
Summary:        Multi-host systemd health daemon written in Rust

License:        MIT OR Apache-2.0
URL:            https://github.com/alejandro-soto-franco/vigild
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
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
/usr/bin/cargo build --release --offline --frozen -p vigild

%install
install -Dpm 0755 target/release/vigild %{buildroot}%{_bindir}/vigild
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
* Tue Jul 28 2026 Alejandro Soto Franco <sotofranco.eng@gmail.com> - 0.2.0-2
- Build for fedora-rawhide
- Drop systemd-devel, dbus-devel and openssl-devel from BuildRequires: zbus is
  pure Rust and the binary links only glibc, libgcc and libm
- Drop the unpackaged /run/vigild install; the unit declares
  RuntimeDirectory=vigild

* Fri Apr 17 2026 Alejandro Soto Franco <sotofranco.eng@gmail.com> - 0.2.0-1
- Report inactive/failed units in watch list instead of silently dropping them
- Resolve unit aliases via LoadUnit (dbus.service -> dbus-broker.service)
- Report not-found for missing units rather than omitting

* Fri Apr 17 2026 Alejandro Soto Franco <sotofranco.eng@gmail.com> - 0.1.0-1
- Initial release
