#!/usr/bin/env python3
"""Build PassGenerator.jar without ant/jar tools (sandbox has a stripped JDK).

Reproduces nsacyber/RandPassGenerator build.xml: compile src -> classes,
bundle classes + wordlist.txt at jar root, manifest with Main-Class.
A .jar is just a zip; the manifest must be the first entry.
"""
import os
import subprocess
import sys
import zipfile

ROOT = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(ROOT, "src")
CLASSES = os.path.join(ROOT, "build", "classes")
JAR = os.path.join(ROOT, "build", "jar", "PassGenerator.jar")
MANIFEST = "Manifest-Version: 1.0\r\nMain-Class: gov.nsa.ia.pass.RandPassGenerator\r\n\r\n"

os.makedirs(CLASSES, exist_ok=True)
os.makedirs(os.path.dirname(JAR), exist_ok=True)

java_files = []
for dirpath, _, names in os.walk(SRC):
    java_files += [os.path.join(dirpath, n) for n in names if n.endswith(".java")]

import shutil
if shutil.which("javac"):
    subprocess.check_call(["javac", "-d", CLASSES] + java_files)
else:
    ecj = os.environ.get("ECJ_JAR")
    if not ecj:
        sys.exit("Neither javac nor ECJ_JAR (path to ecj.jar) is available")
    subprocess.check_call(["java", "-jar", ecj, "-11", "-d", CLASSES] + java_files)

with zipfile.ZipFile(JAR, "w", zipfile.ZIP_DEFLATED) as z:
    z.writestr(zipfile.ZipInfo("META-INF/MANIFEST.MF"), MANIFEST)
    z.write(os.path.join(ROOT, "data", "wordlist.txt"), "wordlist.txt")
    for dirpath, _, names in os.walk(CLASSES):
        for n in names:
            full = os.path.join(dirpath, n)
            rel = os.path.relpath(full, CLASSES)
            z.write(full, rel)

print("built", JAR, os.path.getsize(JAR), "bytes")
