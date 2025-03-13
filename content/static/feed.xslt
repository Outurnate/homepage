<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:atom="http://www.w3.org/2005/Atom" exclude-result-prefixes="atom">
  <xsl:output method="html" version="1.0" encoding="UTF-8" indent="yes"/>
  <xsl:template match="/">
    <html xmlns="http://www.w3.org/1999/xhtml">
      <head>
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8"/>
        <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1"/>
        <title>Web Feed - <xsl:value-of select="atom:feed/atom:title"/><xsl:value-of select="rss/channel/title"/></title>
        <style type="text/css">
          body {
            line-height: 1.5;
            font-family: Sans-serif;
            font-size: 1em;
            width: 80%;
            min-width: 200px;
            max-width: 720px;
            margin: 0 auto;
            padding-top: 40px;
            padding-bottom: 40px;

            @media screen and (max-width: 480px) {
              width: 90%;
            }
          }
          div.alert {
            background-color: #feff9c;
            padding: 10px;
          }
        </style>
      </head>
      <body>
        <div class="alert">
          <p><strong>This is a web feed</strong>, also known as an RSS feed. <strong>Subscribe</strong> by copying the URL from the address bar into your newsreader app.</p>
        </div>
        <h1>Web Feed Preview</h1>
        <p>Visit <a href="https://aboutfeeds.com/">About Feeds</a> to get started with newsreaders and subscribing. It's free.</p>
        <h2>Recent Items</h2>
        <xsl:apply-templates select="atom:feed/atom:entry" />
        <xsl:apply-templates select="rss/channel/item" />
      </body>
    </html>
  </xsl:template>

  <xsl:template match="atom:entry | item">
    <div class="entry">
      <h3>
        <a target="_blank">
          <xsl:attribute name="href">
            <xsl:value-of select="atom:link/@href"/>
            <xsl:value-of select="link"/>
          </xsl:attribute>
          <xsl:value-of select="atom:title"/>
          <xsl:value-of select="title"/>
        </a>
      </h3>
      <p>
        <xsl:value-of select="atom:summary" />
        <xsl:value-of select="description" />
      </p>
      <small>
        Published: <xsl:value-of select="atom:published" /><xsl:value-of select="pubDate" />
      </small>
    </div>
  </xsl:template>

</xsl:stylesheet>