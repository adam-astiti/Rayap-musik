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
install -Dm644 "$ROOT/docs/PRD.md" "$STAGE/usr/share/doc/$PKG/PRD.md"
install -Dm644 "$ROOT/docs/QA.md" "$STAGE/usr/share/doc/$PKG/QA.md"

cat > "$STAGE/usr/share/doc/$PKG/copyright" <<'EOF'
Rayap Musik
Copyright (C) 2026 Rayap Musik maintainers

Licensed under the MIT License.
EOF

mkdir -p "$STAGE/DEBIAN"
cat > "$STAGE/DEBIAN/control" <<EOF
Package: $PKG
Version: $VERSION
Section: sound
Priority: optional
Architecture: $ARCH
Depends: libc6 (>= 2.34), libasound2t64 | libasound2
Maintainer: Rayap Musik maintainers <noreply@example.com>
Description: local-first terminal music player
 Rayap Musik is a keyboard-driven terminal (TUI) music player for Linux.
 It browses a local music directory, reads audio metadata, plays supported
 audio files (including M4A/AAC), and manages an in-memory playback queue.
 No server or network access is required.
EOF

mkdir -p "$ROOT/dist"
OUT="$ROOT/dist/${PKG}_${VERSION}_${ARCH}.deb"
dpkg-deb --build --root-owner-group "$STAGE" "$OUT"
echo "Built $OUT"
