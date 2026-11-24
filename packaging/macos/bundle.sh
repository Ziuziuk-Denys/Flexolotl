#!/bin/bash
set -euo pipefail

version=$1 program=$2 label=$3 out=$4
here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
app=$work/image/Flexolotl.app
mkdir -p "$out" "$app/Contents/MacOS" "$app/Contents/Resources" "$work/Flexolotl.iconset"

for side in 16 32 128 256 512; do
  double=$((side * 2))
  sips -z $side $side "$here/icon.png" --out "$work/Flexolotl.iconset/icon_${side}x$side.png" >/dev/null
  sips -z $double $double "$here/icon.png" --out "$work/Flexolotl.iconset/icon_${side}x$side@2x.png" >/dev/null
done
iconutil -c icns "$work/Flexolotl.iconset" -o "$app/Contents/Resources/Flexolotl.icns"
sed "s/VERSION/$version/" "$here/Info.plist" >"$app/Contents/Info.plist"
cp "$program" "$app/Contents/MacOS/flexolotl"
codesign --force --deep --sign - "$app"

ditto -c -k --keepParent "$app" "$out/Flexolotl-$version-mac-$label.zip"
ln -s /Applications "$work/image/Applications"
hdiutil create -volname Flexolotl -srcfolder "$work/image" -format UDZO -ov "$out/Flexolotl-$version-mac-$label.dmg"
