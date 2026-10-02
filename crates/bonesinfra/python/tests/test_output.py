from bonesinfra.cli import output


def test_print_done_uses_operation_for_success(monkeypatch):
    printed = []

    def capture(*args):
        printed.append(args)

    monkeypatch.setattr(output.console, "print", capture)

    output.print_done(success=True, operation="server setup")

    assert "server setup complete" in printed[1][0]


def test_print_done_uses_operation_for_failure(monkeypatch):
    printed = []

    def capture(*args):
        printed.append(args)

    monkeypatch.setattr(output.console, "print", capture)

    output.print_done(success=False, operation="site deletion")

    assert "site deletion failed" in printed[1][0]
