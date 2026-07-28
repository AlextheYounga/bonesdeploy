from bonesinfra.frameworks.base import StaticFramework


class VueFramework(StaticFramework):
    static_root = "dist"


FRAMEWORK = VueFramework()
