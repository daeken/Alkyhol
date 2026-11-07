#!/usr/bin/env python3
"""Extract exact pinned Sakura function bodies for bounded SER-off specialization."""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "shader/samples/sakura"


def mask_comments_and_strings(source):
    """Keep offsets/newlines while excluding braces/definitions inside comments/strings."""
    out = list(source)
    i = 0
    while i < len(source):
        if source.startswith("//", i):
            end = source.find("\n", i)
            end = len(source) if end < 0 else end
        elif source.startswith("/*", i):
            end = source.find("*/", i + 2)
            if end < 0:
                raise ValueError("unterminated block comment")
            end += 2
        elif source[i] in "\"'":
            quote = source[i]
            end = i + 1
            while end < len(source):
                if source[end] == "\\":
                    end += 2
                elif source[end] == quote:
                    end += 1
                    break
                else:
                    end += 1
            else:
                raise ValueError("unterminated string")
        else:
            i += 1
            continue
        for j in range(i, end):
            if source[j] not in "\r\n":
                out[j] = " "
        i = end
    return "".join(out)


def functions(source, name):
    """Return exact source spans, retaining shader attributes and overloads.

    This extractor is deliberately limited to the pinned sample's free-function
    definitions. It is not an HLSL parser or an arbitrary-source transformation.
    """
    masked = mask_comments_and_strings(source)
    pattern = (r'(?m)^(?:[ \t]*\[shader\("[a-z]+"\)\]\r?\n)?'
               r'[ \t]*(?:inline[ \t]+)?\w+[ \t]+(?P<name>' + re.escape(name) + r')\s*\(')
    found = []
    for match in re.finditer(pattern, source):
        if masked[match.start("name"):match.end("name")] != name:
            continue
        start = masked.find("{", match.end())
        declaration = masked.find(";", match.end())
        if start < 0 or (declaration >= 0 and declaration < start):
            raise ValueError(f"expected function definition: {name}")
        depth = 1
        end = start + 1
        while end < len(masked) and depth:
            depth += (masked[end] == "{") - (masked[end] == "}")
            end += 1
        if depth:
            raise ValueError(f"unterminated function: {name}")
        found.append(source[match.start():end])
    if not found:
        raise ValueError(f"missing pinned function: {name}")
    return found


def prepare(check=False):
    manifest = json.loads((PACKAGE / "upstream.json").read_text())
    for name, record in manifest["files"].items():
        data = (PACKAGE / "original" / name).read_bytes()
        if len(data) != record["bytes"] or hashlib.sha256(data).hexdigest() != record["sha256"]:
            raise ValueError(f"pinned Sakura source differs: {name}")
    source = (PACKAGE / "original/Raytracing.hlsl").read_bytes().decode()
    groups = {
        "helpers": ["HitWorldPosition", "HitAttribute", "GenerateCameraRay", "FresnelReflectanceSchlick"],
        "radiance": ["TraceRadianceRay"],
        "hits": ["FloorClosestHitShader", "TrunkClosestHitShader", "LeavesClosestHitShader", "BushClosestHitShader", "TCubeClosestHitShader"],
        "miss": ["MyMissShader"],
    }
    for group, names in groups.items():
        chunks = []
        for name in names:
            bodies = functions(source, name)
            if len(bodies) != (2 if name == "HitAttribute" else 1):
                raise ValueError(f"unexpected pinned overload count: {name}")
            chunks.extend(bodies)
        data = ("// Generated extraction of unchanged Microsoft function bodies; see upstream.json.\n"
                + "\n\n".join(chunks) + "\n").encode()
        path = PACKAGE / f"{group}.generated.hlsli"
        if check:
            if path.read_bytes() != data:
                raise ValueError(f"generated source differs: {path.name}")
        else:
            path.write_bytes(data)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    prepare(parser.parse_args().check)
