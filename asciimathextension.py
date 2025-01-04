import re
import xml.etree.ElementTree as ET
from markdown.blockparser import BlockParser
from markdown.blockprocessors import BlockProcessor
from markdown import Extension, Markdown
from py_asciimath.translator.translator import ASCIIMath2MathML


class ASCIIMathBlockProcessor(BlockProcessor):
    RE_FENCE_START = r'^ *!{3,}math *\n'
    RE_FENCE_END = r'\n *!{3,}\s*$'

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
                asciimath_source = "\n".join(blocks[0:block_num + 1])
                asciimath2mathml = ASCIIMath2MathML(log=True, inplace=True)
                mathml_source = asciimath2mathml.translate(
                    asciimath_source,
                    output="string",
                    xml_pprint=False
                ).replace("<math", "<math display=\"block\" ")
                el = ET.Element("figure")
                el.text = self.md.htmlStash.store(mathml_source)
                parent.append(el)

                # remove used blocks
                for i in range(0, block_num + 1):
                    blocks.pop(0)
                return True
        # No closing marker!  Restore and do nothing
        blocks[0] = original_block
        return False  # equivalent to our test() routine returning False


class ASCIIMathExtension(Extension):
    def extendMarkdown(self, md):
        processor = ASCIIMathBlockProcessor(md, md.parser)
        md.parser.blockprocessors.register(processor, 'asciimath', 175)


def makeExtension(**kwargs):
    return ASCIIMathExtension(**kwargs)
