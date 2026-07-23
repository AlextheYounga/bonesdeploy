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

	if [ -n "${PHP_VERSION:-}" ]; then
		_add_sury_repo
		apt-get install -y --no-install-recommends \
			"php${PHP_VERSION}-cli" \
			"php${PHP_VERSION}-curl" \
			"php${PHP_VERSION}-mbstring" \
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
			php-cli \
			php-curl \
			php-mbstring \
			php-sqlite3 \
			php-xml \
			php-zip \
			unzip
	fi
}

php_command() {
	if [ -n "${PHP_VERSION:-}" ]; then
		echo "php${PHP_VERSION}"
	else
		echo php
	fi
}

install_composer() {
	local php_bin
	local installer="/tmp/composer-setup.php"
	local expected_checksum
	local actual_checksum

	php_bin="$(php_command)"
	curl -fsSL https://getcomposer.org/installer -o "$installer"
	expected_checksum="$(curl -fsSL https://composer.github.io/installer.sig)"
	actual_checksum="$("$php_bin" -r "echo hash_file('sha384', '$installer');")"

	if [ "$expected_checksum" != "$actual_checksum" ]; then
		rm -f "$installer"
		die "Composer installer checksum mismatch."
	fi

	"$php_bin" "$installer" --quiet --install-dir=/usr/local/bin --filename=composer.phar
	rm -f "$installer"
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
