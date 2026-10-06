#!/usr/bin/env bash
# linuxdeploy leaves AppRun.wrapped as 770. Firejail (AppImageHub) mounts as
# another user, so exec fails with "Permission denied". Make owner-executable
# files world-executable and readable, then repack.
#
# Embed GitHub Releases zsync update information with appimagetool -u.
# Do **not** trust appimagetool to write the sibling .zsync: continuous
# builds log "generating zsync file" then write nothing or Length: 0
# (https://github.com/AppImage/appimagetool/issues/84). System zsyncmake
# writes basename.zsync in its current working directory, not next to an
# absolute path. Always run it ourselves next to the published AppImage.
#
# AppImageHub still reports "not self-contained / glibc 2.35": AppImages use
# the host C library on purpose, and Tauri v2 cannot target older than
# Ubuntu 22.04 (WebKitGTK 4.1).
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

zsyncmake=$(command -v zsyncmake || true)
if [ -z "$zsyncmake" ]; then
  echo "zsyncmake is required to publish AppImageUpdate metadata (apt install zsync)"
  exit 1
fi
zsyncmake=$(readlink -f "$zsyncmake")
echo "Using zsyncmake: $zsyncmake"

repo="${GITHUB_REPOSITORY:-antonpiat/taurvia}"
owner="${repo%%/*}"
name="${repo#*/}"

workdir=$(mktemp -d)
trap 'rm -rf "$workdir"' EXIT
tool="$workdir/appimagetool.AppImage"
curl -fsSL --retry 5 --retry-delay 2 --retry-all-errors -o "$tool" \
  https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod a+x "$tool"

write_zsync() {
  local appimage_path=$1
  local publish_name dest_dir zsync_path size length
  publish_name=$(basename "$appimage_path")
  dest_dir=$(dirname "$appimage_path")
  zsync_path="$dest_dir/${publish_name}.zsync"
  size=$(stat -c%s "$appimage_path")
  rm -f "$zsync_path"
  (
    cd "$dest_dir"
    # -u is the relative URL AppImageUpdate uses to fetch the new payload.
    "$zsyncmake" -u "$publish_name" "$publish_name"
  )
  if [ ! -s "$zsync_path" ]; then
    echo "zsyncmake produced no file at $zsync_path"
    ls -la "$dest_dir" || true
    exit 1
  fi
  length=$(awk '/^Length:/{print $2; exit}' "$zsync_path" || true)
  if ! [[ "$length" =~ ^[0-9]+$ ]] || [ "$length" -ne "$size" ]; then
    echo "zsync Length '${length:-missing}' != AppImage size $size"
    head -20 "$zsync_path" || true
    exit 1
  fi
  echo "Wrote $zsync_path (Length $length)"
}

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

  # Relative output name so appimagetool's own (often broken) zsyncmake at least
  # stays inside $workdir. We delete it and rewrite with system zsyncmake.
  out="$workdir/$publish_name"
  rm -f "$out" "$out.zsync"
  (
    cd "$workdir"
    ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$tool" -u "$upd_info" "$extract" "$publish_name"
  )
  chmod a+x "$out"
  if [ ! -s "$out" ]; then
    echo "appimagetool did not write $out"
    exit 1
  fi
  rm -f "$out.zsync"
  mv -f "$out" "$image"
  write_zsync "$image"
  echo "Repacked $image"
done
