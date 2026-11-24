import pathlib
import re
import sys
import tomllib

root = pathlib.Path(__file__).resolve().parents[1]
expected_hashes = {"manifest.toml": "d6f9dc8d29a9948ce2a23e0b69ed9248bba5dfd0246f0df995a6523335b923e6", "api-contract.toml": "17e047692990c5b37e01681ae6d0ae9825af11ec02d1d44cd951b2e3bc2675f8"}
manifest = tomllib.loads((root / "architecture/manifest.toml").read_text())
allowed = {entry["path"] for entry in manifest["files"]}
ignored = {"build", ".gradle", "target", ".idea", ".git", "node_modules", "__pycache__"}
actual = {
    path.relative_to(root).as_posix()
    for path in root.rglob("*")
    if path.is_file()
    and not ignored.intersection(path.relative_to(root).parts)
    and path.name not in {"local.properties"}
}
errors = [f"CONTRACT CHANGED {name}" for name, digest in expected_hashes.items() if __import__("hashlib").sha256((root / "architecture" / name).read_bytes()).hexdigest() != digest]
errors += [f"MISSING {path}" for path in sorted(allowed - actual)]
errors += [f"UNEXPECTED {path}" for path in sorted(actual - allowed)]
for entry in manifest["files"]:
    path = root / entry["path"]
    if not path.is_file():
        continue
    declarations = entry.get("declarations", [])
    if declarations:
        source = path.read_text()
        for name in declarations:
            if not re.search(r"\b(?:class|interface|object|fun|struct|enum|trait|type)\s+" + re.escape(name) + r"\b", source):
                errors.append(f"DECLARATION {entry['path']}: {name}")
for error in errors:
    print(error)
sys.exit(bool(errors))
