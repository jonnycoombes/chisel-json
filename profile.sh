#!/usr/bin/env zsh

ansiRed='\033[0;31m'
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

  echo -e "$ansiRed"
  echo '*** WARNING *** Profiling everything will take quite some time'
  echo -e "$ansiNoColour"

  profileTime=30

	echo -e "$ansiGreen"
	echo 'Running lexer profiling for ' $profileTime ' seconds per benchmark'
	echo -e "$ansiNoColour"
	cargo bench --bench lexing -- --profile-time $profileTime

	echo -e "$ansiGreen"
	echo 'Running DOM parser profiling for ' $profileTime ' seconds per benchmark'
	echo -e "$ansiNoColour"
	cargo bench --bench dom_parsing -- --profile-time $profileTime

	echo -e "$ansiGreen"
	echo 'Running SAX parser profiling for ' $profileTime ' seconds per benchmark'
	echo -e "$ansiNoColour"
	cargo bench --bench sax_parsing -- --profile-time $profileTime
}

main "$@"
