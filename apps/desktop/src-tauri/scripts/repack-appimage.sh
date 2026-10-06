#!/usr/bin/env bash
# linuxdeploy leaves AppRun.wrapped as 770. Firejail (AppImageHub) mounts as
# another user, so exec fails with "Permission denied". Make owner-executable
# files world-executable and readable, then repack.
#
# Also embed GitHub Releases zsync update information (appimagetool -u) so
# AppImageUpdate works. AppImageHub still reports "not self-contained / glibc
# 2.35": AppImages use the host C library on purpose, and Tauri v2 cannot
# target older than Ubuntu 22.04 (WebKitGTK 4.1).
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

if ! command -v zsyncmake >/dev/null 2>&1; then
  echo "zsyncmake is required to publish AppImageUpdate metadata (apt install zsync)"
  exit 1
fi

repo="${GITHUB_REPOSITORY:-antonpiat/taurvia}"
owner="${repo%%/*}"
name="${repo#*/}"

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

  publish_name=$(basename "$image")
  pattern="$publish_name"
  if [[ "$pattern" =~ ([0-9]+\.[0-9]+\.[0-9]+) ]]; then
    pattern="${pattern/${BASH_REMATCH[1]}/*}"
  fi
  upd_info="gh-releases-zsync|${owner}|${name}|latest|${pattern}.zsync"
  echo "Update information: $upd_info"

  # Output must use the published filename so the .zsync Filename: field matches
  # the GitHub Release asset. Run from $workdir: zsyncmake writes basename.zsync
  # in CWD, not next to an absolute destination path.
  out="$workdir/$publish_name"
  rm -f "$out" "$out.zsync" "./${publish_name}.zsync"
  (
    cd "$workdir"
    ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$tool" -u "$upd_info" "$extract" "$publish_name"
    chmod a+x "$publish_name"
    # appimagetool continuous often logs "generating zsync file" then writes
    # nothing (https://github.com/AppImage/appimagetool/issues/84). Always
    # build the sibling with the system zsyncmake we already require.
    rm -f "${publish_name}.zsync"
    zsyncmake -u "$publish_name" "$publish_name"
  )
  chmod a+x "$out"
  if [ ! -f "$out.zsync" ]; then
    echo "zsyncmake did not write $out.zsync"
    exit 1
  fi
  mv -f "$out" "$image"
  mv -f "$out.zsync" "$(dirname "$image")/${publish_name}.zsync"
  echo "Repacked $image"
  echo "Wrote $(dirname "$image")/${publish_name}.zsync"
done
