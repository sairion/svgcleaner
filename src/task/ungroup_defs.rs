/****************************************************************************
**
** svgcleaner could help you to clean up your SVG files
** from unnecessary data.
** Copyright (C) 2012-2017 Evgeniy Reizner
**
** This program is free software; you can redistribute it and/or modify
** it under the terms of the GNU General Public License as published by
** the Free Software Foundation; either version 2 of the License, or
** (at your option) any later version.
**
** This program is distributed in the hope that it will be useful,
** but WITHOUT ANY WARRANTY; without even the implied warranty of
** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
** GNU General Public License for more details.
**
** You should have received a copy of the GNU General Public License along
** with this program; if not, write to the Free Software Foundation, Inc.,
** 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
**
****************************************************************************/

use task::short::EId;

use svgdom::{Document, Node, ElementType};

pub fn ungroup_defs(doc: &Document) {
    for node in doc.descendants().svg().filter(|n| n.is_tag_name(EId::Defs)) {
        let mut is_valid = true;
        for child in node.children() {
            if !child.is_referenced() {
                is_valid = false;
                break;
            }
        }

        if is_valid {
            let nodes: Vec<Node> = node.children().collect();
            for n in nodes {
                n.detach();
                node.insert_before(&n);
            }

            node.remove();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use svgdom::{Document, WriteToString};

    macro_rules! test {
        ($name:ident, $in_text:expr, $out_text:expr) => (
            base_test!($name, ungroup_defs, $in_text, $out_text);
        )
    }

    macro_rules! test_eq {
        ($name:ident, $in_text:expr) => (
            test!($name, $in_text, String::from_utf8_lossy($in_text));
        )
    }

    test!(ungroup_1,
b"<svg>
    <defs>
        <linearGradient/>
    </defs>
</svg>",
"<svg>
    <linearGradient/>
</svg>
");

    test!(ungroup_2,
b"<svg>
    <defs>
        <linearGradient/>
    </defs>
    <defs>
        <linearGradient/>
    </defs>
</svg>",
"<svg>
    <linearGradient/>
    <linearGradient/>
</svg>
");

    test!(ungroup_3,
b"<svg>
    <defs>
        <linearGradient/>
        <radialGradient/>
    </defs>
</svg>",
"<svg>
    <linearGradient/>
    <radialGradient/>
</svg>
");

    test_eq!(keep_1,
b"<svg>
    <defs>
        <linearGradient/>
        <rect/>
    </defs>
</svg>
");

}
