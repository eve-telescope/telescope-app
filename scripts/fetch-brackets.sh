#!/usr/bin/env bash
# Downloads the overview bracket icons for player ship classes from the
# current Tranquility client build into crates/telescope-app/assets/brackets.
set -euo pipefail

out="$(dirname "$0")/../crates/telescope-app/assets/brackets"
mkdir -p "$out"

hulls=(
  battlecruiser battleship capsule carrier cruiser destroyer dreadnought
  forceauxiliary freighter frigate industrial industrialcommand miningbarge
  miningdestroyer miningfrigate rookie shuttle supercarrier titan
)

build=$(curl -fsS https://binaries.eveonline.com/eveclient_TQ.json |
  sed -E 's/.*"build" *: *"?([0-9]+)"?.*/\1/')
index_file=$(curl -fsS "https://binaries.eveonline.com/eveonline_${build}.txt" |
  grep '^app:/resfileindex.txt,' | cut -d, -f2)
index=$(curl -fsS "https://binaries.eveonline.com/${index_file}" | gunzip)

# Non-hull brackets, stored under their own name.
extras=(cynosuralfield)

for hull in "${hulls[@]}" "${extras[@]}"; do
  path="res:/ui/texture/shared/brackets/${hull}_32.png"
  if [[ " ${extras[*]} " == *" ${hull} "* ]]; then
    path="res:/ui/texture/shared/brackets/${hull}.png"
  fi
  file=$(grep -F "${path}," <<<"$index" | head -1 | cut -d, -f2)
  if [[ -z "$file" ]]; then
    echo "missing ${path}" >&2
    continue
  fi
  curl -fsS "https://resources.eveonline.com/${file}" | gunzip >"${out}/${hull}.png"
done

echo "Fetched brackets from client build ${build}"
