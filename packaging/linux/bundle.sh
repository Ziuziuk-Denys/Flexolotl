#!/bin/bash
set -euo pipefail

version=$1 program=$2 out=$(realpath -m "$3")
here=$(dirname "$(realpath "$0")")
id=quest.nuvo.Flexolotl
machine=$(uname -m)
case $machine in
  x86_64) image=x86_64 debian=amd64 red_hat=x86_64 ;;
  aarch64) image=arm64 debian=arm64 red_hat=aarch64 ;;
esac
mkdir -p "$out"

cargo deb --no-build --no-strip -p flexolotl -o "$out/Flexolotl-$version-linux-$debian.deb"
cargo generate-rpm -p crates/app -o "$out/Flexolotl-$version-linux-$red_hat.rpm"

root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT
install -Dm755 "$program" "$root/usr/bin/flexolotl"
install -Dm644 "$here/$id.desktop" "$root/usr/share/applications/$id.desktop"
install -Dm644 "$here/$id.metainfo.xml" "$root/usr/share/metainfo/$id.appdata.xml"
install -Dm644 "$here/../icon.png" "$root/usr/share/icons/hicolor/512x512/apps/$id.png"
cp "$here/$id.desktop" "$here/../icon.png" "$root/"
mv "$root/icon.png" "$root/$id.png"
ln -s "$id.png" "$root/.DirIcon"
ln -s usr/bin/flexolotl "$root/AppRun"
tar -C "$root" -czf "$out/Flexolotl-$version-linux-$image.tar.gz" usr
ARCH=$machine appimagetool --appimage-extract-and-run "$root" "$out/Flexolotl-$version-linux-$image.AppImage"
