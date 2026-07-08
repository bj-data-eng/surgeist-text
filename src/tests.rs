use super::layout::decoration_top;
use super::*;

#[test]
fn rejects_invalid_utf8_span_boundary() {
    let mut system = System::default();
    let mut builder = system.builder("é");
    builder.span(Range::new(1, 2), Style::default());

    let error = builder.build().expect_err("invalid range should fail");

    assert_eq!(error.code, ErrorCode::InvalidRange);
}

#[test]
fn rejects_invalid_utf8_inline_box_boundary() {
    let mut system = System::default();
    let mut builder = system.builder("é");
    builder.inline_box(InlineBox::new(
        Id::from_u64(1),
        InlineBoxKind::InFlow,
        1,
        Size::new(4.0, 4.0),
    ));

    let error = builder
        .build()
        .expect_err("invalid inline box index should fail");

    assert_eq!(error.code, ErrorCode::InvalidRange);
}

#[test]
fn source_position_accepts_only_utf8_boundaries() {
    let text = "é text";

    let position = SourcePosition::try_new(text, 2).expect("boundary should validate");
    let error = SourcePosition::try_new(text, 1).expect_err("middle of scalar should fail");

    assert_eq!(position.get(), 2);
    assert_eq!(error.code, ErrorCode::InvalidRange);
}

#[test]
fn source_range_accepts_only_ordered_utf8_boundaries() {
    let text = "é text";

    let range = SourceRange::try_new(text, 0, 2).expect("valid source range");
    let reversed = SourceRange::try_new(text, 2, 0).expect_err("reversed range should fail");
    let split = SourceRange::try_new(text, 0, 1).expect_err("split scalar should fail");

    assert_eq!(range.start().get(), 0);
    assert_eq!(range.end().get(), 2);
    assert_eq!(reversed.code, ErrorCode::InvalidRange);
    assert_eq!(split.code, ErrorCode::InvalidRange);
}

#[test]
fn invalid_range_error_names_rejected_boundary() {
    let error = SourceRange::try_new("é", 0, 1).expect_err("split scalar should fail");

    assert_eq!(error.code, ErrorCode::InvalidRange);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidSourceRange {
            start: 0,
            end: 1,
            text_len: 2,
        })
    );
}

#[test]
fn rejects_invalid_numeric_layout_options() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    builder.options(Options {
        scale: f32::NAN,
        ..Options::default()
    });

    let error = builder
        .build()
        .expect_err("invalid scale should fail before layout");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
}

#[test]
fn rejects_invalid_numeric_text_style() {
    let mut system = System::default();
    let style = Style {
        size: 0.0,
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style);

    let error = builder
        .build()
        .expect_err("invalid font size should fail before layout");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert!(error.message.contains("font size"));
}

#[test]
fn rejects_invalid_text_brush_channels() {
    let mut system = System::default();
    let style = Style {
        brush: Brush::color(1.2, 0.0, 0.0, 1.0),
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style);

    let error = builder
        .build()
        .expect_err("out-of-range brush channel should fail before layout");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert!(error.message.contains("red channel"));
}

#[test]
fn builds_plain_text_layout() {
    let mut system = System::default();
    let mut builder = system.builder("hello world");
    builder.options(Options {
        width: Some(100.0),
        ..Options::default()
    });

    let layout = builder.build().expect("layout should build");

    assert_eq!(layout.metrics().line_count(), 1);
    assert!(!layout.glyph_runs().is_empty());
}

#[test]
fn wrap_none_preserves_single_visual_line() {
    let mut system = System::default();
    let style = Style {
        wrap: Wrap::None,
        ..Style::default()
    };
    let mut builder = system.builder("hello world");
    builder.default_style(style).options(Options {
        width: Some(16.0),
        ..Options::default()
    });

    let layout = builder.build().expect("nowrap layout should build");

    assert_eq!(layout.metrics().line_count(), 1);
    assert!(layout.metrics().overflow());
}

#[test]
fn overflow_wrap_anywhere_breaks_unspaced_text() {
    let mut system = System::default();
    let mut normal = system.builder("abcdefghijklmnop");
    normal.options(Options {
        width: Some(32.0),
        ..Options::default()
    });
    let normal = normal.build().expect("normal layout should build");
    let style = Style {
        overflow_wrap: OverflowWrap::Anywhere,
        ..Style::default()
    };
    let mut anywhere = system.builder("abcdefghijklmnop");
    anywhere.default_style(style).options(Options {
        width: Some(32.0),
        ..Options::default()
    });

    let anywhere = anywhere.build().expect("anywhere layout should build");

    assert_eq!(normal.metrics().line_count(), 1);
    assert!(anywhere.metrics().line_count() > 1);
}

#[test]
fn passes_valid_locale_to_parley() {
    let mut system = System::default();
    let style = Style {
        locale: Some("en-US".to_owned()),
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style);

    let layout = builder.build().expect("valid locale should build");

    assert_eq!(layout.metrics().line_count(), 1);
}

#[test]
fn rejects_invalid_locale() {
    let mut system = System::default();
    let style = Style {
        locale: Some("not a locale".to_owned()),
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style);

    let error = builder.build().expect_err("invalid locale should fail");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
}

