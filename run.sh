#!/bin/bash
set -euo pipefail

cd /home/hana/work/hana/blog-testbed/program
cargo build-bpf
solana program deploy target/deploy/bpf_program_template.so
cd /home/hana/work/hana/blog-testbed
npm run main
