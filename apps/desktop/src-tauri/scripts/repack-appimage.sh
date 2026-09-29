#!/usr/bin/env bash
# linuxdeploy leaves AppRun.wrapped as 770. Firejail (AppImageHub) mounts as
# another user, so exec fails with "Permission denied". Make owner-executable
# files world-executable and readable, then repack.
set -euo pipefail

umask 022

bundle_dirs=()
if [ -n "${1:-}" ]; then
  bundle_dirs+=("$1")
else
  bundle_dirs+=(
    target/release/bundle/appimage
    apps/desktop/src-tauri/target/release/bundle/appimage
  )
fi

shopt -s nullglob
appimages=()
for dir in "${bundle_dirs[@]}"; do
  [ -d "$dir" ] || continue
  for f in "$dir"/*.AppImage; do
    appimages+=("$f")
  done
done

if [ "${#appimages[@]}" -eq 0 ]; then
  echo "No AppImage found in: ${bundle_dirs[*]}"
  exit 1
fi

workdir=$(mktemp -d)
trap 'rm -rf "$workdir"' EXIT
tool="$workdir/appimagetool.AppImage"
curl -fsSL -o "$tool" \
  https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod a+x "$tool"

for image in "${appimages[@]}"; do
  image=$(readlink -f "$image")
  echo "Fixing permissions in $image"
  chmod a+x "$image"
  extract="$workdir/squashfs-root"
  rm -rf "$extract"
  (
    cd "$workdir"
    APPIMAGE_EXTRACT_AND_RUN=1 "$image" --appimage-extract >/dev/null
  )
  # a+rX / 755: dirs and owner-exec files become world-readable/executable.
  # linuxdeploy left AppRun.wrapped as 770, which Firejail cannot exec.
  find "$extract" -type d -exec chmod 755 {} +
  find "$extract" -type f -perm -u+x -exec chmod 755 {} +
  find "$extract" -type f ! -perm -u+x -exec chmod 644 {} +
  out="$image.fixed"
  ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$tool" "$extract" "$out"
  chmod a+x "$out"
  mv -f "$out" "$image"
  echo "Repacked $image"
done