#[test]
fn passes_font_fallbacks_features_and_variations_to_parley() {
    let mut system = System::default();
    let style = Style {
        font: Font::new()
            .family("Arial")
            .family("serif")
            .feature(r#""liga" on"#)
            .variation(r#""wght" 700"#),
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style);

    let layout = builder.build().expect("valid font settings should build");

    assert_eq!(layout.metrics().line_count(), 1);
}

#[test]
fn numeric_font_weight_shapes_and_changes_cache_key() {
    let mut system = System::default();
    let first_style = Style {
        font: Font::new().weight(Weight::Number(
            FontWeightValue::try_new(450.0).expect("weight is valid"),
        )),
        ..Style::default()
    };
    let second_style = Style {
        font: Font::new().weight(Weight::Number(
            FontWeightValue::try_new(500.0).expect("weight is valid"),
        )),
        ..Style::default()
    };

    let mut first = system.builder("weight");
    first.default_style(first_style);
    let first = first.build().expect("numeric weight should shape");

    let mut second = system.builder("weight");
    second.default_style(second_style);
    let second = second.build().expect("numeric weight should shape");

    assert_ne!(first.key(), second.key());
}

#[test]
fn font_weight_value_rejects_invalid_values() {
    let zero = FontWeightValue::try_new(0.0).expect_err("zero weight is invalid");
    let nan = FontWeightValue::try_new(f32::NAN).expect_err("nan weight is invalid");

    assert_eq!(zero.code, ErrorCode::InvalidStyle);
    assert_eq!(nan.code, ErrorCode::InvalidStyle);
}

#[test]
fn text_style_support_reports_numeric_font_weight_supported() {
    assert_eq!(
        TextStyleFeature::NumericFontWeight.support(),
        TextStyleSupport::Supported
    );
}

#[test]
fn font_width_ratio_shapes_and_changes_cache_key() {
    let mut system = System::default();
    let condensed = Style {
        font: Font::new().width(Width::Ratio(
            FontWidthRatio::try_new(0.875).expect("width is valid"),
        )),
        ..Style::default()
    };
    let expanded = Style {
        font: Font::new().width(Width::ExtraExpanded),
        ..Style::default()
    };

    let mut first = system.builder("width");
    first.default_style(condensed);
    let first = first.build().expect("width ratio should shape");

    let mut second = system.builder("width");
    second.default_style(expanded);
    let second = second.build().expect("expanded width should shape");

    assert_ne!(first.key(), second.key());
}

#[test]
fn font_width_ratio_rejects_invalid_values() {
    let zero = FontWidthRatio::try_new(0.0).expect_err("zero width is invalid");
    let nan = FontWidthRatio::try_new(f32::NAN).expect_err("nan width is invalid");

    assert_eq!(zero.code, ErrorCode::InvalidStyle);
    assert_eq!(nan.code, ErrorCode::InvalidStyle);
}

#[test]
fn text_style_support_reports_expanded_font_stretch_supported() {
    assert_eq!(
        TextStyleFeature::ExpandedFontStretch.support(),
        TextStyleSupport::Supported
    );
}

#[test]
fn font_variant_normal_is_default_noop() {
    assert_eq!(Style::default().font.variant, FontVariant::Normal);

    let style = Style {
        font: Font::new().variant(FontVariant::Normal),
        ..Style::default()
    };

    let validated = ValidatedStyle::try_from(style).expect("normal variant should validate");

    assert_eq!(validated.authored().font.variant, FontVariant::Normal);
    assert_eq!(
        TextStyleFeature::FontVariant.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresFontPolicy)
    );
}

#[test]
fn root_must_reject_non_normal_font_variants_before_text() {
    assert_eq!(
        TextStyleFeature::FontVariant.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresFontPolicy)
    );
}

#[test]
fn rejects_invalid_font_settings() {
    let mut system = System::default();
    let style = Style {
        font: Font::new().feature("liga on"),
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style);

    let error = builder
        .build()
        .expect_err("feature tags must use CSS OpenType setting syntax");

    assert_eq!(error.code, ErrorCode::InvalidStyle);

    let mut system = System::default();
    let style = Style {
        font: Font::new().variation(r#""wght" nope"#),
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style);

    let error = builder
        .build()
        .expect_err("variation values must use CSS OpenType setting syntax");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
}

#[test]
fn validated_style_preserves_parsed_font_inputs() {
    let style = Style {
        font: Font::new()
            .family("Arial")
            .feature(r#""liga" on"#)
            .variation(r#""wght" 700"#),
        locale: Some("en-US".to_owned()),
        ..Style::default()
    };

    let validated = ValidatedStyle::try_from(style).expect("style should validate");

    assert_eq!(validated.font_families(), ["Arial".to_owned()]);
    assert_eq!(validated.locale_tag(), Some("en-US"));
    assert_eq!(validated.font_features(), [r#""liga" on"#.to_owned()]);
    assert_eq!(validated.font_variations(), [r#""wght" 700"#.to_owned()]);
    assert_eq!(
        validated.authored().font.features,
        [r#""liga" on"#.to_owned()]
    );
}

#[test]
fn validated_style_rejects_invalid_oblique_angle() {
    let style = Style {
        font: Font::new().style(Slant::Oblique(Some(f32::NAN))),
        ..Style::default()
    };

    let error = ValidatedStyle::try_from(style).expect_err("invalid oblique angle should fail");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
}

#[test]
fn invalid_style_error_names_rejected_field() {
    let style = Style {
        size: 0.0,
        ..Style::default()
    };

    let error = ValidatedStyle::try_from(style).expect_err("zero size should fail");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidNumericField {
            field: "font size",
            value: 0.0,
            requirement: NumericRequirement::FiniteGreaterThanZero,
        })
    );
}

#[test]
fn default_style_preserves_authored_whitespace() {
    assert_eq!(Style::default().white_space, WhiteSpace::Preserve);
}

#[test]
fn text_style_support_matrix_reports_current_support() {
    assert_eq!(
        TextStyleFeature::FontFamilyList.support(),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::FontSize.support(),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::WhiteSpacePreserve.support(),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::WhiteSpaceCollapse.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresSourceRangePreservation)
    );
    assert_eq!(
        TextStyleFeature::TextOverflow.support(),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresTextFlowPolicy)
    );
    assert!(
        TextStyleFeature::ALL.contains(&TextStyleFeature::SelectionColor),
        "root should be able to enumerate selection color support"
    );
}

#[test]
fn default_flow_policy_values_are_explicit_noops() {
    assert_eq!(Style::default().text_transform, TextTransform::None);
    assert_eq!(Options::default().text_overflow, TextOverflow::Clip);
    assert_eq!(Options::default().text_align_last, TextAlignLast::Auto);

    let mut system = System::default();
    let style = Style {
        text_transform: TextTransform::None,
        ..Style::default()
    };
    let options = Options {
        text_overflow: TextOverflow::Clip,
        text_align_last: TextAlignLast::Auto,
        ..Options::default()
    };
    let mut builder = system.builder("Flow 123");
    builder.default_style(style);
    builder.options(options);

    let layout = builder.build().expect("default flow policies should build");

    assert_eq!(layout.source().text(), "Flow 123");
}

#[test]
fn default_flow_policy_values_preserve_cache_identity() {
    let mut system = System::default();
    let first = system
        .builder("flow")
        .build()
        .expect("default flow policy should build");

    let mut second = system.builder("flow");
    second.default_style(Style {
        text_transform: TextTransform::None,
        ..Style::default()
    });
    second.options(Options {
        text_overflow: TextOverflow::Clip,
        text_align_last: TextAlignLast::Auto,
        ..Options::default()
    });
    let second = second
        .build()
        .expect("explicit default flow policy should build");

    assert_eq!(first.key(), second.key());
}

#[test]
fn unsupported_text_style_errors_are_typed() {
    let style = Style {
        direction: Direction::LeftToRight,
        ..Style::default()
    };

    let error =
        ValidatedStyle::try_from(style).expect_err("explicit direction should remain unsupported");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedTextStyle {
            feature: TextStyleFeature::ExplicitTextDirection,
            reason: UnsupportedTextStyleReason::RequiresParleyBaseDirection,
        })
    );
}

#[test]
fn whitespace_collapse_reports_explicit_error() {
    let mut system = System::default();
    let style = Style {
        white_space: WhiteSpace::Collapse,
        ..Style::default()
    };
    let mut builder = system.builder("  hello\t\n world  ");
    builder.default_style(style);

    let error = builder
        .build()
        .expect_err("collapse must fail until source range projection is robust");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(error.message.contains("whitespace collapse"));
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedTextStyle {
            feature: TextStyleFeature::WhiteSpaceCollapse,
            reason: UnsupportedTextStyleReason::RequiresSourceRangePreservation,
        })
    );
}

