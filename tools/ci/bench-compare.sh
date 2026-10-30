#!/usr/bin/env bash
# Times two revisions on one machine with the same bench workloads (plan §7, M10's gate: the bench
# stays within 10 % of M9's figures). Each revision is built in its own worktree and target, then
# every workload runs on both, interleaved, pinned to the same CPUs. Timings from different
# machines or CPU allocations are not comparable; only the ratio printed here is.
#
#   tools/ci/bench-compare.sh <base rev> <new rev> <out dir>
#
# BENCH_CPUS (taskset list, default 0-3, or every CPU under four), BENCH_ROUNDS (default 3), BENCH_JOBS (cargo -j, default
# 2) and BENCH_GATE=1 (also M9's gate workload once a revision: 10 × 10,000 people, about an hour
# each on four cores and 2.5 GB). Both revisions must be in the local clone (fetch-depth 0 in CI).
set -euo pipefail
if (( $# != 3 )); then
  echo 'usage: tools/ci/bench-compare.sh <base rev> <new rev> <out dir>' >&2
  exit 2
fi
repo=$(cd "$(dirname "$0")/../.." && pwd)
out=$(mkdir -p "$3" && cd "$3" && pwd)
cpus=${BENCH_CPUS:-0-$(( $(nproc) < 4 ? $(nproc) - 1 : 3 ))}
rounds=${BENCH_ROUNDS:-3}
jobs=${BENCH_JOBS:-2}
mkdir -p "$out/bin" "$out/logs"

# The workloads: DO's three bench worlds (M10a slice DO) and DN's (M9c), by bench_village flags.
declare -A work=(
  [village500]='--people 500 --days 90'
  [three400]='--people 400 --groups 3 --size 1024 --known --max --household 2 --days 60'
  [ten60]='--seed 1 --people 60 --groups 10 --size 2048 --known --max --household 10 --days 40'
)
names=(village500 three400 ten60)
gate='--people 10000 --groups 10 --size 2048 --known --max --household 10 --days 5'

{
  echo "date: $(date -u +%FT%TZ)"
  echo "host: $(hostname); $(nproc) CPUs online; pinned to $cpus"
  lscpu | grep -E '^Model name' || true
  free -h | sed -n 2p
  echo "rounds: $rounds; gate workload: ${BENCH_GATE:-0}"
} > "$out/environment.txt"

for side in base new; do
  rev=$([[ $side == base ]] && echo "$1" || echo "$2")
  sha=$(git -C "$repo" rev-parse --verify "$rev^{commit}")
  tree="$out/trees/$side"
  if [[ ! -d $tree ]]; then
    git -C "$repo" worktree add --detach "$tree" "$sha" >/dev/null
  fi
  (cd "$tree/kernel" && CARGO_TARGET_DIR="$out/target-$side" \
    cargo build --locked --release -j "$jobs" -p civ-sim --example bench_village)
  cp "$out/target-$side/release/examples/bench_village" "$out/bin/$side"
  echo "$side: $sha $(git -C "$repo" log -1 --format=%s "$sha")" >> "$out/environment.txt"
done

run() { # workload, side, round, flags...
  local name=$1 side=$2 round=$3
  shift 3
  echo "$name $side round $round"
  # shellcheck disable=SC2068
  taskset -c "$cpus" "$out/bin/$side" $@ > "$out/logs/$name-$side-$round.txt" 2>&1
}
for ((r = 1; r <= rounds; r++)); do
  for name in "${names[@]}"; do
    # Alternate which side goes first, so drift in the machine falls on both.
    if (( r % 2 )); then order=(base new); else order=(new base); fi
    for side in "${order[@]}"; do
      run "$name" "$side" "$r" ${work[$name]}
    done
  done
done
if [[ ${BENCH_GATE:-0} == 1 ]]; then
  for side in base new; do
    run gate100k "$side" 1 $gate
  done
  names+=(gate100k)
fi

# The summary: each workload's median ms a day on both sides, and new over base.
{
  printf 'workload\tbase ms/day\tnew ms/day\tnew/base\tbase people\tnew people\n'
  for name in "${names[@]}"; do
    line="$name"
    for side in base new; do
      med=$(grep -h ' ms a day on average' "$out/logs/$name-$side-"*.txt \
        | sed -E 's/.*: ([0-9.]+) ms a day.*/\1/' | sort -n \
        | awk '{a[NR] = $1} END {if (NR) print (NR % 2 ? a[(NR + 1) / 2] : (a[NR / 2] + a[NR / 2 + 1]) / 2)}')
      line+=$'\t'"${med:-missing}"
      eval "med_$side=\${med:-0}"
    done
    ratio=$(awk -v b="$med_base" -v n="$med_new" 'BEGIN {if (b > 0) printf "%.3f", n / b; else print "-"}')
    line+=$'\t'"$ratio"
    for side in base new; do
      people=$(grep -h '^scale: ' "$out/logs/$name-$side-"*.txt | sed -E 's/^scale: ([0-9]+) people.*/\1/' \
        | paste -sd/ -)
      line+=$'\t'"${people:-missing}"
    done
    echo "$line"
  done
} | tee "$out/summary.tsv"
