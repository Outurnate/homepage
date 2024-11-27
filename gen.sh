#!/bin/bash
rm -rf output
pygmentize -S monokai -f html -a .highlight > outurnate/static/css/_pygments.scss
pelican
for model in content/articles/tesla-coil/models/*.glb
do
  npx screenshot-glb -f "image/webp" -i "$model" -o "${model%.glb}.webp"
done