#[test]
fn span_whitespace_collapse_reports_explicit_error() {
    let mut system = System::default();
    let collapsed = Style {
        white_space: WhiteSpace::Collapse,
        ..Style::default()
    };
    let mut builder = system.builder("pre  collapse\t\n  post");
    builder.span(Range::new(3, 15), collapsed);

    let error = builder
        .build()
        .expect_err("span-level collapse must fail until source range projection is robust");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(error.message.contains("whitespace collapse"));
}

#[test]
fn default_indent_targets_first_line() {
    assert!(Indent::default().first_line);
}

#[test]
fn first_line_indent_offsets_first_line() {
    let mut system = System::default();
    let mut builder = system.builder("hello\nworld");
    builder.options(Options {
        indent: Indent {
            amount: 10.0,
            ..Indent::default()
        },
        ..Options::default()
    });

    let layout = builder.build().expect("layout should build");
    let first = layout.cursor(Cursor::new(0, Affinity::After));
    let second = layout.cursor(Cursor::new(6, Affinity::After));

    assert!(first.rect().origin.x >= 9.0);
    assert!(second.rect().origin.x < 1.0);
}

#[test]
fn first_line_false_without_other_scope_skips_indent() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    builder.options(Options {
        indent: Indent {
            amount: 10.0,
            first_line: false,
            each_line: false,
            hanging: false,
        },
        ..Options::default()
    });

    let layout = builder.build().expect("layout should build");

    assert!(
        layout
            .cursor(Cursor::new(0, Affinity::After))
            .rect()
            .origin
            .x
            < 1.0
    );
}

#[test]
fn rejects_each_line_indent_without_first_line_scope() {
    let mut system = System::default();
    let mut builder = system.builder("hello\nworld");
    builder.options(Options {
        indent: Indent {
            amount: 10.0,
            first_line: false,
            each_line: true,
            hanging: false,
        },
        ..Options::default()
    });

    let error = builder
        .build()
        .expect_err("unsupported indent combination should fail");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedTextStyle {
            feature: TextStyleFeature::TextIndent,
            reason: UnsupportedTextStyleReason::IndentShapeNotExpressibleByCurrentBackend,
        })
    );
}

#[test]
fn validated_options_reject_unsupported_indent_shape() {
    let options = Options {
        indent: Indent {
            amount: 10.0,
            first_line: false,
            each_line: true,
            hanging: false,
        },
        ..Options::default()
    };

    let error = ValidatedOptions::try_from(options).expect_err("unsupported shape should fail");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedTextStyle {
            feature: TextStyleFeature::TextIndent,
            reason: UnsupportedTextStyleReason::IndentShapeNotExpressibleByCurrentBackend,
        })
    );
}

#[test]
fn public_text_style_contract_is_enumerable() {
    let unsupported: Vec<_> = TextStyleFeature::ALL
        .iter()
        .copied()
        .filter(|feature| matches!(feature.support(), TextStyleSupport::Unsupported(_)))
        .collect();

    assert!(TextStyleFeature::ALL.contains(&TextStyleFeature::ExplicitTextDirection));
    assert!(TextStyleFeature::ALL.contains(&TextStyleFeature::WhiteSpaceCollapse));
    assert!(unsupported.contains(&TextStyleFeature::ExplicitTextDirection));
    assert!(unsupported.contains(&TextStyleFeature::WhiteSpaceCollapse));
    assert!(unsupported.contains(&TextStyleFeature::FontVariant));
    assert!(unsupported.contains(&TextStyleFeature::TextOverflow));
    assert!(unsupported.contains(&TextStyleFeature::VerticalAlign));
    assert!(unsupported.contains(&TextStyleFeature::SelectionColor));
    assert_eq!(
        TextStyleFeature::TextIndent.support(),
        TextStyleSupport::Supported,
        "text indent is generally supported; only one value shape is rejected"
    );
}

