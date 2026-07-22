#!/usr/bin/env python3
"""Verifica post-pubblicazione di una release firmata di Prompt à Porter.

Uso: python3 verifica.py vX.Y.Z [--repo-root /path/al/repo]

Controlla (vedi SKILL.md per il razionale):
  1. la release è pubblicata (non draft) ed è la Latest del repo;
  2. latest.json: version corretta, ogni campo `signature` identico al file
     `.sig` pubblicato, tutte le URL puntano al tag, notes riferite al tag;
  3. firma Ed25519 (minisign) del setup.exe ri-firmato, valida contro la
     pubkey updater di tauri.conf.json (payload + global signature);
  4. presenza della firma Authenticode embedded nel PE (security directory).

Richiede: gh autenticato, python3 con il pacchetto `cryptography`.
Exit code 0 = tutto ok, 1 = almeno un controllo fallito.
"""

import argparse
import base64
import hashlib
import json
import struct
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.parse import unquote

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

MINISIGN_PREHASH_ALG = b"ED"  # BLAKE2b-512 del file, poi Ed25519
PE_HEADER_PTR_OFFSET = 0x3C
PE32PLUS_MAGIC = 0x20B
# Offset della security directory (indice 4) dall'inizio dell'optional header.
SECURITY_DIR_OFFSET = {True: 144, False: 128}  # True = PE32+


def sh(*args: str) -> str:
    return subprocess.run(args, check=True, capture_output=True, text=True).stdout


class Verifica:
    def __init__(self) -> None:
        self.tutto_ok = True

    def check(self, label: str, cond: bool, extra: str = "") -> None:
        print(f"{'OK ' if cond else 'FAIL'} {label}" + (f" — {extra}" if extra else ""))
        self.tutto_ok = self.tutto_ok and cond


def leggi_pubkey_updater(repo_root: Path) -> tuple[bytes, bytes]:
    conf = json.loads((repo_root / "apps/client/src-tauri/tauri.conf.json").read_text())
    righe = base64.b64decode(conf["plugins"]["updater"]["pubkey"]).decode().splitlines()
    raw = base64.b64decode(righe[1])
    return raw[2:10], raw[10:42]  # key id, chiave Ed25519


def verifica_minisign(v: Verifica, exe: Path, sig_file: Path, repo_root: Path) -> bytes:
    pk_keyid, pk_key = leggi_pubkey_updater(repo_root)
    righe = base64.b64decode(sig_file.read_text().strip()).decode().splitlines()
    raw = base64.b64decode(righe[1])
    alg, keyid, firma = raw[:2], raw[2:10], raw[10:74]
    trusted_comment = righe[2].replace("trusted comment: ", "", 1)
    global_sig = base64.b64decode(righe[3])

    v.check("key id della firma == pubkey updater", keyid == pk_keyid)
    dati = exe.read_bytes()
    msg = (
        hashlib.blake2b(dati, digest_size=64).digest()
        if alg == MINISIGN_PREHASH_ALG
        else dati
    )
    pub = Ed25519PublicKey.from_public_bytes(pk_key)
    try:
        pub.verify(firma, msg)
        v.check("firma Ed25519 updater sul setup.exe", True)
    except Exception as e:  # InvalidSignature e affini
        v.check("firma Ed25519 updater sul setup.exe", False, repr(e))
    try:
        pub.verify(global_sig, firma + trusted_comment.encode())
        v.check("global signature (trusted comment)", True)
    except Exception as e:
        v.check("global signature (trusted comment)", False, repr(e))
    return dati


def verifica_authenticode_presente(v: Verifica, dati: bytes) -> None:
    pe_off = struct.unpack_from("<I", dati, PE_HEADER_PTR_OFFSET)[0]
    magic = struct.unpack_from("<H", dati, pe_off + 24)[0]
    dir_off = pe_off + 24 + SECURITY_DIR_OFFSET[magic == PE32PLUS_MAGIC]
    _, cert_size = struct.unpack_from("<II", dati, dir_off)
    v.check("Authenticode embedded nel PE", cert_size > 0, f"{cert_size} byte")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("tag", help="tag della release, es. v0.8.42")
    ap.add_argument("--repo-root", default=".", type=Path)
    args = ap.parse_args()
    tag, versione = args.tag, args.tag.lstrip("v")
    v = Verifica()

    info = json.loads(sh("gh", "release", "view", tag, "--json", "isDraft,publishedAt"))
    v.check("release pubblicata (non draft)", not info["isDraft"], info.get("publishedAt") or "")
    latest = json.loads(sh("gh", "api", "repos/{owner}/{repo}/releases/latest"))["tag_name"]
    v.check(f"Latest del repo == {tag}", latest == tag, f"latest={latest}")

    with tempfile.TemporaryDirectory() as td_str:
        td = Path(td_str)
        sh(
            "gh", "release", "download", tag, "--dir", str(td), "--clobber",
            "--pattern", "latest.json", "--pattern", "*.sig", "--pattern", "*setup.exe",
        )
        lj = json.loads((td / "latest.json").read_text())
        v.check(f"latest.json version == {versione}", lj["version"] == versione)

        url_sbagliate = []
        for piattaforma, entry in lj["platforms"].items():
            if tag not in entry["url"]:
                url_sbagliate.append(piattaforma)
            nome_asset = unquote(entry["url"].rsplit("/", 1)[-1])
            sig_path = td / (nome_asset + ".sig")
            if sig_path.exists():
                v.check(
                    f"signature {piattaforma} == {nome_asset}.sig",
                    entry["signature"].strip() == sig_path.read_text().strip(),
                )
        v.check("tutte le URL puntano al tag", not url_sbagliate, ", ".join(url_sbagliate))

        note = lj.get("notes", "")
        v.check("notes presenti e riferite al tag", bool(note) and tag in note)

        exe = next(td.glob("*setup.exe"), None)
        v.check("setup.exe scaricato", exe is not None)
        if exe is not None:
            dati = verifica_minisign(v, exe, td / (exe.name + ".sig"), args.repo_root)
            verifica_authenticode_presente(v, dati)

    print("\nESITO:", "TUTTO OK" if v.tutto_ok else "PROBLEMI RILEVATI")
    sys.exit(0 if v.tutto_ok else 1)


if __name__ == "__main__":
    main()
