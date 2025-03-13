from feedgenerator import get_tag_uri
from markupsafe import Markup
from pelican import signals
from pelican.utils import set_date_tzinfo
from pelican.plugins.pelican_feed_stylesheet import StyledFeedWriter


class StyledFeedWriterModified(StyledFeedWriter):
    def _create_new_feed(self, feed_type, feed_title, context):
        return super()._create_new_feed(feed_type, feed_title, context)

    def _add_item_to_the_feed(self, feed, item):
        title = Markup(item.title).striptags()
        link = self.urljoiner(self.site_url, item.url)

        description = Markup(item.summary).striptags()

        categories = list()
        if hasattr(item, "category"):
            categories.append(item.category)
        if hasattr(item, "tags"):
            categories.extend(item.tags)

        feed.add_item(
            title=title,
            link=link,
            unique_id=get_tag_uri(link, item.date),
            description=description,
            categories=categories if categories else None,
            author_name=getattr(item, "author", ""),
            pubdate=set_date_tzinfo(item.date, self.settings.get("TIMEZONE", None)),
            updateddate=set_date_tzinfo(
                item.modified, self.settings.get("TIMEZONE", None)
            )
            if hasattr(item, "modified")
            else None,
        )


def add_writer(pelican_object):
    return StyledFeedWriterModified


def register():
    signals.get_writer.connect(add_writer)
