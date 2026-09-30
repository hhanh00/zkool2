import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import update_nix_hashes as updater


class UpdateHashesTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.flake = Path(self.directory.name) / "flake.nix"
        self.original = updater.FLAKE.read_text()
        self.flake.write_text(self.original)
        self.hashes = ["sha256-" + letter * 43 + "=" for letter in "BCD"]

    def result(self, target, fail_verification=False):
        index = [entry[2] for entry in updater.TARGETS].index(target)
        if updater.FAKE_HASH in self.flake.read_text():
            output = ("error: hash mismatch in fixed-output derivation '/nix/store/test.drv':\n"
                      f"         specified:    {updater.FAKE_HASH}\n"
                      f"            got:       {self.hashes[index]}\n")
            return subprocess.CompletedProcess([], 1, output)
        return subprocess.CompletedProcess([], int(fail_verification), "")

    def test_refreshes_and_verifies_only_dependency_outputs(self):
        with patch.object(updater, "FLAKE", self.flake), patch.object(
            updater, "build", side_effect=self.result
        ) as build:
            updater.main()
        expected = self.original
        for (marker, attribute, _), value in zip(updater.TARGETS, self.hashes):
            expected = updater.replace_hash(expected, marker, attribute, value)
        self.assertEqual(self.flake.read_text(), expected)
        self.assertEqual([call.args[0] for call in build.call_args_list],
                         [target for _, _, target in updater.TARGETS for _ in range(2)])

    def test_fetch_failure_restores_all_hashes(self):
        def fail_second(target):
            if target == "cargo-vendor":
                return subprocess.CompletedProcess([], 1, "error: network unavailable")
            return self.result(target)

        with patch.object(updater, "FLAKE", self.flake), patch.object(
            updater, "build", side_effect=fail_second
        ), self.assertRaises(RuntimeError):
            updater.main()
        self.assertEqual(self.flake.read_text(), self.original)

    def test_verification_failure_restores_original(self):
        with patch.object(updater, "FLAKE", self.flake), patch.object(
            updater, "build", side_effect=lambda target: self.result(target, True)
        ), self.assertRaises(RuntimeError):
            updater.main()
        self.assertEqual(self.flake.read_text(), self.original)


if __name__ == "__main__":
    unittest.main()