#[test]
fn auto_direction_reports_resolved_base_direction() {
    let mut system = System::default();
    let mut builder = system.builder("שלום");

    let layout = builder.build().expect("layout should build");

    assert_eq!(layout.direction(), Direction::RightToLeft);
}

#[test]
fn rejects_explicit_base_direction_until_parley_exposes_it() {
    let mut system = System::default();
    let style = Style {
        direction: Direction::RightToLeft,
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style);

    let error = builder
        .build()
        .expect_err("explicit direction should fail loudly");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
}

#[test]
fn repeated_layout_uses_cache() {
    let mut system = System::default();

    let first = system
        .builder("hello")
        .build()
        .expect("first layout should build");
    let second = system
        .builder("hello")
        .build()
        .expect("second layout should build from cache");

    assert_eq!(first.key(), second.key());
    assert_eq!(system.stats().layout_misses(), 1);
    assert_eq!(system.stats().layout_hits(), 1);
}

#[test]
fn validated_source_snapshot_rejects_invalid_boxes_before_cache_keying() {
    let mut source = Source::new("é");
    source.inline_box(InlineBox::new(
        Id::from_u64(11),
        InlineBoxKind::InFlow,
        1,
        Size::new(1.0, 1.0),
    ));

    let error = ValidatedSource::try_from(source).expect_err("invalid box anchor should fail");

    assert_eq!(error.code, ErrorCode::InvalidRange);
}

#[test]
fn validated_source_snapshot_rejects_invalid_inline_box_size_before_cache_keying() {
    let mut source = Source::new("hello");
    source.inline_box(InlineBox::new(
        Id::from_u64(12),
        InlineBoxKind::InFlow,
        2,
        Size::new(-1.0, 1.0),
    ));

    let error = ValidatedSource::try_from(source).expect_err("invalid box size should fail");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidNumericField {
            field: "inline box width",
            value: -1.0,
            requirement: NumericRequirement::FiniteNonNegative,
        })
    );
}

#[test]
fn system_build_rejects_invalid_inline_box_size_before_projection() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    builder.inline_box(InlineBox::new(
        Id::from_u64(13),
        InlineBoxKind::InFlow,
        2,
        Size::new(1.0, f32::NAN),
    ));

    let error = builder
        .build()
        .expect_err("invalid box size should fail before layout");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert!(error.message.contains("inline box height"));
}

#[test]
fn validated_source_snapshot_rejects_invalid_span_styles_before_cache_keying() {
    let mut source = Source::new("hello");
    source.span(
        Range::new(0, 5),
        Style {
            size: 0.0,
            ..Style::default()
        },
    );

    let error = ValidatedSource::try_from(source).expect_err("invalid span style should fail");

    assert_eq!(error.code, ErrorCode::InvalidStyle);
}

#[test]
fn cache_key_requires_normalized_parts() {
    let source = ValidatedSource::try_from(Source::identified("hello", Id::from_u64(1), 2))
        .expect("source validates");
    let style = ValidatedStyle::try_from(Style::default()).expect("style validates");
    let options = ValidatedOptions::try_from(Options::default()).expect("options validate");

    let key = Key::from_validated(&source, &style, options, FontGeneration::initial());

    assert_eq!(key.source().id(), Some(Id::from_u64(1)));
    assert_eq!(key.source().revision().get(), 2);
}

#[test]
fn source_identity_and_revision_participate_in_cache_key() {
    let mut system = System::default();

    let mut first = system.builder("hello");
    first.identity(Id::from_u64(42), 1);
    let first_layout = first.build().expect("first layout should build");

    let mut second = system.builder("hello");
    second.identity(Id::from_u64(42), 2);
    let second_layout = second.build().expect("second layout should build");

    assert_eq!(first_layout.key().source().id(), Some(Id::from_u64(42)));
    assert_eq!(first_layout.key().source().revision().get(), 1);
    assert_eq!(second_layout.key().source().id(), Some(Id::from_u64(42)));
    assert_eq!(second_layout.key().source().revision().get(), 2);
    assert_ne!(first_layout.key(), second_layout.key());
    assert_eq!(system.stats().layout_misses(), 2);
    assert_eq!(system.stats().layout_hits(), 0);
}

#[test]
fn projection_outputs_expose_semantic_accessors() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");

    let metrics = layout.metrics();
    let line = layout.lines().into_iter().next().expect("line exists");
    let run = layout.glyph_runs().into_iter().next().expect("run exists");

    assert_eq!(metrics.line_count(), 1);
    assert_eq!(line.range(), Range::new(0, 5));
    assert!(run.font_size().is_finite());
}

#[test]
fn cache_stats_expose_counters_without_field_access() {
    let mut system = System::default();
    system.builder("hello").build().expect("layout builds");

    let stats = system.stats();

    assert_eq!(stats.layout_misses(), 1);
    assert_eq!(stats.layout_hits(), 0);
}

#[test]
fn layout_preserves_authored_source_identity() {
    let mut system = System::default();
    let mut source = Source::identified("  hello  ", Id::from_u64(9), 7);
    source.set_identity(Some(Id::from_u64(9)), 8);

    let layout = system
        .layout(source, Style::default(), Options::default())
        .expect("layout should build");

    assert_eq!(layout.source().id(), Some(Id::from_u64(9)));
    assert_eq!(layout.source().revision(), 8);
    assert_eq!(layout.source().text(), "  hello  ");
}

