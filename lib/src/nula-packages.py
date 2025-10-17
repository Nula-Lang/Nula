# nula-packages.py - Expanded Dependency manager in Python
# Now uses requests to download from a hypothetical repo (e.g., GitHub).
# Install: pip install requests
# Handles install, list, to global or project lib.
# For working, assume repo like https://example.com/libs/{pkg}.nula

import os
import sys
import requests
import shutil

home_dir = os.path.expanduser("~")
nula_dir = os.path.join(home_dir, ".nula")
lib_dir = os.path.join(nula_dir, "lib")

if not os.path.exists(nula_dir):
    os.makedirs(nula_dir)
if not os.path.exists(lib_dir):
    os.makedirs(lib_dir)

def install_pkg(pkg):
    # Hypothetical repo URL
    url = f"https://hypothetical-nula-repo.com/libs/{pkg}.nula"
    try:
        response = requests.get(url)
        response.raise_for_status()
        lib_path = os.path.join(lib_dir, f"{pkg}.nula")
        with open(lib_path, "wb") as f:
            f.write(response.content)
        print(f"Installed {pkg} from {url}")

        # Copy to project if exists
        current_dir = os.getcwd()
        project_lib = os.path.join(current_dir, "lib")
        if os.path.exists(project_lib):
            shutil.copy(lib_path, project_lib)
            print(f"Copied to project lib")
    except requests.RequestException as e:
        print(f"Error installing {pkg}: {e}")

def list_pkgs():
    pkgs = os.listdir(lib_dir)
    if pkgs:
        print("Installed packages:")
        for p in pkgs:
            print(f"- {p}")
    else:
        print("No packages installed.")

if len(sys.argv) < 2:
    print("Usage: nula-packages [install <pkg> | list]")
    sys.exit(1)

command = sys.argv[1]
if command == "install":
    if len(sys.argv) < 3:
        print("Usage: nula-packages install <pkg>")
        sys.exit(1)
    install_pkg(sys.argv[2])
elif command == "list":
    list_pkgs()
else:
    print(f"Unknown command: {command}")
    sys.exit(1)
