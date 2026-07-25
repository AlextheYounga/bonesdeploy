def load(ctx):
    template = ctx.runtime.data.get("template")
    if not template:
        return

    from bonesinfra.runtimes import get_runtime

    runtime = get_runtime(template)
    runtime.deploy(ctx)
