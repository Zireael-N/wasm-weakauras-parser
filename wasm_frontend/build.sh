#!/usr/bin/env bash

# -e - exit on error
# -u - treat unset variables as an error
# -x - print each command before executing it
# -o pipefail - sets the exit code of a pipeline to that of the rightmost command to exit with a non-zero status,
#   or to zero if all commands of the pipeline exit successfully.
set -euxo pipefail

LIB_PATH="${PWD}/opt"
export GIT_AUTHOR_DATE="$(git log -n1 --format='%aI')"
export GIT_COMMITTER_DATE="$(git log -n1 --format='%cI')"

set +x
if [[ "${SSH_KEY:+1}" ]]; then
	if [[ -z "${GIT_REMOTE:+1}" ]]; then
		GIT_REMOTE="$(git remote get-url origin | sed -E 's|https?://([^/]+)/(.*)|git@\1:\2|g')"
	fi
	mkdir ~/.ssh
	echo -e "Host *\n\tStrictHostKeyChecking no\n" > ~/.ssh/config
	eval $(ssh-agent -s)
	echo "${SSH_KEY}" | ssh-add -
fi
set -x

wasm-pack build --release --target web --out-dir dist

# patch the .js-file
sed -i -e '/^export { initSync }$/d' -e '/^export default __wbg_init;$/d' -e "/^[[:space:]]*if (typeof input === 'undefined')/,+2 d" -e 's/^export //g' dist/wasm_weakauras_parser.js
cat ./src/worker.js >> ./dist/wasm_weakauras_parser.js
cp ./src/index.html ./dist/index.html

# deploy to gh-pages
if [[ "${GIT_REMOTE:+1}" ]]; then
	pushd dist
	rm .gitignore
	git init
	git checkout --orphan gh-pages
	git add .
	git config user.name "${GIT_NAME:-GitHub Actions}"
	git config user.email "${GIT_EMAIL:-githubci@localhost}"
	git commit -m "gh-pages"
	git remote add origin "${GIT_REMOTE}"
	git push -f -u origin gh-pages
	popd # dist
fi
