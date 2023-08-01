import os
import sys
sys.path.append(os.curdir)
from pelicanconf import *

SITEURL = 'https://outurnate.com'
RELATIVE_URLS = False

CATEGORY_FEED_ATOM = 'feeds/{slug}.atom.xml'

DELETE_OUTPUT_DIRECTORY = True