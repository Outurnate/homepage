Title: One Click RCE By Design
Date: 2025-02-09
Category: Cybersecurity

This is an old story about a defunct custom integration between two software platforms.  To my knowledge, my employer at the time was (thankfully) the only site this integration was deployed at.  I'll be using pseudonyms in this article, as names aren't necessary.  The integration (henceforth `INTEGRATION.EXE`) was between an emergency department triage and patient flow system (henceforth `EDSOFT.EXE`) and an electronic medical record system (`EMR.EXE`).  Each of these executables came from a different vendor.  The goal of the integration was called a "contextual launch".  A user viewing a patient in `EDSOFT.EXE` should be able to launch a new session in `EMR.EXE` and view the same patient.  This was because the patient's medical record was spread across both software platforms, and getting a complete record meant switching between both products[^1].

`INTEGRATION.EXE` took command line arguments containing a one-time auth token alongside the medical record number for the current patient.  It would then, in turn, launch `EMR.EXE`, login, and open the correct patient.  This was...weak, but not the subject of the article.  The issue was, `EDSOFT.EXE` could only add http(s) links to the patient's chart.  There was no facility for launching an arbitrary executable.  Faced with this issue, did `EDSOFT.EXE`'s vendor decide to add in support for arbitrary executables?  No, they did something much worse.  Enter `launch.asp`.  I sadly don't have the original source, however, I have reproduced a simile below.

```html
<html>
  <head>
    <title>Close this window</title>
  </head>
  <body>
    <h1>Close this window</h1>
    <script type="text/javascript">
      var objShell = new ActiveXObject("WScript.shell")
      objShell.run("<%= Request.QueryString('cmdline') %>")
    </script>
  </body>
</html>
```

If you read that and thought it was a web shell, read it again.  It's a client side web shell.  `EDSOFT.EXE` would craft a link like `http://server/launch.asp?cmdline=INTEGRATION.EXE%20...`, which would launch an Internet Explorer[^2] window, which would launch `INTEGRATION.EXE`, which would launch `EMR.EXE`.  Of course, it goes without saying, one could craft a link to run any executable, and it would work.  Nobody cared about that at the time, as was the trend with that era of cybersecurity in healthcare.  Thankfully, a few years later, `EDSOFT.EXE` was dumped, and along with it, this abomination.

I only found this because I was asked to make it faster.

[^1]: this was exactly as inconvenient as you'd think
[^2]: because only IE was silly enough to execute this JS - provided the site was on the trusted sites list (it was) and "unsafe ActiveX" was enabled for the zone.  This also allowed any site on the trusted site list the same permission, since the setting could only be applied to the entire zone.
