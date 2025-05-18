# Blacklist builtins
del __builtins__.__dict__['open']

import importlib


def secure_importer_factory(importlib):
    def secure_importer(name, globals=None, locals=None, fromlist=(), level=0):
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
        if name.split('.')[0] in whitelist:
            return importlib.__import__(name, globals, locals, fromlist, level)
        else:
            raise ImportError(f"module `{name}` is not whitelist")

    return secure_importer

__builtins__.__dict__['__import__'] = secure_importer_factory(importlib)
importlib = None  # prevent using importlib in python code
del secure_importer_factory

# Default import
try:
    import numpy
except ImportError:
    pass
try:
    import scipy
except ImportError:
    pass
