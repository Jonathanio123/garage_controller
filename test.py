#!/usr/bin/python3

from time import sleep
import sys, os

import random
from traceback import print_tb
print("starting...")
print("running...")
try:
    while True:
        with open("./distance", 'w') as f:
            f.write(str(random.randint(0,30)))
            f.flush
            os.fsync(f.fileno())

        sleep(1)
except KeyboardInterrupt:
    print("Signal received")
    print("done")
    os.remove("./distance")
    sys.exit(0)
except:
    print("done")
    sys.exit(0)
