#!/usr/bin/env python

import argparse
import os.path
import tomlkit

parser = argparse.ArgumentParser(
        description='Replace the mmtk-core dependency of a given VM binding',
        )

parser.add_argument('toml_path', help='Path to Cargo.toml')
parser.add_argument('mmtk_core_path', help='Path to the mmtk_core repo')

args = parser.parse_args()

print("Reading TOML from '{}'".format(args.toml_path))
with open(args.toml_path, "rt") as f:
    toml_data = tomlkit.load(f)

mmtk_node = toml_data["dependencies"]["mmtk"]

print("Deleting dependencies.mmtk.git")
if "git" in mmtk_node:
    del mmtk_node["git"]

mmtk_repo_path = os.path.realpath(args.mmtk_core_path)
print("Setting dependencies.mmtk.path to {}".format(mmtk_repo_path))
mmtk_node["path"] = mmtk_repo_path

print("Writing TOML to '{}'".format(args.toml_path))
with open(args.toml_path, "wt") as f:
    tomlkit.dump(toml_data, f)

print("Done.")