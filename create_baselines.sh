#!/usr/bin/env zsh

ansiGreen='\033[0;32m'
ansiLightGrey='\033[1;37m'
ansiNoColour='\033[0m'

# muted pushd
pushd() {
  command pushd "$@" >/dev/null
}

# muted popd
popd() {
  command pushd "$@" >/dev/null
}

set -o errexit
set -o nounset
set -o pipefail
if [[ "${TRACE-0}" == "1" ]]; then
    set -o xtrace
fi

cd "$(dirname "$0")"

main() {
	echo -e "$ansiGreen"
	echo 'Creating baseline benchmarking for trunk (lexer)'
	echo -e "$ansiNoColour"
	cargo bench --bench lexing -- --save-baseline trunk --verbose

	echo -e "$ansiGreen"
	echo 'Creating baseline benchmarking for trunk (parser)'
	echo -e "$ansiNoColour"
	cargo bench --bench parsing -- --save-baseline trunk --verbose
}

main "$@"