#[test]
fn style_span_changes_cache_key() {
    let mut system = System::default();
    let mut first = system.builder("hello");
    let first_style = Style {
        underline: Decoration::solid(None),
        ..Style::default()
    };
    first.span(Range::new(0, 5), first_style);
    let first_layout = first.build().expect("first layout should build");

    let mut second = system.builder("hello");
    let second_style = Style {
        strikethrough: Decoration::solid(None),
        ..Style::default()
    };
    second.span(Range::new(0, 5), second_style);
    let second_layout = second.build().expect("second layout should build");

    assert_ne!(first_layout.key(), second_layout.key());
    assert_eq!(system.stats().layout_misses(), 2);
    assert_eq!(system.stats().layout_hits(), 0);
}

#[test]
fn overlapping_spans_resolve_in_declaration_order() {
    let mut system = System::default();
    let first_brush = Brush::color(1.0, 0.0, 0.0, 1.0);
    let second_brush = Brush::color(0.0, 0.0, 1.0, 1.0);
    let first_style = Style {
        brush: first_brush,
        ..Style::default()
    };
    let second_style = Style {
        brush: second_brush,
        ..Style::default()
    };
    let mut builder = system.builder("abcd");
    builder
        .span(Range::new(0, 4), first_style)
        .span(Range::new(1, 3), second_style);

    let layout = builder.build().expect("layout should build");
    let overlap_run = layout
        .glyph_runs()
        .into_iter()
        .find(|run| {
            run.glyphs()
                .iter()
                .any(|glyph| glyph.range() == Range::new(1, 2))
        })
        .expect("overlap glyph run should exist");

    assert_eq!(overlap_run.brush(), second_brush);
    assert_eq!(overlap_run.style().brush, second_brush);
}

#[test]
fn font_refresh_invalidates_cached_layouts() {
    let mut system = System::default();
    system
        .builder("hello")
        .build()
        .expect("layout should build");

    system.refresh_fonts().expect("font refresh should succeed");
    system
        .builder("hello")
        .build()
        .expect("layout after refresh should build");

    assert_eq!(system.stats().font_refreshes(), 1);
    assert_eq!(system.stats().invalidations(), 1);
    assert_eq!(system.stats().layout_misses(), 2);
    assert_eq!(system.stats().layout_hits(), 0);
}

#[test]
fn selection_geometry_for_non_empty_range() {
    let mut system = System::default();
    let mut builder = system.builder("hello world");
    let layout = builder.build().expect("layout should build");

    let selection = Selection::new(
        Cursor::new(0, Affinity::After),
        Cursor::new(5, Affinity::Before),
    );

    assert!(!layout.selection(selection).rects().is_empty());
}

#[test]
fn selection_geometry_for_multi_line_range() {
    let mut system = System::default();
    let mut builder = system.builder("hello world");
    builder.options(Options {
        width: Some(48.0),
        ..Options::default()
    });
    let layout = builder.build().expect("layout should build");
    let selection = Selection::new(
        Cursor::new(0, Affinity::After),
        Cursor::new(layout.source().text().len(), Affinity::Before),
    );
    let geometry = layout.selection(selection);

    assert!(
        geometry.rects().len() >= 2,
        "wrapped selection should produce geometry on multiple lines"
    );
}

#[test]
fn cursor_geometry_for_empty_text() {
    let mut system = System::default();
    let mut builder = system.builder("");
    let layout = builder.build().expect("empty layout should build");
    let cursor = layout.cursor(Cursor::new(0, Affinity::After));

    assert!(cursor.rect().size.height > 0.0);
}

#[test]
fn cursor_geometry_for_bidi_boundary() {
    let mut system = System::default();
    let mut builder = system.builder("abc שלום def");
    let layout = builder.build().expect("layout should build");
    let before_rtl = layout.cursor(Cursor::new(4, Affinity::After));
    let after_rtl = layout.cursor(Cursor::new("abc שלום".len(), Affinity::Before));

    assert!(before_rtl.rect().size.height > 0.0);
    assert!(after_rtl.rect().size.height > 0.0);
    assert!(before_rtl.rect().origin.x.is_finite());
    assert!(after_rtl.rect().origin.x.is_finite());
}

#[test]
fn selection_geometry_for_bidi_range() {
    let mut system = System::default();
    let mut builder = system.builder("abc שלום def");
    let layout = builder.build().expect("layout should build");
    let selection = Selection::new(
        Cursor::new(0, Affinity::After),
        Cursor::new(layout.source().text().len(), Affinity::Before),
    );
    let geometry = layout.selection(selection);

    assert!(!geometry.rects().is_empty());
    assert!(
        geometry
            .rects()
            .iter()
            .all(|rect| rect.rect().origin.x.is_finite() && rect.rect().size.width.is_finite())
    );
}

#[test]
fn inline_box_participates_in_layout() {
    let mut system = System::default();
    let mut builder = system.builder("hello world");
    builder.inline_box(InlineBox::new(
        Id::from_u64(7),
        InlineBoxKind::InFlow,
        5,
        Size::new(20.0, 10.0),
    ));

    let layout = builder.build().expect("layout should build");
    let boxes = layout.inline_boxes();

    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].id(), Id::from_u64(7));
}

#[test]
fn out_of_flow_inline_box_preserves_metrics_and_reports_anchor() {
    let mut system = System::default();
    let plain = system
        .builder("hello world")
        .build()
        .expect("plain layout should build");
    let plain_metrics = plain.metrics();
    let mut builder = system.builder("hello world");
    builder.inline_box(InlineBox::new(
        Id::from_u64(8),
        InlineBoxKind::OutOfFlow,
        5,
        Size::new(20.0, 10.0),
    ));

    let layout = builder.build().expect("layout should build");
    let boxes = layout.inline_boxes();

    assert_eq!(layout.metrics().size(), plain_metrics.size());
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].id(), Id::from_u64(8));
    assert_eq!(boxes[0].kind(), InlineBoxKind::OutOfFlow);
    assert_eq!(boxes[0].index(), 5);
    assert_eq!(boxes[0].rect().size, Size::new(20.0, 10.0));
}

