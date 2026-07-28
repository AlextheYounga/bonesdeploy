class RuntimeService:
    def provision(self, ctx):
        raise NotImplementedError

    @staticmethod
    def _db_identifier(project_name):
        name = project_name.replace("-", "_")
        if not name or len(name) > 48 or not name.replace("_", "").isalnum():
            raise ValueError("project_name cannot be used as a database identifier")
        return name
