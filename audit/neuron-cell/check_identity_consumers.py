#!/usr/bin/env python3
"""Compile isolated identity consumers and audit only normal/build dependency edges.

No workspace feature unification can accidentally make a tiny profile look heavy
or hide a heavy dependency. Output is evidence, never an automatic snapshot bless.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess

PROFILES = {
    "id": ("neuron-id", "neuron/id", [], "pub fn id() -> subject::NeuronId { [0;32] }"),
    "model": ("neuron-model", "neuron/model", [], "pub fn id() -> subject::identity::SubjectRef { subject::identity::SubjectRef::Native([0;32]) }"),
    "model-serde": ("neuron-model", "neuron/model", ["serde"], "pub fn id() -> subject::identity::SubjectRef { subject::identity::SubjectRef::Native([0;32]) }"),
    "mudra": ("cyber-mudra", "mudra", [], "pub fn id() -> subject::NeuronId { [0;32] }"),
    "lytics-event": ("lytics-event", "lytics/rs/event", [], "pub use subject::*;"),
    "inf-value": ("inf-value", "inf/rs/value", [], "pub fn id() -> subject::Value { subject::Value::neuron([0;32]) }"),
    "inf-source": ("inf-source", "inf/rs/source", [], "pub fn source() -> subject::LocalSource { subject::LocalSource::new() }"),
}
FORBIDDEN = {"bbg", "cybergraph", "neuron-engine", "neuron-node", "neuron-rune", "cyber-nox", "zheng", "soma-agent", "kernel", "glia", "cyb-core", "foculus"}

def run(command, cwd, env, output):
    with output.open("w") as log:
        subprocess.run(command, cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--root",type=Path,default=Path(__file__).resolve().parents[3])
    parser.add_argument("--out",type=Path,required=True)
    args=parser.parse_args();root=args.root.resolve();out=args.out.resolve();out.mkdir(parents=True,exist_ok=True)
    nu=shutil.which("nu");assert nu,"Nushell is required"
    env=os.environ.copy()
    # Cargo and rustc must belong to the same installed WASM toolchain, including
    # on hosts where Homebrew cargo precedes rustup in PATH.
    sysroot=Path(subprocess.check_output(["rustup","run","stable","rustc","--print","sysroot"],text=True).strip())
    env["RUSTC"]=str(sysroot/"bin/rustc");env["RUSTDOC"]=str(sysroot/"bin/rustdoc")
    env["CARGO_TARGET_DIR"]=str(out/"target")
    results=[]
    for name,(package,path,features,code) in PROFILES.items():
        directory=out/name;(directory/"src").mkdir(parents=True,exist_ok=True)
        (directory/"Cargo.toml").write_text(f'''[package]
name = "identity-check-{name}"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies.subject]
package = {json.dumps(package)}
path = {json.dumps(str(root/path))}
default-features = false
features = {json.dumps(features)}
''')
        (directory/"src/lib.rs").write_text(code+"\n")
        metadata=subprocess.check_output([nu,"-c","rustup run stable cargo metadata --offline --format-version 1 --filter-platform wasm32-unknown-unknown"],cwd=directory,env=env,text=True)
        data=json.loads(metadata);nodes={n["id"]:n for n in data["resolve"]["nodes"]};packages={p["id"]:p for p in data["packages"]}
        active=set();done=set()
        def visit(key):
            assert key not in active,"dependency cycle"
            if key in done:return
            active.add(key)
            for dependency in nodes[key]["deps"]:
                if any(kind["kind"] in (None,"build") for kind in dependency["dep_kinds"]):visit(dependency["pkg"])
            active.remove(key);done.add(key)
        visit(data["resolve"]["root"])
        names={packages[key]["name"] for key in done}
        heavy={n for n in names if n in FORBIDDEN or n.startswith(("rune-","bevy","glia-"))}
        assert not heavy,(name,sorted(heavy))
        run([nu,"-c","rustup run stable cargo check --offline --target wasm32-unknown-unknown"],directory,env,out/f"{name}.log")
        results.append({"profile":name,"package":package,"features":features,"target":"wasm32-unknown-unknown","dependencies":sorted(names),"result":"passed"})
        print(f"PASS {name}: {len(names)-1} normal/build dependencies; WASM compiled",flush=True)
    (out/"results.json").write_text(json.dumps({"schema":"neuron/identity-consumer-audit/1","rustc":subprocess.check_output([env["RUSTC"],"--version"],text=True).strip(),"profiles":results},indent=2)+"\n")

if __name__=="__main__":main()
