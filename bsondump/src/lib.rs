use std::io::Write;

use bson::{RawArray, RawBsonRef, RawDocument, RawDocumentBuf};
use serde::ser::Serialize;
use serde_json::{ser::PrettyFormatter, Serializer};

mod bytes;
pub mod docbytes;
use bytes::CountBytes;


fn get_indent(indent_level: usize) -> String {
    "\t".repeat(indent_level)
}

pub fn to_pretty_string(value: &serde_json::value::Value) -> std::result::Result<String, std::io::Error> {
    let mut pretty_json: Vec<u8> = Vec::new();
    let formatter = PrettyFormatter::with_indent(b"\t");
    let mut ser = Serializer::with_formatter(&mut pretty_json, formatter);
    value.serialize(&mut ser)?;
    Ok(String::from_utf8_lossy(&pretty_json).to_string())
}

pub fn to_canonical_extjson_value(
    raw_document_buf: &RawDocumentBuf,
) -> std::result::Result<serde_json::value::Value, bson::ser::Error> {
    let bson_doc: bson::Bson = bson::to_bson(&raw_document_buf)?;
    Ok(bson_doc.into_canonical_extjson())
}

pub fn debug(raw_doc: &RawDocument) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let mut buf: Vec<u8> = Vec::new();
    debug_document(&mut buf, raw_doc, 0)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn new_object_header<W: Write, O: CountBytes + ?Sized>(
    writer: &mut W,
    object: &O,
    indent_level: usize,
) -> std::result::Result<(), std::io::Error> {
    writeln!(writer, "{}--- new object ---", get_indent(indent_level))?;
    writeln!(
        writer,
        "{indent}size : {size}",
        indent = get_indent(indent_level + 1),
        size = object.count_bytes(),
    )?;
    Ok(())
}

fn debug_array<W: Write>(
    writer: &mut W,
    array: &RawArray,
    indent_level: usize,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    new_object_header(writer, array, indent_level)?;
    for (i, element) in array.into_iter().enumerate() {
        let name = i.to_string();
        let bson_ref = element?;
        debug_item(writer, &name, &bson_ref, indent_level)?;
    }
    Ok(())
}

fn debug_document<W: Write>(
    writer: &mut W,
    raw_document: &RawDocument,
    indent_level: usize,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    new_object_header(writer, raw_document, indent_level)?;
    for element in raw_document {
        let (name, bson_ref) = element?;
        debug_item(writer, name, &bson_ref, indent_level)?;
    }
    Ok(())
}

fn debug_item<W: Write>(
    writer: &mut W,
    name: &str,
    bson_ref: &RawBsonRef,
    indent_level: usize,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    writeln!(writer, "{indent}{name}", indent = get_indent(indent_level + 2), name = name,)?;
    let size_of_type = 1usize;
    let size_of_name = name.len() + 1; // null terminator
    let size = size_of_type + size_of_name + bson_ref.count_bytes();
    writeln!(
        writer,
        "{indent}type: {type:>4} size: {size}",
        indent = get_indent(indent_level + 3),
        type = bson_ref.element_type() as u8,
        size = size
    )?;
    match bson_ref {
        RawBsonRef::Document(embedded) => debug_document(writer, embedded, indent_level + 3)?,
        RawBsonRef::Array(embedded) => debug_array(writer, embedded, indent_level + 3)?,
        _ => (),
    };
    Ok(())
}
