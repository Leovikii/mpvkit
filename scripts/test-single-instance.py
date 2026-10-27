"""Linux GUI regression on a private session bus and fresh application roots."""
import concurrent.futures
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

if len(sys.argv) == 1:
    sys.exit(subprocess.call(["dbus-run-session", "--", sys.executable, __file__, "--private-session"]))
assert sys.argv[1:] == ["--private-session"]

app = Path("src-tauri/target/debug/mpvkit").resolve()
root = Path(tempfile.mkdtemp(prefix="mpvkit-single-instance-"))
env = dict(os.environ, GDK_BACKEND="wayland", WAYLAND_DEBUG="client",
           MPVKIT_TEST_MODE="1", MPVKIT_TEST_ROOT=str(root / "state"),
           XDG_DATA_HOME=str(root / "data"), XDG_CONFIG_HOME=str(root / "config"),
           XDG_CACHE_HOME=str(root / "cache"))
name = "io.github.mpvkit.SingleInstance"


def owner_pid():
    reply = subprocess.run(["gdbus", "call", "--session", "--dest", "org.freedesktop.DBus",
                            "--object-path", "/org/freedesktop/DBus", "--method",
                            "org.freedesktop.DBus.GetConnectionUnixProcessID", name],
                           capture_output=True, text=True, timeout=3)
    return reply.stdout.strip() if reply.returncode == 0 else None


def stop(child):
    child.terminate()
    try:
        child.wait(timeout=5)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait(timeout=5)


rounds = []
for attempt in range(2):
    logfile = root / f"primary-{attempt}.log"
    with logfile.open("w") as log:
        primary = subprocess.Popen([str(app)], env=env, stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 20
            while time.monotonic() < deadline:
                assert primary.poll() is None, logfile.read_text()[-2000:]
                if owner_pid() == f"(uint32 {primary.pid},)" and '.set_app_id("io.github.mpvkit")' in logfile.read_text():
                    break
                time.sleep(.1)
            else:
                raise AssertionError("Primary GUI did not acquire its bus name and Wayland window")

            def duplicate(index):
                result = subprocess.run([str(app)], env=env, capture_output=True, text=True, timeout=15)
                (root / f"duplicate-{attempt}-{index}.log").write_text(result.stdout + result.stderr)
                assert result.returncode == 0, result.stderr[-2000:]
                assert '.set_app_id(' not in result.stderr, "Duplicate created a Wayland window"
                return result.returncode

            with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
                codes = list(pool.map(duplicate, range(6)))
            assert primary.poll() is None
            assert owner_pid() == f"(uint32 {primary.pid},)"
            helper = subprocess.run([str(app), "--save-playback"], env=env, capture_output=True, text=True, timeout=5)
            assert helper.returncode == 1 and "helper_arguments" in helper.stdout + helper.stderr
            rounds.append({"primaryPid": primary.pid, "duplicateExitCodes": codes, "helperNotIntercepted": True})
        finally:
            stop(primary)

broken_env = dict(env, DBUS_SESSION_BUS_ADDRESS="unix:path=/nonexistent-mpvkit-test-bus")
broken = subprocess.run([str(app)], env=broken_env, capture_output=True, text=True, timeout=5)
assert broken.returncode != 0 and "session_bus_unavailable" in broken.stderr
assert '.set_app_id(' not in broken.stderr
helper = subprocess.run([str(app), "--save-playback"], env=broken_env, capture_output=True, text=True, timeout=5)
assert helper.returncode == 1 and "helper_arguments" in helper.stdout + helper.stderr
result = {"root": str(root), "rounds": rounds, "missingBusRejected": True, "helperWorksWithoutBus": True}
(root / "result.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
