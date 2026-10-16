"""Recompute retained negative pixel evidence, including the A/A controls."""
import hashlib
import json
from pathlib import Path
from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


provenance = read("archive-provenance.json")
assert provenance["productionSource"] == "823a8a0615153068594daef530c2e80f1d95fc29"
assert provenance["productionWorktreeClean"] and not provenance["hostProductStarted"]
assert not provenance["formalMemoryMeasurementRepeated"] and not provenance["releaseAccepted"]
for item in provenance["originalCopies"]:
    assert hashlib.sha256((ROOT / item["path"]).read_bytes()).hexdigest() == item["sha256"]
images = {}
for item in provenance["images"]:
    data = (ROOT / item["blob"]).read_bytes()
    assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
    images[item["run"], item["name"]] = ROOT / item["blob"]


def diff(run, name, role):
    left = Image.open(images[run, f"{name}-reference.png"]).convert("RGBA")
    right = Image.open(images[run, f"{name}-{role}.png"]).convert("RGBA")
    assert left.size == right.size
    delta = ImageChops.difference(left, right)
    return (sum(1 for pixel in delta.getdata() if any(pixel)),
            max(high for low, high in delta.getextrema()))


counts = {}
for run in ("crop-height", "control", "crop-transform", "premultiplied"):
    raw = read(f"{run}/pixels.original.json")
    assert raw["baseline"] == provenance["productionSource"]
    assert not raw["hostProductStarted"] and not raw["nativePerformanceGatePassed"]
    assert len(raw["results"]) == 4
    cases = [case for result in raw["results"] for case in result["cases"]]
    assert len(cases) == 24
    if run != "premultiplied":
        assert all(c["differentBytes"] == c["maxDifference"] == 0
                   and c["rgbaSha256"][0] == c["rgbaSha256"][1] for c in cases)
    else:
        assert all(c["differentBytes"] > 0 and 0 < c["maxDifference"] <= 2 for c in cases)
    if run == "crop-height":
        report = read(f"{run}/compositor-comparison.json")["comparisons"]
        for row in report:
            observed = diff(run, row["name"], "candidate")
            assert observed == (row["differentPixels"], row["maxChannelDifference"])
        assert max(r["maxChannelDifference"] for r in report) == 5
    elif run in ("control", "crop-transform"):
        report = read(f"{run}/" + ("compositor-control.json" if run == "control"
                                  else "compositor-comparison.json"))
        for row in report:
            for role in ("candidate", "reference-repeat"):
                observed = diff(run, row["name"], role)
                assert observed == (row[role]["differentPixels"], row[role]["maxChannelDifference"])
            assert row["reference-repeat"]["differentPixels"] == 0
        if run == "control":
            assert all(row["candidate"]["differentPixels"] == 0 for row in report)
        else:
            assert any(row["candidate"]["differentPixels"] > 0 for row in report)
    else:
        report = read(f"{run}/compositor-comparison.json")
        for row in report:
            assert diff(run, row["name"], "candidate") == (
                row["compositedDifferentPixels"], row["maxCompositedChannelDifference"])
        assert all(row["compositedDifferentPixels"] > 0 for row in report)
    counts[run] = len(cases)
if (ROOT / "manifest.json").exists():
    for item in read("manifest.json")["files"]:
        data = (ROOT / item["path"]).read_bytes()
        assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
print(json.dumps({
    "status": "verified-rejected-render-experiments", "cases": counts,
    "imageReferences": len(images), "uniquePngFiles": len(set(images.values())),
    "allAaControlsIdentical": True, "productionChangesAdopted": False,
    "nativeMemoryGatePassed": False, "historicalFailureCleared": False,
    "releaseAccepted": False,
}, indent=2))
