class Framework:
    def deploy(self, ctx):
        raise NotImplementedError


class StaticFramework(Framework):
    pass


class ServerFramework(Framework):
    pass


class PHPFramework(Framework):
    pass
