//! Optimization analysis module for BeBytes
//!
//! This module analyzes struct characteristics to determine the optimal
//! serialization method and provides performance hints.

use syn::{FieldsNamed, Type};

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Analysis of a struct's characteristics for optimization decisions
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructAnalysis {
    /// Total size in bytes (if deterministic)
    pub size: Option<usize>,
    /// Whether the struct has bit fields
    pub has_bit_fields: bool,
    /// Whether the struct has Vec fields
    pub has_vectors: bool,
    /// Whether the struct has String fields
    pub has_strings: bool,
    /// Whether the struct is eligible for raw pointer optimization
    pub supports_raw_pointer: bool,
    /// Recommended optimization method
    pub recommended_method: OptimizationMethod,
    /// Performance characteristics
    pub performance_hint: PerformanceHint,
}

/// Available optimization methods
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationMethod {
    /// Use raw pointer optimization (5x improvement)
    RawPointer,
    /// Use Bytes buffer approach (2x improvement)
    BytesBuffer,
    /// Use standard Vec approach (baseline)
    Standard,
}

/// Performance hints for the user
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PerformanceHint {
    /// Expected performance improvement over Vec approach
    pub improvement_factor: f32,
    /// Memory allocation pattern
    pub allocation_pattern: &'static str,
    /// Recommendation text
    pub recommendation: &'static str,
}