#[test]
fn hit_detects_inline_box() {
    let mut system = System::default();
    let mut builder = system.builder("hello world");
    builder.inline_box(InlineBox::new(
        Id::from_u64(7),
        InlineBoxKind::InFlow,
        5,
        Size::new(20.0, 10.0),
    ));
    let layout = builder.build().expect("layout should build");
    let box_rect = layout.inline_boxes()[0].rect();

    assert_eq!(layout.hit(box_rect.origin), Hit::InlineBox(Id::from_u64(7)));
}

#[test]
fn hit_detects_text_and_empty_space() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");

    assert!(matches!(layout.hit(Point::new(1.0, 1.0)), Hit::Text(_)));
    assert_eq!(layout.hit(Point::new(-1.0, -1.0)), Hit::None);
    assert_eq!(layout.hit(Point::new(10_000.0, 10_000.0)), Hit::None);
}

#[test]
fn movement_handles_clusters() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");

    let moved = layout.move_cursor(Cursor::new(0, Affinity::After), Movement::NextCluster);

    assert!(moved.index() > 0);
}

#[test]
fn glyphs_report_cluster_source_ranges() {
    let mut system = System::default();
    let mut builder = system.builder("ab");
    let layout = builder.build().expect("layout should build");
    let runs = layout.glyph_runs();
    let ranges = runs
        .iter()
        .flat_map(|run| run.glyphs().iter().map(|glyph| glyph.range()))
        .collect::<Vec<_>>();

    assert!(ranges.contains(&Range::new(0, 1)));
    assert!(ranges.contains(&Range::new(1, 2)));
}

#[test]
fn clusters_report_source_ranges_and_bounds() {
    let mut system = System::default();
    let mut builder = system.builder("ab");
    let layout = builder.build().expect("layout should build");

    let clusters = layout.clusters();

    assert!(
        clusters
            .iter()
            .any(|cluster| cluster.range() == Range::new(0, 1))
    );
    assert!(
        clusters
            .iter()
            .any(|cluster| cluster.range() == Range::new(1, 2))
    );
    assert!(
        clusters
            .iter()
            .all(|cluster| cluster.bounds().size.width >= 0.0)
    );
    assert!(
        clusters
            .iter()
            .all(|cluster| cluster.bounds().size.height >= 0.0)
    );
}

#[test]
fn movement_handles_line_and_document_boundaries() {
    let mut system = System::default();
    let mut builder = system.builder("alpha\nbeta\ngamma");
    let layout = builder.build().expect("layout should build");

    assert_eq!(
        layout.move_cursor(Cursor::new(8, Affinity::After), Movement::LineStart),
        Cursor::new(6, Affinity::After)
    );
    assert_eq!(
        layout.move_cursor(Cursor::new(8, Affinity::After), Movement::LineEnd),
        Cursor::new(10, Affinity::Before)
    );
    assert_eq!(
        layout.move_cursor(Cursor::new(8, Affinity::After), Movement::DocumentStart),
        Cursor::new(0, Affinity::After)
    );
    assert_eq!(
        layout.move_cursor(Cursor::new(8, Affinity::After), Movement::DocumentEnd),
        Cursor::new("alpha\nbeta\ngamma".len(), Affinity::Before)
    );
}

#[test]
fn movement_handles_previous_and_next_line() {
    let mut system = System::default();
    let mut builder = system.builder("alpha\nbeta\ngamma");
    let layout = builder.build().expect("layout should build");

    let previous = layout.move_cursor(Cursor::new(8, Affinity::After), Movement::PreviousLine);
    let next = layout.move_cursor(Cursor::new(8, Affinity::After), Movement::NextLine);

    assert!(previous.index() < 6);
    assert!(next.index() > 10);
}

#[test]
fn movement_can_extend_selection() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");
    let selection = Selection::collapsed(Cursor::new(0, Affinity::After));

    let extended = layout.move_selection(selection, Movement::NextCluster, true);

    assert_eq!(extended.anchor(), Cursor::new(0, Affinity::After));
    assert!(extended.focus().index() > extended.anchor().index());
    assert!(!extended.is_collapsed());
}

#[test]
fn movement_without_extend_collapses_selection() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");
    let selection = Selection::new(
        Cursor::new(0, Affinity::After),
        Cursor::new(5, Affinity::Before),
    );

    let moved = layout.move_selection(selection, Movement::PreviousCluster, false);

    assert!(moved.is_collapsed());
    assert!(moved.focus().index() < 5);
}

#[test]
fn edits_validate_ranges() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");

    let replaced = layout
        .try_apply(Edit::Replace {
            range: Range::new(1, 4),
            text: "ipp".to_owned(),
        })
        .expect("valid replace should apply");
    let deleted = layout
        .try_apply(Edit::Delete {
            range: Range::new(1, 4),
        })
        .expect("valid delete should apply");
    let error = layout
        .try_apply(Edit::Delete {
            range: Range::new(1, 99),
        })
        .expect_err("invalid delete should fail");
    let insert_error = layout
        .try_apply(Edit::Insert {
            index: 99,
            text: "!".to_owned(),
        })
        .expect_err("invalid insert should fail");

    assert_eq!(replaced.text(), "hippo");
    assert_eq!(deleted.text(), "ho");
    assert_eq!(error.code, ErrorCode::InvalidRange);
    assert_eq!(insert_error.code, ErrorCode::InvalidRange);
}

