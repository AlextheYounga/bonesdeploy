#!/usr/bin/env bash

# Basic deployment concerns for Node.js applications

check_for_nvm() {
	# Check for .nvmrc and install the specified Node.js version
	if [ -f "./.nvmrc" ]; then
		export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
		if [ -s "$NVM_DIR/nvm.sh" ]; then
			source "$NVM_DIR/nvm.sh"
		elif [ -s "$HOME/.config/nvm/nvm.sh" ]; then
			source "$HOME/.config/nvm/nvm.sh"
		fi
		nvm install
	fi
}

run_node_build() {
	# Check for yarn.lock or package-lock.json and install dependencies accordingly
	if [ -f "./yarn.lock" ]; then
		npm install -g yarn
		yarn
		yarn build
	fi

	if [ -f "./package-lock.json" ]; then
		npm install
		npm run build
	fi
}

main() {
	check_for_nvm
	run_node_build
}

main