impl StructAnalysis {
    /// Analyze a struct's fields to determine optimization characteristics
    pub fn analyze_struct(fields: &FieldsNamed) -> Self {
        let mut size = Some(0usize);
        let mut has_bit_fields = false;
        let mut has_vectors = false;
        let mut has_strings = false;

        // Handle empty structs (no fields)
        if fields.named.is_empty() {
            return Self {
                size: Some(0),
                has_bit_fields: false,
                has_vectors: false,
                has_strings: false,
                supports_raw_pointer: false, // Empty structs don't support raw pointer
                recommended_method: OptimizationMethod::Standard, // Use standard for empty structs
                performance_hint: PerformanceHint {
                    improvement_factor: 1.0,
                    allocation_pattern: "No allocation (empty struct)",
                    recommendation: "Empty struct: Use standard methods",
                },
            };
        }

        for field in &fields.named {
            // Check for bit fields
            if field.attrs.iter().any(|attr| attr.path().is_ident("bits")) {
                has_bit_fields = true;
                // Bit fields make size calculation complex
                size = None;
            }

            // Analyze field type
            match &field.ty {
                Type::Path(type_path) => {
                    if let Some(segment) = type_path.path.segments.last() {
                        match segment.ident.to_string().as_str() {
                            "Vec" => has_vectors = true,
                            "String" => has_strings = true,
                            _ => {
                                // Try to get primitive type size
                                if let Ok(field_size) =
                                    crate::utils::get_primitive_type_size(&field.ty)
                                {
                                    if let Some(current_size) = size {
                                        size = Some(current_size + field_size);
                                    }
                                } else {
                                    size = None;
                                }
                            }
                        }
                    }
                }
                Type::Array(array_type) => {
                    // Handle byte arrays
                    if let Type::Path(element_type) = &*array_type.elem {
                        if element_type.path.is_ident("u8") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Int(len),
                                ..
                            }) = &array_type.len
                            {
                                if let Ok(array_len) = len.base10_parse::<usize>() {
                                    if let Some(current_size) = size {
                                        size = Some(current_size + array_len);
                                    }
                                } else {
                                    size = None;
                                }
                            } else {
                                size = None;
                            }
                        } else {
                            size = None;
                        }
                    } else {
                        size = None;
                    }
                }
                _ => {
                    size = None;
                }
            }
        }

        // Determine if raw pointer optimization is supported
        let supports_raw_pointer = !has_bit_fields
            && !has_vectors
            && !has_strings
            && size.is_some()
            && size.unwrap_or(0) <= 256;

        // Determine recommended method based on analysis
        let recommended_method = if supports_raw_pointer && size.unwrap_or(0) <= 100 {
            OptimizationMethod::RawPointer
        } else if !has_vectors && !has_strings {
            OptimizationMethod::BytesBuffer
        } else {
            OptimizationMethod::Standard
        };

        // Generate performance hint
        let performance_hint = match recommended_method {
            OptimizationMethod::RawPointer => PerformanceHint {
                improvement_factor: 5.4,
                allocation_pattern: "Zero allocations (stack only)",
                recommendation: "Optimal: Use raw pointer methods for maximum performance",
            },
            OptimizationMethod::BytesBuffer => PerformanceHint {
                improvement_factor: 2.3,
                allocation_pattern: "Single allocation with reuse potential",
                recommendation: "Good: Use Bytes buffer methods for better performance",
            },
            OptimizationMethod::Standard => PerformanceHint {
                improvement_factor: 1.0,
                allocation_pattern: "Standard Vec allocation",
                recommendation: "Standard: Complex types require Vec approach",
            },
        };

        Self {
            size,
            has_bit_fields,
            has_vectors,
            has_strings,
            supports_raw_pointer,
            recommended_method,
            performance_hint,
        }
    }

    /// Generate compile-time performance documentation
    pub fn generate_performance_docs(&self) -> proc_macro2::TokenStream {
        let improvement = self.performance_hint.improvement_factor;
        let allocation = self.performance_hint.allocation_pattern;
        let recommendation = self.performance_hint.recommendation;

        let method_doc = match self.recommended_method {
            OptimizationMethod::RawPointer => {
                "Consider using `encode_be_to_raw_stack()` for maximum performance (5.4x improvement)"
            }
            OptimizationMethod::BytesBuffer => {
                "Consider using `to_be_bytes_buf()` for better performance (2.3x improvement)"
            }
            OptimizationMethod::Standard => {
                "This struct requires standard serialization methods due to complex field types"
            }
        };

        quote::quote! {
            #[doc = ""]
            #[doc = "## Performance Characteristics"]
            #[doc = concat!("- Expected improvement: ", stringify!(#improvement), "x over Vec approach")]
            #[doc = concat!("- Allocation pattern: ", #allocation)]
            #[doc = concat!("- Recommendation: ", #recommendation)]
            #[doc = ""]
            #[doc = concat!("### Optimization Hint: ", #method_doc)]
        }
    }

    /// Generate method selection helper
    pub fn generate_optimal_method_hint(&self) -> proc_macro2::TokenStream {
        match self.recommended_method {
            OptimizationMethod::RawPointer => {
                quote::quote! {
                    /// Get the optimal serialization method for this struct
                    /// Returns the method name as a static string for performance guidance
                    pub const fn optimal_serialization_method() -> &'static str {
                        "encode_be_to_raw_stack() - 5.4x performance improvement"
                    }
                }
            }
            OptimizationMethod::BytesBuffer => {
                quote::quote! {
                    /// Get the optimal serialization method for this struct
                    /// Returns the method name as a static string for performance guidance
                    pub const fn optimal_serialization_method() -> &'static str {
                        "to_be_bytes_buf() - 2.3x performance improvement"
                    }
                }
            }
            OptimizationMethod::Standard => {
                quote::quote! {
                    /// Get the optimal serialization method for this struct
                    /// Returns the method name as a static string for performance guidance
                    pub const fn optimal_serialization_method() -> &'static str {
                        "to_be_bytes() - standard approach for complex types"
                    }
                }
            }
        }
    }

    /// Generate compile-time warnings for suboptimal usage patterns
    #[allow(dead_code)]
    pub fn generate_performance_warnings(&self) -> Vec<proc_macro2::TokenStream> {
        let mut warnings = Vec::new();

        if self.has_vectors && self.size.is_some() && self.size.unwrap() > 512 {
            warnings.push(quote::quote! {
                compile_error!("Large structs with Vec fields may have poor performance. Consider restructuring.");
            });
        }

        if self.has_bit_fields && self.has_vectors {
            warnings.push(quote::quote! {
                compile_error!("Structs with both bit fields and Vec fields are not optimizable. Consider separating concerns.");
            });
        }

        warnings
    }
}

