#!/usr/bin/env bash
#
# Build a Debian package (.deb) for Rayap Musik.
#
# Usage: packaging/build-deb.sh
# Output: dist/rayap-musik_<version>_<arch>.deb
#
set -euo pipefail

PKG="rayap-musik"
ARCH="$(dpkg --print-architecture)"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(grep -m1 '^version' "$ROOT/Cargo.toml" | sed -E 's/.*"(.*)".*/\1/')"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

echo "Building ${PKG} ${VERSION} (${ARCH})..."

cargo build --release --locked --manifest-path "$ROOT/Cargo.toml"

install -Dm755 "$ROOT/target/release/$PKG" "$STAGE/usr/bin/$PKG"
install -Dm644 "$ROOT/README.md" "$STAGE/usr/share/doc/$PKG/README.md"
install -Dm644 "$ROOT/LICENSE" "$STAGE/usr/share/doc/$PKG/LICENSE"

cat > "$STAGE/usr/share/doc/$PKG/copyright" <<'EOF'
Rayap Musik
Copyright (C) 2026 Rayap Musik maintainers

Licensed under the MIT License. See the accompanying LICENSE file.
EOF

mkdir -p "$STAGE/DEBIAN"
cat > "$STAGE/DEBIAN/control" <<EOF
Package: $PKG
Version: $VERSION
Section: sound
Priority: optional
Architecture: $ARCH
Depends: libc6 (>= 2.31), libasound2t64 | libasound2
Maintainer: Rayap Musik maintainers <noreply@example.com>
Description: local-first terminal music library manager and player
 Rayap Musik is a keyboard-driven terminal (TUI) music library manager and
 player for Linux. It indexes a local music directory into an embedded SQLite
 database, groups the collection into Songs / Albums / Artists, offers global
 search and persistent playlists (with M3U import/export), and plays audio.
 No server or network access is required.
EOF

mkdir -p "$ROOT/dist"
OUT="$ROOT/dist/${PKG}_${VERSION}_${ARCH}.deb"
dpkg-deb --build --root-owner-group "$STAGE" "$OUT"
echo "Built $OUT"
