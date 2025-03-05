#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::collections::HashMap;
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as HashMap, vec::Vec};

use proc_macro2::TokenStream;

#[derive(Debug, Clone)]
struct BitRange {
    start: usize,
    end: usize,
}

impl BitRange {
    fn new(pos: usize, size: usize) -> Self {
        BitRange {
            start: pos,
            end: pos + size - 1,
        }
    }

    fn overlaps(&self, other: &BitRange) -> bool {
        (self.start <= other.start && other.start <= self.end)
            || (other.start <= self.start && self.start <= other.end)
    }
}

pub fn validate_field_sequence(fields: &syn::FieldsNamed) -> Result<(), TokenStream> {
    let mut total_size = 0;
    let mut current_byte_ranges: HashMap<usize, Vec<BitRange>> = HashMap::new();

    for field in &fields.named {
        for attr in &field.attrs {
            if attr.path().is_ident("U8") {
                let mut pos = None;
                let mut size = None;

                if let Err(e) = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("pos") {
                        let content;
                        syn::parenthesized!(content in meta.input);
                        let lit: syn::LitInt = content.parse()?;
                        pos = Some(lit.base10_parse()?);
                    } else if meta.path.is_ident("size") {
                        let content;
                        syn::parenthesized!(content in meta.input);
                        let lit: syn::LitInt = content.parse()?;
                        size = Some(lit.base10_parse()?);
                    }
                    Ok(())
                }) {
                    return Err(e.to_compile_error());
                }

                if let (Some(pos), Some(size)) = (pos, size) {
                    // Check position sequence
                    if pos % 8 != total_size % 8 {
                        return Err(syn::Error::new_spanned(field, 
                            format_args!("U8 attributes must obey the sequence specified by the previous attributes. Expected position {} but got {}", 
                                total_size, 
                                pos
                            )
                        ).to_compile_error());
                    }
                    // Check for overlaps
                    let new_range = BitRange::new(pos, size);
                    let byte_idx = pos / 8;

                    if let Some(ranges) = current_byte_ranges.get(&byte_idx) {
                        for existing_range in ranges {
                            if new_range.overlaps(existing_range) {
                                return Err(
                                    syn::Error::new_spanned(field, 
                                        format_args!(
                                            "Bit ranges overlap: positions {}-{} overlap with {}-{}",
                                            new_range.start,
                                            new_range.end,
                                            existing_range.start,
                                            existing_range.end
                                        )
                                    ).to_compile_error()
                                );
                            }
                        }
                    }
                    
                    // Add new range to current byte
                    current_byte_ranges
                        .entry(byte_idx)
                        .or_default()
                        .push(new_range);

                    // Update total size
                    total_size += size;
                }
            }
        }
    }

    Ok(())
}
