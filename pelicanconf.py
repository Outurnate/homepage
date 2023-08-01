AUTHOR = 'Joe Dillon'
SITENAME = 'Joe Dillon'
SITEURL = ''

PLUGINS = ['minify', 'webassets']

PATH = 'content'
PAGE_PATHS = ['pages']
STATIC_PATHS = ['pages']
ARTICLE_PATHS = ['articles']
IGNORE_FILES = ['.#*', '*.scss']

TIMEZONE = 'America/Toronto'

DEFAULT_LANG = 'en'

USE_FOLDER_AS_CATEGORY = True
DELETE_OUTPUT_DIRECTORY = True

FEED_ALL_ATOM = 'feeds/all.atom.xml'
FEED_ALL_RSS = 'feeds/all.xml'
CATEGORY_FEED_ATOM = None
TRANSLATION_FEED_ATOM = None
AUTHOR_FEED_ATOM = None
AUTHOR_FEED_RSS = None

DIRECT_TEMPLATES = ['index', 'archives']
CATEGORY_SAVE_AS = ''
AUTHOR_SAVE_AS = ''

DEFAULT_PAGINATION = False

THEME = 'outurnate'