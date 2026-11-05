#!/usr/bin/env bash
# build-jar.sh — build RandPassGenerator/PassGenerator.jar without hard
# dependencies on Apache Ant. Tries, in order:
#   1. ant jar                (upstream's official build)
#   2. javac + jar            (replicates build.xml by hand)
#   3. build_noant.py         (javac + python3 zipfile; or ECJ_JAR=... for javac-less boxes)
#
# Result: build/jar/PassGenerator.jar (classes + bundled wordlist.txt)
set -e
cd "$(dirname "$0")"

if command -v ant >/dev/null 2>&1; then
    echo "[randpass] building with ant"
    ant jar
elif command -v javac >/dev/null 2>&1 && command -v jar >/dev/null 2>&1; then
    echo "[randpass] building with javac + jar"
    mkdir -p build/classes build/jar
    find src -name '*.java' > build/sources.txt
    javac -d build/classes @build/sources.txt
    printf 'Main-Class: gov.nsa.ia.pass.RandPassGenerator\n' > build/manifest.txt
    jar cfm build/jar/PassGenerator.jar build/manifest.txt -C build/classes gov -C . data
    rm -f build/sources.txt build/manifest.txt
elif command -v python3 >/dev/null 2>&1; then
    echo "[randpass] building with build_noant.py (no ant/jar tool found)"
    python3 build_noant.py
else
    echo "[randpass] FAIL: need ant, or a JDK (javac+jar), or python3" >&2
    echo "           hint: Debian/Ubuntu: apt install default-jdk" >&2
    exit 1
fi

test -f build/jar/PassGenerator.jar
echo "[randpass] OK: build/jar/PassGenerator.jar"
