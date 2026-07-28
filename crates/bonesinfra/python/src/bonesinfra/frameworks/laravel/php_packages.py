from pyinfra.operations import apt


def install_php(php_version):
    packages = [
        f"php{php_version}",
        f"php{php_version}-cli",
        f"php{php_version}-fpm",
        f"php{php_version}-bcmath",
        f"php{php_version}-curl",
        f"php{php_version}-gd",
        f"php{php_version}-intl",
        f"php{php_version}-mbstring",
        f"php{php_version}-mysql",
        f"php{php_version}-sqlite3",
        f"php{php_version}-xml",
        f"php{php_version}-zip",
        "composer",
    ]

    apt.packages(
        name="Install Laravel PHP packages",
        packages=packages,
        present=True,
        update=True,
        _sudo=True,
    )
