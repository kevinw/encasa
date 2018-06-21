#!/usr/bin/python

import subprocess
import sys
import urllib

if __name__ == "__main__":
    prefix = "editlocal://"
    if not sys.argv[1].startswith(prefix):
        raw_input("expected url to start with editlocal://")
    else:
        arg = sys.argv[1][len(prefix):]
        arg = urllib.unquote(arg)
        subprocess.Popen(["c:\\Program Files (x86)\\Vim\\vim80\\gvim.exe", arg])
