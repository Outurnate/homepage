import sys
sys.path.append(".")

import d2extension
import asciimathextension
import pelican_feed_stylesheet2

AUTHOR = "Joe Dillon"
SITENAME = "Joe Dillon"
SITESUBTITLE = "Personal projects, research, and other things I find worth sharing"
SITEURL = "https://outurnate.com"
RELATIVE_URLS = True

PLUGINS = ["minify", "webassets", "sitemap", "seo", "pelican_feed_stylesheet2"]

PATH = "content"
PAGE_PATHS = ["pages"]
STATIC_PATHS = ["pages", "static"]
ARTICLE_PATHS = ["articles"]
IGNORE_FILES = [".#*", "*.scss"]
EXTRA_PATH_METADATA = {
    "static/favicon.ico": {"path": "favicon.ico"},
    "static/favicon.svg": {"path": "favicon.svg"},
    "static/feed.xslt": {"path": "feeds/feed.xslt"}
}

TIMEZONE = "America/Toronto"
LOCALE = "en_CA"
DEFAULT_LANG = "en"

USE_FOLDER_AS_CATEGORY = True
DELETE_OUTPUT_DIRECTORY = True

FEED_ALL_ATOM = "feeds/feed.atom.xml"
FEED_ALL_RSS = "feeds/feed.xml"
CATEGORY_FEED_ATOM = None
TRANSLATION_FEED_ATOM = None
AUTHOR_FEED_ATOM = None
AUTHOR_FEED_RSS = None
FEED_STYLESHEET = "feed.xslt"

DIRECT_TEMPLATES = ["index", "archives"]
CATEGORY_SAVE_AS = ""
AUTHOR_SAVE_AS = ""

DEFAULT_PAGINATION = False

THEME = "theme"

MARKDOWN = {
  "extension_configs": {
    "d2extension": {},
    "markdown_include.include": {
      "base_path": "content"
    },
    "asciimathextension": {},
    "markdown.extensions.codehilite": {
      "css_class": "highlight"
    },
    "markdown.extensions.extra": {},
    "markdown.extensions.admonition": {},
    "markdown.extensions.meta": {},
  },
  "output_format": "html5",
}

SEO_REPORT = True
SEO_ENHANCER = True
SEO_ENHANCER_OPEN_GRAPH = True
SEO_ENHANCER_TWITTER_CARDS = False
