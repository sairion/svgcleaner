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

use std::fs;
use std::io::{Read, Write};
use std::io;

use svgdom;
use svgdom::{Document, ParseOptions, WriteOptions, WriteBuffer, ElementId};

use options::Options;
use task::*;
use error;

pub fn load_file(path: &str) -> Result<Vec<u8>, io::Error> {
    let mut file = fs::File::open(path)?;
    let length = file.metadata()?.len() as usize;

    let mut v = Vec::with_capacity(length + 1);
    file.read_to_end(&mut v)?;

    Ok(v)
}

pub fn parse_data(data: &[u8], opt: &ParseOptions) -> Result<Document, svgdom::Error> {
    Document::from_data_with_opt(data, opt)
}

pub fn clean_doc(doc: &Document, options: &Options, opt: &WriteOptions)
                 -> Result<(), error::Error> {
    preclean_checks(doc)?;

    // NOTE: Order is important.
    //       Methods should not depend on each other, but for performance reasons
    //       they should be executed in this order.

    // Prepare our document.
    // This methods is not optional.
    utils::resolve_gradient_attributes(doc)?;
    svgdom::postproc::resolve_inherit(doc)?;
    fix_invalid_attributes(doc);
    group_defs(doc);

    // Manipulate with tree structure.
    // Do not remove any attributes before this methods
    // since they uses them.

    if options.remove_title {
        remove_element(doc, ElementId::Title);
    }

    if options.remove_desc {
        remove_element(doc, ElementId::Desc);
    }

    if options.remove_metadata {
        remove_element(doc, ElementId::Metadata);
    }

    if options.remove_unused_defs {
        remove_unused_defs(doc);
    }

    if options.remove_invalid_stops {
        remove_invalid_stops(doc);
    }

    if options.apply_transform_to_gradients {
        // Apply transform to gradients before processing to simplify duplicates
        // detecting and merging.
        apply_transforms::apply_transform_to_gradients(doc);
    }

    if options.remove_dupl_linear_gradients {
        remove_dupl_linear_gradients(doc);
    }

    if options.remove_dupl_radial_gradients {
        remove_dupl_radial_gradients(doc);
    }

    if options.remove_dupl_fe_gaussian_blur {
        remove_dupl_fe_gaussian_blur(doc);
    }

    if options.merge_gradients {
        merge_gradients(doc);
    }

    if options.apply_transform_to_gradients {
        // Do it again, because something may changed after gradient processing.
        apply_transforms::apply_transform_to_gradients(doc);
    }

    if options.apply_transform_to_shapes {
        // Apply before 'convert_shapes_to_paths'.
        apply_transforms::apply_transform_to_shapes(doc);
    }

    if options.convert_shapes {
        convert_shapes_to_paths(doc);
    }

    // NOTE: run before `remove_invisible_elements`, because this method can remove all
    //       segments from the path which makes it invisible.
    if options.paths_to_relative {
        // we only process path's segments if 'PathsToRelative' is enabled
        paths::process_paths(doc, options);
    }

    if options.remove_invisible_elements {
        remove_invisible_elements(doc);
    }

    if options.regroup_gradient_stops {
        regroup_gradient_stops(doc);
    }

    if options.ungroup_groups {
        ungroup_groups(doc);
    }

    if options.resolve_use {
        resolve_use(doc);
    }

    // now we can remove any unneeded attributes

    if options.remove_default_attributes {
        remove_default_attributes(doc);
    }

    if options.remove_text_attributes {
        remove_text_attributes(doc);
    }

    if options.remove_needless_attributes {
        remove_needless_attributes(doc);
    }

    if options.remove_gradient_attributes {
        remove_gradient_attributes(doc);
    }

    if options.remove_unused_coordinates {
        remove_unused_coordinates(doc);
    }

    // Run only after attributes processed, because
    // there is no point in grouping default/unneeded attributes.
    if options.group_by_style {
        group_by_style(doc);
    }

    // final fixes
    // list of things that can't break anything

    if options.remove_unreferenced_ids {
        remove_unreferenced_ids(doc);
    }

    if options.trim_ids {
        trim_ids(doc);
    }

    if options.remove_version {
        remove_version(doc);
    }

    if options.ungroup_defs {
        ungroup_defs(doc);
    }

    remove_empty_defs(doc);
    fix_xmlns_attribute(doc, options.remove_xmlns_xlink_attribute);

    // NOTE: must be run at last, since it breaks the linking.
    if options.join_style_attributes {
        join_style_attributes(doc, opt);
    }

    Ok(())
}

pub fn write_buffer(doc: &Document, opt: &WriteOptions, buf: &mut Vec<u8>) {
    doc.write_buf_opt(opt, buf);
}

pub fn write_stdout(data: &[u8]) -> Result<usize, io::Error> {
    io::stdout().write(&data)
}

pub fn save_file(data: &[u8], path: &str) -> Result<(), io::Error> {
    let mut f = fs::File::create(&path)?;
    f.write_all(&data)?;

    Ok(())
}
