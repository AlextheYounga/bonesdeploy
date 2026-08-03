def load(ctx):
    template = ctx.runtime.data.get("template")
    if not template:
        return

    from bonesinfra.frameworks import get_framework

    runtime = get_framework(template)
    runtime.deploy(ctx)
