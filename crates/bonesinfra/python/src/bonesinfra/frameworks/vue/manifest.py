def artifacts(ctx):
    paths = ctx.paths
    return [
        ("static placeholder web root", f"{paths.placeholder_release}/dist", "directory", "framework"),
        ("static placeholder index", f"{paths.placeholder_release}/dist/index.html", "file", "framework"),
        ("current static web root", f"{paths.current}/dist", "directory", "framework"),
    ]


def services(_ctx):
    return []


def mode(_ctx):
    return "static"
