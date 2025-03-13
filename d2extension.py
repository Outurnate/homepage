import cssutils
import cssmin
import re
import xml.etree.ElementTree as ET
import logging
from subprocess import Popen, PIPE, DEVNULL
from markdown.blockparser import BlockParser
from markdown.blockprocessors import BlockProcessor
from markdown import Extension, Markdown

cssutils.log.setLevel(logging.CRITICAL)

d2_preamble = """
vars: {
  d2-config: {
    dark-theme-overrides: {
      N1: "#D6D6D6"
      N2: "#C2C2C2"
      N3: "#ADADAD"
      N4: "#5B5B5B"
      N5: "#474747"
      N6: "#323232"
      N7: "#1E1E1E"

      B1: "#98E585" # --diagram-color-a
      B2: "#6B946C" # --bordercolor
      B3: "#389844" # --highlight-fill
      B4: "#0C200D" # --diagram-color-b
      B5: "#0C200D" # --diagram-color-b
      B6: "#0C200D" # --diagram-color-b

      AA4: "#389844" # --highlight-fill
      AA5: "#389844" # --highlight-fill

      AB4: "#6B946C" # --bordercolor
      AB5: "#389844" # --highlight-fill
    }
    theme-overrides: {
      N1: "#0F0F25"
      N2: "#6C6C6C"
      N3: "#999999"
      N4: "#D2D2D2"
      N5: "#E1E1E1"
      N6: "#F1F1F1"
      N7: "#FFFFFF"

      B1: "#0C200D" # --diagram-color-a
      B2: "#6B946C" # --bordercolor
      B3: "#389844" # --highlight-fill
      B4: "#98E585" # --diagram-color-b
      B5: "#98E585" # --diagram-color-b
      B6: "#98E585" # --diagram-color-b

      AA4: "#389844" # --highlight-fill
      AA5: "#389844" # --highlight-fill

      AB4: "#6B946C" # --bordercolor
      AB5: "#389844" # --highlight-fill
    }
  }
}
style: {
  fill: transparent
}
"""


class D2BlockProcessor(BlockProcessor):
    RE_FENCE_START = r'^ *!{3,}d2 *\n'
    RE_FENCE_END = r'\n *!{3,}\s*$'

    fonts = {
        "regular": "d2/FiraSans-Regular.ttf",
        "italic": "d2/FiraSans-Italic.ttf",
        "bold": "d2/FiraSans-Bold.ttf",
        "semibold": "d2/FiraSans-SemiBold.ttf"
    }

    def css_fix_rule(self, rule):
        for property in rule.style:
            if property.name == "font-family":
                if "regular" in property.value.lower():
                    property.value = "Fira Sans"
                    rule.style["font-style"] = "normal"
                if "italic" in property.value.lower():
                    property.value = "Fira Sans"
                    rule.style["font-style"] = "italic"
                if "bold" in property.value.lower():
                    property.value = "Fira Sans"
                    rule.style["font-weight"] = "bold"
                if "semibold" in property.value.lower():
                    property.value = "Fira Sans"
                    rule.style["font-weight"] = "600"
        return rule

    def d2_render(self, source):
        p = Popen(["d2/d2",
                   "--theme=1",
                   "--dark-theme=1",
                   f"--font-regular={self.fonts['regular']}",
                   f"--font-italic={self.fonts['italic']}",
                   f"--font-bold={self.fonts['bold']}",
                   f"--font-semibold={self.fonts['semibold']}",
                   "--pad=0",
                   "-"], stdout=PIPE, stdin=PIPE, stderr=DEVNULL, text=True)
        return p.communicate(input=source)[0]

    def svg_shrink(self, input: str):
        ET.register_namespace("", "http://www.w3.org/2000/svg")
        doc = ET.ElementTree(ET.fromstring(input))
        for style in doc.getroot().iter("{http://www.w3.org/2000/svg}style"):
            originalSheet = cssutils.parseString(style.text)
            finalSheet = cssutils.css.CSSStyleSheet()
            for rule in originalSheet:
                if rule.type == rule.STYLE_RULE:
                    finalSheet.add(self.css_fix_rule(rule))
                elif rule.type != rule.FONT_FACE_RULE:
                    finalSheet.add(rule)
            style.text = cssmin.cssmin(finalSheet.cssText.decode("UTF-8"))
        return ET.tostring(doc.getroot(), encoding="unicode")

    def __init__(self, md: Markdown, parser: BlockParser) -> None:
        super().__init__(parser)
        self.md = md

    def test(self, parent, block):
        return re.match(self.RE_FENCE_START, block)

    def run(self, parent, blocks):
        original_block = blocks[0]
        blocks[0] = re.sub(self.RE_FENCE_START, '', blocks[0])

        # Find block with ending fence
        for block_num, block in enumerate(blocks):
            if re.search(self.RE_FENCE_END, block):
                # remove fence
                blocks[block_num] = re.sub(self.RE_FENCE_END, '', block)

                # render content
                d2_source = d2_preamble + "\n".join(blocks[0:block_num + 1])
                svg_source = self.svg_shrink(self.d2_render(d2_source))
                el = ET.Element("figure")
                el.text = self.md.htmlStash.store(svg_source)
                parent.append(el)

                # remove used blocks
                for i in range(0, block_num + 1):
                    blocks.pop(0)
                return True
        # No closing marker!  Restore and do nothing
        blocks[0] = original_block
        return False  # equivalent to our test() routine returning False


class D2Extension(Extension):
    def extendMarkdown(self, md):
        processor = D2BlockProcessor(md, md.parser)
        md.parser.blockprocessors.register(processor, 'd2', 175)


def makeExtension(**kwargs):
    return D2Extension(**kwargs)
