+++
title = "Simple markov generator"
date = 2020-04-12
+++

<script src="generator.js"></script>
<script src="markov.js"></script>
<link rel="stylesheet" type="text/css" href="markov.css" />

A while back, as a first foray into Rust and WASM, I wrote a simple
markov word generator.  I had trouble with getting Rust to compile into
WASM, so I wrote that bit in C++.  So, behold, C++ in the browser, using
pre-generated wordlists from a Rust program

<h1>Your word is:</h1>
<div>
  <span id="word"></span>
  <img id="loader" src="loading.gif" />
</div>
<p>Get a <a id="regen" href="#">new word</a></p>
<label for="jsonfile">Or choose a different wordlist:</label>
<select id="jsonfile">
  <option value="google-10000-english-no-swears.json">Google word list</option>
  <option value="arineng-last-names.json">Last names</option>
  <option value="dominictarr-first-names.json">First names</option>
</select>

<a href="https://github.com/Outurnate/markov-word-generator">Have a look at the code, if you really want to</a>