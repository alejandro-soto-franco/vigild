# vigild

[![COPR build status](https://copr.fedorainfracloud.org/coprs/alejandro-soto-franco/vigild/package/vigild/status_image/last_build.png)](https://copr.fedorainfracloud.org/coprs/alejandro-soto-franco/vigild/package/vigild/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Multi-host Linux service health daemon written in Rust.

vigild integrates with systemd via D-Bus, tails journal logs per unit, gossips health state across hosts over TCP, and exposes aggregated status on a Unix socket as newline-delimited JSON.

## Install

### Fedora (COPR)

```bash
sudo dnf copr enable alejandro-soto-franco/vigild
sudo dnf install vigild
sudo systemctl enable --now vigild
```

Verify:

```bash
systemctl status vigild
sudo -u vigild cat /run/vigild/status.sock | head -1   # if readable
```

Example config is at `/etc/vigild/config.toml.example`; copy to `/etc/vigild/config.toml` and edit.

### Build from source

Requires Rust 1.75+, `systemd-devel`, `dbus-devel`, `openssl-devel`.

```bash
git clone https://github.com/alejandro-soto-franco/vigild
cd vigild
cargo build --release -p vigild
sudo install -Dpm0755 target/release/vigild /usr/local/bin/vigild
sudo install -Dpm0644 systemd/vigild.service /etc/systemd/system/vigild.service
sudo install -Dpm0644 config/vigild.toml.example /etc/vigild/config.toml.example
sudo systemctl daemon-reload
sudo systemctl enable --now vigild
```

### Build an RPM locally

```bash
./deploy/mksrpm.sh
rpmbuild --rebuild ~/rpmbuild/SRPMS/vigild-*-1.fc*.src.rpm
sudo dnf install ~/rpmbuild/RPMS/x86_64/vigild-*-1.fc*.x86_64.rpm
```

## Architecture

- `vigild-core/`: types, gossip protocol, journal reader, D-Bus client, aggregator
- `vigild/`: daemon binary wiring core tasks together with graceful SIGINT shutdown
- `systemd/vigild.service`: unit file installed to `/usr/lib/systemd/system/`
- `deploy/vigild.spec`: RPM specfile (offline-vendored Rust build)

## Packaging

The RPM is built with `--offline --frozen` against a `cargo vendor` tree. `deploy/mksrpm.sh` regenerates `vendor/` and `.cargo/config.toml` before tarring, so neither needs to be in git. The SRPM is the unit of distribution to COPR.

```bash
./deploy/mksrpm.sh
copr-cli build alejandro-soto-franco/vigild ~/rpmbuild/SRPMS/vigild-*.src.rpm
```

Enabled COPR chroots: `fedora-43-x86_64`, `fedora-44-x86_64`, `fedora-rawhide-x86_64`. Fedora branching is followed, so each new release inherits the rawhide chroot when it branches.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
