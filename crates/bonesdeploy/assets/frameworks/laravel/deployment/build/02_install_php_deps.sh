#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

require_file() {
	local file="$1"
	local message="$2"

	[ -f "$file" ] || die "$message"
}

skip_unless_laravel_project() {
	if [ ! -f artisan ]; then
		log "artisan not found; skipping Laravel PHP dependency install."
		exit 0
	fi
}

require_environment() {
	require_file artisan "artisan not found"

	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

_add_sury_repo() {
	local codename="bookworm"
	local keyring="/usr/share/keyrings/deb.sury.org-php.gpg"

	if [ -f "$keyring" ]; then
		apt-get update
		return
	fi

	apt-get update
	apt-get install -y --no-install-recommends apt-transport-https ca-certificates curl

	local deb
	deb="$(mktemp /tmp/debsuryorg-keyring.XXXXXX.deb)"

	curl -fsSL -o "$deb" https://packages.sury.org/debsuryorg-archive-keyring.deb
	dpkg -i "$deb"
	rm -f "$deb"

	# ponytail: codename tied to BUILD_IMAGE (buildpack-deps:bookworm).
	# If the build container moves to a different Debian release, update this.
	echo "deb [signed-by=${keyring}] https://packages.sury.org/php ${codename} main" \
		>/etc/apt/sources.list.d/php.list
	apt-get update
}

install_system_packages() {
	log "Installing PHP and Composer build packages..."
	export DEBIAN_FRONTEND=noninteractive
	# Prevent dpkg triggers from trying to start/restart daemons. policy-rc.d
	# blocks the action; the systemctl stub prevents deb-systemd-invoke from
	# hanging while waiting for a dbus socket that doesn't exist in the container.
	printf '#!/bin/sh\nexit 101\n' >/usr/sbin/policy-rc.d
	chmod +x /usr/sbin/policy-rc.d
	printf '#!/bin/sh\nexit 0\n' >/usr/bin/systemctl
	chmod +x /usr/bin/systemctl

	if [ -n "${PHP_VERSION:-}" ]; then
		_add_sury_repo
		apt-get install -y --no-install-recommends \
			"php${PHP_VERSION}-bcmath" \
			"php${PHP_VERSION}-cli" \
			"php${PHP_VERSION}-curl" \
			"php${PHP_VERSION}-gd" \
			"php${PHP_VERSION}-intl" \
			"php${PHP_VERSION}-mbstring" \
			"php${PHP_VERSION}-mysql" \
			"php${PHP_VERSION}-sqlite3" \
			"php${PHP_VERSION}-xml" \
			"php${PHP_VERSION}-zip" \
			curl \
			ca-certificates \
			git \
			unzip
	else
		apt-get update
		apt-get install -y --no-install-recommends \
			git \
			curl \
			ca-certificates \
			php-bcmath \
			php-cli \
			php-curl \
			php-gd \
			php-intl \
			php-mbstring \
			php-mysql \
			php-sqlite3 \
			php-xml \
			php-zip \
			unzip
	fi
	log "System packages installed."
}

php_command() {
	if [ -n "${PHP_VERSION:-}" ]; then
		echo "php${PHP_VERSION}"
	else
		echo php
	fi
}

composer_version() {
	local version="${COMPOSER_VERSION:-2.8.12}"

	[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "COMPOSER_VERSION must be a stable semantic version, got: $version"
	echo "$version"
}

install_composer() {
	local composer="/usr/local/bin/composer.phar"
	local temporary_composer="${composer}.tmp"
	local expected_checksum
	local actual_checksum
	local version

	version="$(composer_version)"
	log "Downloading Composer ${version}..."
	curl --fail --show-error --silent --location \
		--connect-timeout 15 --max-time 120 \
		--retry 3 --retry-delay 2 --retry-connrefused \
		-o "$temporary_composer" \
		"https://getcomposer.org/download/${version}/composer.phar"

	read -r expected_checksum _ < <(curl --fail --show-error --silent --location \
		--connect-timeout 15 --max-time 30 \
		"https://getcomposer.org/download/${version}/composer.phar.sha256sum")
	actual_checksum="$(sha256sum "$temporary_composer")"
	actual_checksum="${actual_checksum%% *}"

	if [ "$expected_checksum" != "$actual_checksum" ]; then
		rm -f "$temporary_composer"
		die "Composer checksum mismatch."
	fi

	chmod 0755 "$temporary_composer"
	mv -f "$temporary_composer" "$composer"
}

configure_environment() {
	export COMPOSER_ALLOW_SUPERUSER="${COMPOSER_ALLOW_SUPERUSER:-1}"
	export CI=1
	export COREPACK_ENABLE_DOWNLOAD_PROMPT=0
}

install_composer_dependencies() {
	log "Installing Composer dependencies..."

	"$(php_command)" /usr/local/bin/composer.phar install \
		--no-dev \
		--prefer-dist \
		--no-interaction \
		--optimize-autoloader
}

main() {
	skip_unless_laravel_project
	require_environment
	install_system_packages
	configure_environment
	install_composer

	install_composer_dependencies

	trap - ERR

	log "Successfully installed php dependencies."
}

main "$@"
