#!/usr/bin/env python3

from PIL import Image
import numpy
import json
import sys

image = Image.open(sys.argv[1])
colours = image.getcolors(image.size[0] * image.size[1])
colours = numpy.flip(numpy.sort(colours, axis=0), axis=0)

print(json.dumps(colours.tolist()))
