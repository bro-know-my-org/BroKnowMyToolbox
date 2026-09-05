#!/usr/bin/env bash
# CI-only: format newly created regular images, never an existing disk/device.
set -euo pipefail

[[ "$(uname -s)" == Linux ]]
[[ "${GITHUB_ACTIONS:-}" == true ]]
[[ "${RUNNER_TEMP:-}" == /* && -d "$RUNNER_TEMP" ]]
scratch=$(mktemp -d "$RUNNER_TEMP/bkmt-filesystems.XXXXXX")
mounted=""
cleanup() {
  result=$?
  if [[ -n "$mounted" ]]; then
    sudo umount -- "$mounted" || result=1
  fi
  exit "$result"
}
trap cleanup EXIT

# Compile outside the tested filesystem; the subsequent test runs reuse this build.
cargo test --locked -p bkmt-file-generator --test concurrent_creation --no-run

for filesystem in vfat exfat; do
  disk_image="$scratch/$filesystem.img"
  mount_root="$scratch/$filesystem"
  mkdir "$mount_root"
  truncate -s 128M "$disk_image"
  [[ -f "$disk_image" && ! -L "$disk_image" ]]
  if [[ "$filesystem" == vfat ]]; then
    mkfs.fat -F 32 "$disk_image"
  else
    mkfs.exfat "$disk_image"
  fi
  sudo mount -t "$filesystem" -o "loop,uid=$(id -u),gid=$(id -g)" "$disk_image" "$mount_root"
  mounted="$mount_root"
  [[ "$(findmnt --noheadings --output FSTYPE --target "$mount_root")" == "$filesystem" ]]

  # Prove this is writable but lacks hard links, not a fallback directory on ext4.
  touch "$mount_root/link-source"
  if ln "$mount_root/link-source" "$mount_root/link-target"; then
    echo "Unexpected hard-link support on $filesystem" >&2
    exit 1
  fi
  echo "Testing no-replace creation on mounted $filesystem"
  TMPDIR="$mount_root" cargo test --locked -p bkmt-file-generator --test concurrent_creation
  sudo umount -- "$mount_root"
  mounted=""
done
