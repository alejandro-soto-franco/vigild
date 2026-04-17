#!/usr/bin/env bash
set -euo pipefail

NAME=vigild
VERSION=$(awk -F'"' '/^version/ {print $2; exit}' vigild/Cargo.toml)
TOPDIR=${HOME}/rpmbuild

mkdir -p "${TOPDIR}"/{SOURCES,SPECS,SRPMS,RPMS,BUILD,BUILDROOT}

cargo vendor --quiet > /dev/null
mkdir -p .cargo
cat > .cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

tar czf "${TOPDIR}/SOURCES/${NAME}-${VERSION}.tar.gz" \
  --transform "s,^\./,${NAME}-${VERSION}/," \
  --exclude='./target' \
  --exclude='./docs' \
  --exclude='./.git' \
  ./Cargo.toml ./Cargo.lock ./vigild ./vigild-core ./systemd ./config ./deploy \
  ./LICENSE-MIT ./LICENSE-APACHE ./.cargo ./vendor ./.gitignore ./README.md

cp deploy/${NAME}.spec "${TOPDIR}/SPECS/${NAME}.spec"
rpmbuild -bs "${TOPDIR}/SPECS/${NAME}.spec"

echo
echo "SRPM: ${TOPDIR}/SRPMS/${NAME}-${VERSION}-1.$(rpm --eval %{dist} | tr -d .).src.rpm"