#[test]
fn text_edit_normalizes_insert_replace_and_delete() {
    let source = Source::identified("hello", Id::from_u64(4), 10);

    let insert = TextEdit::insert(&source, 2, "y").expect("insert validates");
    let replace = TextEdit::replace(&source, Range::new(1, 4), "ipp").expect("replace validates");
    let delete = TextEdit::delete(&source, Range::new(1, 4)).expect("delete validates");

    assert_eq!(insert.range(), Range::new(2, 2));
    assert_eq!(insert.inserted_text(), "y");
    assert_eq!(replace.range(), Range::new(1, 4));
    assert_eq!(replace.inserted_text(), "ipp");
    assert_eq!(delete.range(), Range::new(1, 4));
    assert_eq!(delete.inserted_text(), "");
}

#[test]
fn text_edit_application_advances_source_revision_once() {
    let source = Source::identified("hello", Id::from_u64(4), 10);
    let edit = TextEdit::insert(&source, 2, "y").expect("insert validates");

    let edited = edit
        .apply_to(source)
        .expect("edit applies to original source");

    assert_eq!(edited.text(), "heyllo");
    assert_eq!(edited.revision(), 11);
}

#[test]
fn text_edit_revalidates_target_source_before_applying() {
    let source = Source::new("hello");
    let edit = TextEdit::replace(&source, Range::new(1, 4), "ipp").expect("replace validates");

    let error = edit
        .apply_to(Source::new("é"))
        .expect_err("edit range should be invalid for different source");

    assert_eq!(error.code, ErrorCode::InvalidRange);
}

#[test]
fn edits_project_ranges_and_revision() {
    let mut system = System::default();
    let mut builder = system.builder("abcdef");
    builder
        .identity(Id::from_u64(3), 4)
        .span(Range::new(2, 5), Style::default())
        .inline_box(InlineBox::new(
            Id::from_u64(9),
            InlineBoxKind::OutOfFlow,
            5,
            Size::new(1.0, 1.0),
        ));
    let layout = builder.build().expect("layout should build");

    let replaced = layout
        .try_apply(Edit::Replace {
            range: Range::new(1, 3),
            text: "XXYY".to_owned(),
        })
        .expect("replace should apply");
    let deleted = layout
        .try_apply(Edit::Delete {
            range: Range::new(1, 4),
        })
        .expect("delete should apply");
    let inserted = layout
        .try_apply(Edit::Insert {
            index: 3,
            text: "ZZ".to_owned(),
        })
        .expect("insert should apply");

    assert_eq!(replaced.text(), "aXXYYdef");
    assert_eq!(replaced.revision(), 5);
    assert_eq!(replaced.spans()[0].range(), Range::new(1, 7));
    assert_eq!(replaced.boxes()[0].index(), 7);
    assert_eq!(deleted.text(), "aef");
    assert_eq!(deleted.revision(), 5);
    assert_eq!(deleted.spans()[0].range(), Range::new(1, 2));
    assert_eq!(deleted.boxes()[0].index(), 2);
    assert_eq!(inserted.text(), "abcZZdef");
    assert_eq!(inserted.revision(), 5);
    assert_eq!(inserted.spans()[0].range(), Range::new(2, 7));
    assert_eq!(inserted.boxes()[0].index(), 7);
}

#[test]
fn edit_insert_targets_source_index() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    builder.identity(Id::from_u64(5), 9);
    let layout = builder.build().expect("layout should build");

    let edited = layout
        .try_apply(Edit::Insert {
            index: 2,
            text: "y".to_owned(),
        })
        .expect("insert should apply");

    assert_eq!(edited.text(), "heyllo");
    assert_eq!(edited.revision(), 10);
}

#[test]
fn glyph_runs_preserve_resolved_brush() {
    let mut system = System::default();
    let brush = Brush::color(0.2, 0.4, 0.6, 1.0);
    let decoration_brush = Brush::color(0.7, 0.2, 0.1, 1.0);
    let style = Style {
        size: 20.0,
        brush,
        underline: Decoration::solid(Some(decoration_brush)),
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style.clone());
    let layout = builder.build().expect("layout should build");

    assert!(
        layout
            .glyph_runs()
            .iter()
            .any(|run| run.brush() == brush && run.style() == &style)
    );
}

#[test]
fn glyph_runs_expose_resolved_font_data() {
    let mut system = System::default();
    let mut builder = system.builder("effect text");
    let layout = builder.build().expect("layout should build");
    let runs = layout.glyph_runs();
    let data = runs
        .iter()
        .find_map(|run| run.font().data())
        .expect("glyph runs should expose resolved font data");

    assert!(!data.bytes().is_empty());
    assert_eq!(data.index(), 0);
}

#[test]
fn composer_push_appends_text_and_returns_range() {
    let mut composer = compose();

    let first = composer.push("é");
    let second = composer.push(" text");
    let source = composer.finish();

    assert_eq!(source.text(), "é text");
    assert_eq!(first.range(), Range::new(0, 2));
    assert_eq!(second.range(), Range::new(2, 7));
}

#[test]
fn composer_source_matches_manual_source() {
    let style = Style {
        brush: Brush::color(1.0, 0.0, 0.0, 1.0),
        ..Style::default()
    };

    let composed = source(|t| {
        t.push("plain ");
        t.with(style.clone(), |t| {
            t.push("styled");
        });
    });

    let mut manual = Source::new("");
    manual.push("plain ");
    let styled = manual.push("styled");
    manual.span(styled, style);

    assert_eq!(composed, manual);
}

#[test]
fn composer_with_captures_only_text_added_inside_closure() {
    let style = Style {
        brush: Brush::color(0.0, 0.0, 1.0, 1.0),
        ..Style::default()
    };

    let composed = source(|t| {
        t.push("before ");
        let mark = t.with(style.clone(), |t| {
            t.push("inside");
        });
        t.push(" after");

        assert_eq!(mark.range(), Range::new(7, 13));
    });

    assert_eq!(composed.text(), "before inside after");
    assert_eq!(composed.spans(), &[Span::new(Range::new(7, 13), style)]);
}

