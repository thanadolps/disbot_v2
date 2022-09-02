import importlib

# Default import
import numpy
import scipy

whitelist = {
    "numpy",
    "scipy",
    "math",
    "string",
    "re",
    "struct",
    "datetime",
    "collections",
    "enum",
    "fractions",
    "itertools",
    "functools",
    "random",
    "glob",
    "hashlib",
    "time",
    "queue",
}

def secure_importer(name, globals=None, locals=None, fromlist=(), level=0):
    if name.split('.')[0] in whitelist:
        return importlib.__import__(name, globals, locals, fromlist, level)
    else:
        raise ImportError(f"module `{name}` is not whitelist")


__builtins__.__dict__['__import__'] = secure_importer

