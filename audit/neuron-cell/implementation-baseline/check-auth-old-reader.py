"""Local compatibility rehearsal. Requires retained pre-auth and current binaries."""
import json
import pathlib
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent
OLD = HERE / "legacy-unsigned-soft3-binary"
NEW = HERE.parents[2] / "crate/target/debug/soft3"


def port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def start(binary, home, number):
    return subprocess.Popen(
        [str(binary), "node", "--home", str(home), "--bind", f"127.0.0.1:{number}"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )


def ready(process, number):
    for _ in range(100):
        if process.poll() is not None:
            raise AssertionError(process.communicate()[1].decode())
        try:
            with urllib.request.urlopen(f"http://127.0.0.1:{number}/health", timeout=0.2) as response:
                assert response.status == 200
                return
        except OSError:
            time.sleep(0.05)
    raise AssertionError("local node readiness timeout")


def stop(process):
    if process.poll() is None:
        process.terminate()
    process.communicate(timeout=5)


with tempfile.TemporaryDirectory(prefix="soft3-auth-upgrade-") as directory:
    home = pathlib.Path(directory)
    number = port()
    old = start(OLD, home, number)
    try:
        ready(old, number)
        request = urllib.request.Request(
            f"http://127.0.0.1:{number}/v1/link",
            data=json.dumps({"neuron":"01", "from":"a", "to":"b", "amount":1}).encode(),
            headers={"Content-Type":"application/json"},
        )
        with urllib.request.urlopen(request, timeout=2) as response:
            assert json.load(response)["height"] == 1
    finally:
        stop(old)
    original = (home / "genesis.json").read_bytes()
    legacy = (home / "log").read_bytes()
    print("PASS: original executable opens and writes its pre-BBG flat log")
    result = subprocess.run([str(NEW), "auth", "enable", "--home", str(home), "--import-legacy"], capture_output=True, timeout=10)
    assert result.returncode == 0, result.stderr.decode()
    assert (home / "genesis.json/source").read_bytes() == original
    assert (home / "log").read_bytes() == legacy
    print("PASS: explicit activation preserves original genesis bytes")
    old = start(OLD, home, number)
    try:
        _, error = old.communicate(timeout=10)
        assert old.returncode != 0
        assert b"directory" in error.lower(), error.decode()
        print("PASS: original flat-log executable rejects the retired genesis reader path")
    finally:
        stop(old)
    current = start(NEW, home, number)
    try:
        ready(current, number)
        with urllib.request.urlopen(f"http://127.0.0.1:{number}/capabilities", timeout=2) as response:
            caps = json.load(response)
        assert caps["authenticated"] and caps["profile"] == "neuron/signed-native/1"
        with urllib.request.urlopen(f"http://127.0.0.1:{number}/status", timeout=2) as response:
            assert "height: 1\n" in response.read().decode()
        for route in ("/v1/link", "/v1/pay", "/v1/frame", "/v2/frame"):
            request = urllib.request.Request(f"http://127.0.0.1:{number}{route}", data=b"{}", method="POST")
            try:
                urllib.request.urlopen(request, timeout=2)
            except urllib.error.HTTPError as error:
                assert error.code == 401
            else:
                raise AssertionError("unsigned mutation succeeded")
        print("PASS: current executable reopens and all unsigned mutation routes reject")
    finally:
        stop(current)