#[test]
fn composer_nested_with_orders_outer_before_inner() {
    let outer = Style {
        brush: Brush::color(1.0, 0.0, 0.0, 1.0),
        ..Style::default()
    };
    let inner = Style {
        brush: Brush::color(0.0, 0.0, 1.0, 1.0),
        ..Style::default()
    };

    let composed = source(|t| {
        t.with(outer.clone(), |t| {
            t.push("a");
            t.with(inner.clone(), |t| {
                t.push("b");
            });
            t.push("c");
        });
    });

    assert_eq!(composed.text(), "abc");
    assert_eq!(
        composed.spans(),
        &[
            Span::new(Range::new(0, 3), outer),
            Span::new(Range::new(1, 2), inner),
        ]
    );
}

#[test]
fn composer_with_can_produce_empty_mark() {
    let style = Style::default();

    let composed = source(|t| {
        t.push("a");
        let mark = t.with(style.clone(), |_t| {});

        assert!(mark.is_empty());
        assert_eq!(mark.range(), Range::new(1, 1));
    });

    assert_eq!(composed.spans(), &[Span::new(Range::new(1, 1), style)]);
}

#[test]
fn composer_box_inserts_inline_box_at_current_end() {
    let composed = source(|t| {
        t.push("before");
        t.box_(
            Id::from_u64(7),
            InlineBoxKind::InFlow,
            Size::new(16.0, 20.0),
        );
        t.push("after");
    });

    assert_eq!(
        composed.boxes(),
        &[InlineBox::new(
            Id::from_u64(7),
            InlineBoxKind::InFlow,
            6,
            Size::new(16.0, 20.0)
        )]
    );
}

#[test]
fn composer_try_span_rejects_invalid_ranges() {
    let mut composer = compose();
    composer.push("é");

    let error = composer
        .try_span(Range::new(1, 2), Style::default())
        .expect_err("non-boundary range should fail");

    assert_eq!(error.code, ErrorCode::InvalidRange);
}

#[test]
fn composer_try_inline_box_rejects_invalid_indices() {
    let mut composer = compose();
    composer.push("é");

    let error = composer
        .try_inline_box(InlineBox::new(
            Id::from_u64(1),
            InlineBoxKind::OutOfFlow,
            1,
            Size::new(4.0, 4.0),
        ))
        .expect_err("non-boundary inline box index should fail");

    assert_eq!(error.code, ErrorCode::InvalidRange);
}

#[test]
fn composer_identity_and_revision_update_source_fields() {
    let mut composer = compose();
    composer.identity(Id::from_u64(42), 7).revision(8);
    composer.push("hello");
    let source = composer.finish();

    assert_eq!(source.id(), Some(Id::from_u64(42)));
    assert_eq!(source.revision(), 8);
}

#[test]
fn composed_source_builds_through_layout_system() {
    let mut system = System::default();
    let source = source(|t| {
        t.push("hello ");
        t.with(Style::default(), |t| {
            t.push("layout");
        });
    });

    let layout = system
        .layout(source, Style::default(), Options::default())
        .expect("composed source should build");

    assert_eq!(layout.metrics().line_count(), 1);
}

#[cfg(feature = "text-render")]
#[test]
fn render_projection_encodes_prepared_glyph_runs() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");
    let mut scene = surgeist_render::Scene::new();

    layout.push_render_text(&mut scene, surgeist_render::Transform::identity());

    assert!(!scene.is_empty());
}

#[cfg(feature = "text-render")]
#[test]
fn render_projection_draws_prepared_glyph_runs() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");
    let mut scene = surgeist_render::Scene::new();

    layout.push_render_text(&mut scene, surgeist_render::Transform::identity());

    let mut renderer = pollster::block_on(surgeist_render::Renderer::new(
        surgeist_render::Options::default(),
    ))
    .expect("renderer should initialize");
    let mut surface = renderer
        .create_headless(
            surgeist_render::Size::try_new(64.0, 32.0)
                .expect("headless test surface size is valid"),
            1.0,
        )
        .expect("headless surface should initialize");
    let stats = renderer
        .render(&mut surface, &scene, surgeist_render::Parameters::default())
        .expect("prepared text should render with resolved font data");

    assert!(stats.glyphs > 0);
}

#[cfg(feature = "text-render")]
#[test]
fn render_projection_encodes_decorations() {
    let mut system = System::default();
    let style = Style {
        underline: Decoration::solid(Some(Brush::color(1.0, 0.0, 0.0, 1.0))),
        ..Style::default()
    };
    let mut builder = system.builder("hello");
    builder.default_style(style);
    let layout = builder.build().expect("layout should build");
    let mut scene = surgeist_render::Scene::new();

    layout.push_render_text(&mut scene, surgeist_render::Transform::identity());

    assert!(
        scene.len() > layout.glyph_runs().len(),
        "decoration fills should be encoded in addition to glyph runs"
    );
}

#[test]
fn decoration_offsets_are_measured_from_baseline() {
    assert_eq!(decoration_top(20.0, 3.0), 17.0);
}

#[cfg(feature = "text-accessibility")]
#[test]
fn accessibility_projection_preserves_cursor_and_selection() {
    let mut system = System::default();
    let mut builder = system.builder("hello");
    let layout = builder.build().expect("layout should build");
    let mut accessibility = Accessibility::default();
    let parent = accesskit::NodeId(100);

    let update = layout.accessibility_update(&mut accessibility, parent, parent, Point::default());
    let position = layout
        .access_position(&accessibility, Cursor::new(0, Affinity::After))
        .expect("cursor should map to AccessKit");
    let selection = layout
        .access_selection(
            &accessibility,
            Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(5, Affinity::Before),
            ),
        )
        .expect("selection should map to AccessKit");

    assert!(update.nodes.iter().any(|(id, _)| *id == parent));
    assert_eq!(position.character_index, 0);
    assert_eq!(selection.anchor.character_index, 0);
    assert!(selection.focus.character_index > 0);
}