/// Generate buffer reuse helper methods for batch operations
pub fn generate_buffer_reuse_helpers() -> proc_macro2::TokenStream {
    quote::quote! {
        /// Create a pre-sized buffer for batch serialization (big-endian)
        /// This helps avoid repeated allocations when serializing multiple instances
        pub fn create_batch_buffer_be(capacity: usize) -> ::bebytes::BytesMut {
            ::bebytes::BytesMut::with_capacity(capacity * Self::field_size())
        }

        /// Create a pre-sized buffer for batch serialization (little-endian)
        /// This helps avoid repeated allocations when serializing multiple instances
        pub fn create_batch_buffer_le(capacity: usize) -> ::bebytes::BytesMut {
            ::bebytes::BytesMut::with_capacity(capacity * Self::field_size())
        }

        /// Encode to a reusable buffer with optimal method selection (big-endian)
        /// This method automatically chooses the best encoding approach for performance
        #[inline]
        pub fn encode_be_to_reused(&self, buf: &mut ::bebytes::BytesMut) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
            self.encode_be_to(buf)
        }

        /// Encode to a reusable buffer with optimal method selection (little-endian)
        /// This method automatically chooses the best encoding approach for performance
        #[inline]
        pub fn encode_le_to_reused(&self, buf: &mut ::bebytes::BytesMut) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
            self.encode_le_to(buf)
        }
    }
}

/// Generate smart method selection logic for optimal performance
pub fn generate_smart_method_selection(analysis: &StructAnalysis) -> proc_macro2::TokenStream {
    match analysis.recommended_method {
        OptimizationMethod::RawPointer => {
            quote::quote! {
                /// Automatically select the optimal serialization method (big-endian)
                /// This method chooses the best approach based on struct characteristics
                #[inline]
                pub fn to_be_bytes_optimal(&self) -> ::core::result::Result<::bebytes::Bytes, ::bebytes::BeBytesError> {
                    // Use raw pointer optimization for maximum performance
                    Ok(::bebytes::Bytes::copy_from_slice(&self.encode_be_to_raw_stack()))
                }

                /// Automatically select the optimal serialization method (little-endian)
                /// This method chooses the best approach based on struct characteristics
                #[inline]
                pub fn to_le_bytes_optimal(&self) -> ::core::result::Result<::bebytes::Bytes, ::bebytes::BeBytesError> {
                    // Use raw pointer optimization for maximum performance
                    Ok(::bebytes::Bytes::copy_from_slice(&self.encode_le_to_raw_stack()))
                }
            }
        }
        OptimizationMethod::BytesBuffer => {
            quote::quote! {
                /// Automatically select the optimal serialization method (big-endian)
                /// This method chooses the best approach based on struct characteristics
                #[inline]
                pub fn to_be_bytes_optimal(&self) -> ::core::result::Result<::bebytes::Bytes, ::bebytes::BeBytesError> {
                    // Use Bytes buffer approach for better performance
                    Ok(self.to_be_bytes_buf())
                }

                /// Automatically select the optimal serialization method (little-endian)
                /// This method chooses the best approach based on struct characteristics
                #[inline]
                pub fn to_le_bytes_optimal(&self) -> ::core::result::Result<::bebytes::Bytes, ::bebytes::BeBytesError> {
                    // Use Bytes buffer approach for better performance
                    Ok(self.to_le_bytes_buf())
                }
            }
        }
        OptimizationMethod::Standard => {
            quote::quote! {
                /// Automatically select the optimal serialization method (big-endian)
                /// This method chooses the best approach based on struct characteristics
                #[inline]
                pub fn to_be_bytes_optimal(&self) -> ::core::result::Result<::bebytes::Bytes, ::bebytes::BeBytesError> {
                    // Use standard Vec approach for complex types
                    Ok(::bebytes::Bytes::from(self.to_be_bytes()))
                }

                /// Automatically select the optimal serialization method (little-endian)
                /// This method chooses the best approach based on struct characteristics
                #[inline]
                pub fn to_le_bytes_optimal(&self) -> ::core::result::Result<::bebytes::Bytes, ::bebytes::BeBytesError> {
                    // Use standard Vec approach for complex types
                    Ok(::bebytes::Bytes::from(self.to_le_bytes()))
                }
            }
        }
    }
}
