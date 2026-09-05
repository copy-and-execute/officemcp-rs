use serde::{Deserialize, Serialize};

// ---- Formatting ----
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextFormatting {
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub strikethrough: bool,
    #[serde(default)]
    pub font_name: Option<String>,
    #[serde(default)]
    pub font_size: Option<i32>,
    #[serde(default)]
    pub font_color: Option<String>,
    #[serde(default)]
    pub highlight_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParagraphFormatting {
    #[serde(default = "default_alignment")]
    pub alignment: String,
    #[serde(default)]
    pub line_spacing: Option<f64>,
    #[serde(default)]
    pub spacing_before: Option<f64>,
    #[serde(default)]
    pub spacing_after: Option<f64>,
    #[serde(default)]
    pub first_line_indent: Option<f64>,
    #[serde(default)]
    pub left_indent: Option<f64>,
    #[serde(default)]
    pub right_indent: Option<f64>,
}
fn default_alignment() -> String { "Left".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: String,
    #[serde(default)]
    pub text_format: Option<TextFormatting>,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default = "default_left")]
    pub horizontal_alignment: String,
    #[serde(default = "default_center")]
    pub vertical_alignment: String,
    #[serde(default = "default_one")]
    pub column_span: i32,
    #[serde(default = "default_one")]
    pub row_span: i32,
}
fn default_left() -> String { "Left".to_string() }
fn default_center() -> String { "Center".to_string() }
fn default_one() -> i32 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableFormatting {
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default = "default_border_width")]
    pub border_width: f64,
    #[serde(default = "default_true")]
    pub has_header: bool,
    #[serde(default)]
    pub header_background_color: Option<String>,
    #[serde(default)]
    pub alternate_row_color: Option<String>,
}
fn default_border_width() -> f64 { 1.0 }
fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOptions {
    #[serde(default = "default_emu")]
    pub width_emu: i64,
    #[serde(default = "default_emu")]
    pub height_emu: i64,
    #[serde(default)]
    pub alt_text: Option<String>,
    #[serde(default = "default_inline")]
    pub positioning: String,
    #[serde(default = "default_rectangle")]
    pub crop_shape: String,
    #[serde(default)]
    pub rotation: f64,
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default)]
    pub border_width: f64,
    #[serde(default)]
    pub has_shadow: bool,
    #[serde(default)]
    pub perspective_3d_angle_y: f64,
    #[serde(default)]
    pub perspective_3d_angle_x: f64,
}
fn default_emu() -> i64 { 914400 }
fn default_inline() -> String { "Inline".to_string() }
fn default_rectangle() -> String { "Rectangle".to_string() }
impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            width_emu: 914400,
            height_emu: 914400,
            alt_text: None,
            positioning: "Inline".to_string(),
            crop_shape: "Rectangle".to_string(),
            rotation: 0.0,
            border_color: None,
            border_width: 0.0,
            has_shadow: false,
            perspective_3d_angle_y: 0.0,
            perspective_3d_angle_x: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeaderFooterOptions {
    #[serde(default)]
    pub left_content: Option<String>,
    #[serde(default)]
    pub center_content: Option<String>,
    #[serde(default)]
    pub right_content: Option<String>,
    #[serde(default)]
    pub include_page_number: bool,
    #[serde(default)]
    pub include_date: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLayoutOptions {
    #[serde(default = "default_portrait")]
    pub orientation: String,
    #[serde(default)]
    pub margin_top: Option<f64>,
    #[serde(default)]
    pub margin_bottom: Option<f64>,
    #[serde(default)]
    pub margin_left: Option<f64>,
    #[serde(default)]
    pub margin_right: Option<f64>,
    #[serde(default = "default_letter")]
    pub page_size: String,
}
fn default_portrait() -> String { "Portrait".to_string() }
fn default_letter() -> String { "Letter".to_string() }
impl Default for PageLayoutOptions {
    fn default() -> Self {
        Self {
            orientation: "Portrait".to_string(),
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            page_size: "Letter".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExcelCellFormatting {
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub font_color: Option<String>,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub number_format: Option<String>,
    #[serde(default = "default_general")]
    pub horizontal_alignment: String,
    #[serde(default = "default_bottom")]
    pub vertical_alignment: String,
    #[serde(default)]
    pub wrap_text: bool,
    #[serde(default)]
    pub border_style: Option<String>,
}
fn default_general() -> String { "General".to_string() }
fn default_bottom() -> String { "Bottom".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRange {
    pub start_cell: String,
    pub end_cell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlideLayoutOptions {
    #[serde(default = "default_blank")]
    pub layout_type: String,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub gradient_background: Option<GradientFillOptions>,
}
fn default_blank() -> String { "Blank".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    #[serde(default)]
    pub format: Option<TextFormatting>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RichParagraph {
    #[serde(default)]
    pub runs: Option<Vec<TextRun>>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default = "default_left")]
    pub alignment: String,
    #[serde(default)]
    pub spacing_before_pt: Option<f64>,
    #[serde(default)]
    pub spacing_after_pt: Option<f64>,
    #[serde(default)]
    pub line_spacing_percent: Option<f64>,
    #[serde(default)]
    pub is_bullet: bool,
    #[serde(default)]
    pub bullet_char: Option<String>,
    #[serde(default)]
    pub indent_level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStop {
    pub position: i32,
    pub color: String,
    #[serde(default)]
    pub transparency_percent: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientFillOptions {
    pub stops: Vec<GradientStop>,
    #[serde(default)]
    pub angle: f64,
    #[serde(default = "default_linear")]
    pub gradient_type: String,
}
fn default_linear() -> String { "Linear".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LineOptions {
    pub x1: i64,
    pub y1: i64,
    pub x2: i64,
    pub y2: i64,
    #[serde(default)]
    pub line_color: Option<String>,
    #[serde(default = "default_line_width")]
    pub line_width: f64,
    #[serde(default = "default_solid")]
    pub dash_style: String,
    #[serde(default)]
    pub start_arrow: Option<String>,
    #[serde(default)]
    pub end_arrow: Option<String>,
}
fn default_line_width() -> f64 { 1.0 }
fn default_solid() -> String { "Solid".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectorOptions {
    pub x1: i64,
    pub y1: i64,
    pub x2: i64,
    pub y2: i64,
    #[serde(default = "default_straight")]
    pub connector_type: String,
    #[serde(default)]
    pub line_color: Option<String>,
    #[serde(default = "default_line_width")]
    pub line_width: f64,
    #[serde(default = "default_solid")]
    pub dash_style: String,
    #[serde(default)]
    pub start_arrow: Option<String>,
    #[serde(default)]
    pub end_arrow: Option<String>,
}
fn default_straight() -> String { "Straight".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupShapeItem {
    #[serde(rename = "itemType")]
    pub item_type: String,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub text_format: Option<TextFormatting>,
    #[serde(default)]
    pub shape_type: Option<String>,
    #[serde(default)]
    pub fill_color: Option<String>,
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default = "default_line_width")]
    pub border_width: f64,
    #[serde(default)]
    pub image_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBoxOptions {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default)]
    pub text_format: Option<TextFormatting>,
    #[serde(default = "default_line_width")]
    pub border_width: f64,
    #[serde(default = "default_top")]
    pub vertical_alignment: String,
    #[serde(default = "default_margin_lr")]
    pub margin_left_inches: f64,
    #[serde(default = "default_margin_lr")]
    pub margin_right_inches: f64,
    #[serde(default = "default_margin_tb")]
    pub margin_top_inches: f64,
    #[serde(default = "default_margin_tb")]
    pub margin_bottom_inches: f64,
    #[serde(default = "default_true")]
    pub word_wrap: bool,
    #[serde(default = "default_none")]
    pub auto_fit: String,
    #[serde(default)]
    pub rotation: f64,
    #[serde(default = "default_left")]
    pub alignment: String,
    #[serde(default)]
    pub paragraphs: Option<Vec<RichParagraph>>,
    #[serde(default)]
    pub gradient_fill: Option<GradientFillOptions>,
}
fn default_top() -> String { "Top".to_string() }
fn default_margin_lr() -> f64 { 0.1 }
fn default_margin_tb() -> f64 { 0.05 }
fn default_none() -> String { "None".to_string() }
impl Default for TextBoxOptions {
    fn default() -> Self {
        Self {
            x: 0, y: 0, width: 914400, height: 914400,
            background_color: None, border_color: None, text_format: None,
            border_width: 1.0, vertical_alignment: "Top".to_string(),
            margin_left_inches: 0.1, margin_right_inches: 0.1,
            margin_top_inches: 0.05, margin_bottom_inches: 0.05,
            word_wrap: true, auto_fit: "None".to_string(),
            rotation: 0.0, alignment: "Left".to_string(),
            paragraphs: None, gradient_fill: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShapeOptions {
    pub shape_type: String,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    #[serde(default)]
    pub fill_color: Option<String>,
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default = "default_line_width")]
    pub border_width: f64,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub text_format: Option<TextFormatting>,
    #[serde(default = "default_center")]
    pub text_alignment: String,
    #[serde(default = "default_middle")]
    pub vertical_text_alignment: String,
    #[serde(default)]
    pub rotation: f64,
    #[serde(default)]
    pub corner_radius_pt: Option<i32>,
    #[serde(default)]
    pub has_shadow: bool,
    #[serde(default = "default_solid")]
    pub dash_style: String,
    #[serde(default)]
    pub transparency_percent: i32,
    #[serde(default)]
    pub gradient_fill: Option<GradientFillOptions>,
    #[serde(default)]
    pub paragraphs: Option<Vec<RichParagraph>>,
    #[serde(default = "default_margin_lr")]
    pub margin_left_inches: f64,
    #[serde(default = "default_margin_lr")]
    pub margin_right_inches: f64,
    #[serde(default = "default_margin_tb")]
    pub margin_top_inches: f64,
    #[serde(default = "default_margin_tb")]
    pub margin_bottom_inches: f64,
    #[serde(default)]
    pub no_fill: bool,
    #[serde(default)]
    pub perspective_3d_angle_y: f64,
    #[serde(default)]
    pub perspective_3d_angle_x: f64,
}
fn default_middle() -> String { "Middle".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResult {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "FilePath", skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(rename = "Format", skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(rename = "Suggestion", skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}
impl DocumentResult {
    pub fn ok(msg: impl Into<String>, path: Option<String>, fmt: Option<String>) -> Self {
        Self { success: true, message: msg.into(), file_path: path, format: fmt, suggestion: None }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, message: msg.into(), file_path: None, format: None, suggestion: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResult {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "Content", skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "ErrorMessage", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(rename = "TotalParagraphs", skip_serializing_if = "Option::is_none")]
    pub total_paragraphs: Option<i32>,
    #[serde(rename = "TotalPages", skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<i32>,
    #[serde(rename = "Format", skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(rename = "Suggestion", skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContentItem {
    #[serde(rename = "Type")]
    pub r#type: String,
    #[serde(rename = "Text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "Level", skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    #[serde(rename = "Style", skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(rename = "MimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(rename = "ImageBase64", skip_serializing_if = "Option::is_none")]
    pub image_base64: Option<String>,
    #[serde(rename = "AltText", skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    #[serde(rename = "WidthPx", skip_serializing_if = "Option::is_none")]
    pub width_px: Option<i32>,
    #[serde(rename = "HeightPx", skip_serializing_if = "Option::is_none")]
    pub height_px: Option<i32>,
}

// Batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordOperation {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub level: Option<i32>,
    #[serde(default)]
    pub items: Option<Vec<String>>,
    #[serde(default, alias = "tableData")]
    pub table_data: Option<Vec<Vec<String>>>,
    #[serde(default, alias = "imagePath")]
    pub image_path: Option<String>,
    #[serde(default, alias = "widthInches")]
    pub width_inches: Option<f64>,
    #[serde(default, alias = "heightInches")]
    pub height_inches: Option<f64>,
    #[serde(default, alias = "altText")]
    pub alt_text: Option<String>,
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub italic: Option<bool>,
    #[serde(default)]
    pub underline: Option<bool>,
    #[serde(default, alias = "fontName")]
    pub font_name: Option<String>,
    #[serde(default, alias = "fontSize")]
    pub font_size: Option<i32>,
    #[serde(default, alias = "fontColor")]
    pub font_color: Option<String>,
    #[serde(default)]
    pub alignment: Option<String>,
    #[serde(default, alias = "lineSpacing")]
    pub line_spacing: Option<f64>,
    #[serde(default, alias = "hasHeader")]
    pub has_header: Option<bool>,
    #[serde(default, alias = "headerBackgroundColor")]
    pub header_background_color: Option<String>,
    #[serde(default, alias = "alternateRowColor")]
    pub alternate_row_color: Option<String>,
    #[serde(default, alias = "borderColor")]
    pub border_color: Option<String>,
    #[serde(default, alias = "borderWidth")]
    pub border_width: Option<f64>,
    #[serde(default, alias = "leftContent")]
    pub left_content: Option<String>,
    #[serde(default, alias = "centerContent")]
    pub center_content: Option<String>,
    #[serde(default, alias = "rightContent")]
    pub right_content: Option<String>,
    #[serde(default, alias = "includePageNumber")]
    pub include_page_number: Option<bool>,
    #[serde(default, alias = "includeDate")]
    pub include_date: Option<bool>,
    #[serde(default)]
    pub markdown: Option<String>,
    #[serde(default, alias = "baseImagePath")]
    pub base_image_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelOperation {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, alias = "sheetName")]
    pub sheet_name: Option<String>,
    #[serde(default, alias = "cellReference")]
    pub cell_reference: Option<String>,
    #[serde(default, alias = "startCell")]
    pub start_cell: Option<String>,
    #[serde(default, alias = "endCell")]
    pub end_cell: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub values: Option<Vec<Vec<String>>>,
    #[serde(default, alias = "tableData")]
    pub table_data: Option<Vec<Vec<String>>>,
    #[serde(default, alias = "hasHeaders")]
    pub has_headers: Option<bool>,
    #[serde(default)]
    pub formula: Option<String>,
    #[serde(default, alias = "imagePath")]
    pub image_path: Option<String>,
    #[serde(default, alias = "widthInches")]
    pub width_inches: Option<f64>,
    #[serde(default, alias = "heightInches")]
    pub height_inches: Option<f64>,
    #[serde(default, alias = "altText")]
    pub alt_text: Option<String>,
    #[serde(default, alias = "columnIndex")]
    pub column_index: Option<i32>,
    #[serde(default, alias = "rowIndex")]
    pub row_index: Option<i32>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub italic: Option<bool>,
    #[serde(default, alias = "wrapText")]
    pub wrap_text: Option<bool>,
    #[serde(default, alias = "newSheetName")]
    pub new_sheet_name: Option<String>,
    #[serde(default, alias = "fontColor")]
    pub font_color: Option<String>,
    #[serde(default, alias = "fillColor")]
    pub fill_color: Option<String>,
    #[serde(default, alias = "numberFormat")]
    pub number_format: Option<String>,
    #[serde(default, alias = "horizontalAlignment")]
    pub horizontal_alignment: Option<String>,
    #[serde(default, alias = "verticalAlignment")]
    pub vertical_alignment: Option<String>,
    #[serde(default, alias = "borderStyle")]
    pub border_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerPointOperation {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, alias = "slideIndex")]
    pub slide_index: Option<i32>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub points: Option<Vec<String>>,
    #[serde(default, alias = "tableData")]
    pub table_data: Option<Vec<Vec<String>>>,
    #[serde(default, alias = "imagePath")]
    pub image_path: Option<String>,
    #[serde(default, alias = "shapeType")]
    pub shape_type: Option<String>,
    #[serde(default, alias = "xInches")]
    pub x_inches: Option<f64>,
    #[serde(default, alias = "yInches")]
    pub y_inches: Option<f64>,
    #[serde(default, alias = "widthInches")]
    pub width_inches: Option<f64>,
    #[serde(default, alias = "heightInches")]
    pub height_inches: Option<f64>,
    #[serde(default, alias = "backgroundColor")]
    pub background_color: Option<String>,
    #[serde(default, alias = "fillColor")]
    pub fill_color: Option<String>,
    #[serde(default, alias = "borderColor")]
    pub border_color: Option<String>,
    #[serde(default, alias = "borderWidth")]
    pub border_width: Option<f64>,
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub italic: Option<bool>,
    #[serde(default)]
    pub underline: Option<bool>,
    #[serde(default, alias = "fontSize")]
    pub font_size: Option<i32>,
    #[serde(default, alias = "fontColor")]
    pub font_color: Option<String>,
    #[serde(default, alias = "fontName")]
    pub font_name: Option<String>,
    #[serde(default, alias = "altText")]
    pub alt_text: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default, alias = "sourceIndex")]
    pub source_index: Option<i32>,
    #[serde(default, alias = "fromIndex")]
    pub from_index: Option<i32>,
    #[serde(default, alias = "toIndex")]
    pub to_index: Option<i32>,
    #[serde(default)]
    pub alignment: Option<String>,
    #[serde(default, alias = "verticalAlignment")]
    pub vertical_alignment: Option<String>,
    #[serde(default, alias = "paragraphsJson")]
    pub paragraphs_json: Option<String>,
    #[serde(default)]
    pub rotation: Option<f64>,
    #[serde(default, alias = "cornerRadiusPt")]
    pub corner_radius_pt: Option<i32>,
    #[serde(default, alias = "hasShadow")]
    pub has_shadow: Option<bool>,
    #[serde(default, alias = "dashStyle")]
    pub dash_style: Option<String>,
    #[serde(default, alias = "transparencyPercent")]
    pub transparency_percent: Option<i32>,
    #[serde(default, alias = "gradientJson")]
    pub gradient_json: Option<String>,
    #[serde(default, alias = "noFill")]
    pub no_fill: Option<bool>,
    #[serde(default, alias = "wordWrap")]
    pub word_wrap: Option<bool>,
    #[serde(default, alias = "autoFit")]
    pub auto_fit: Option<String>,
    #[serde(default, alias = "x2Inches")]
    pub x2_inches: Option<f64>,
    #[serde(default, alias = "y2Inches")]
    pub y2_inches: Option<f64>,
    #[serde(default, alias = "startArrow")]
    pub start_arrow: Option<String>,
    #[serde(default, alias = "endArrow")]
    pub end_arrow: Option<String>,
    #[serde(default, alias = "connectorType")]
    pub connector_type: Option<String>,
    #[serde(default, alias = "lineColor")]
    pub line_color: Option<String>,
    #[serde(default, alias = "lineWidth")]
    pub line_width: Option<f64>,
    #[serde(default, alias = "groupItemsJson")]
    pub group_items_json: Option<String>,
    #[serde(default, alias = "marginLeftInches")]
    pub margin_left_inches: Option<f64>,
    #[serde(default, alias = "marginRightInches")]
    pub margin_right_inches: Option<f64>,
    #[serde(default, alias = "marginTopInches")]
    pub margin_top_inches: Option<f64>,
    #[serde(default, alias = "marginBottomInches")]
    pub margin_bottom_inches: Option<f64>,
    #[serde(default, alias = "slideSize")]
    pub slide_size: Option<String>,
    #[serde(default, alias = "cropShape")]
    pub crop_shape: Option<String>,
    #[serde(default, alias = "perspective3DAngleY")]
    pub perspective_3d_angle_y: Option<f64>,
    #[serde(default, alias = "perspective3DAngleX")]
    pub perspective_3d_angle_x: Option<f64>,
    #[serde(default, alias = "imageBase64")]
    pub image_base64: Option<String>,
    #[serde(default, alias = "imageMimeType")]
    pub image_mime_type: Option<String>,
    #[serde(default, alias = "zOrderPosition")]
    pub z_order_position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResult {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "TotalOperations")]
    pub total_operations: i32,
    #[serde(rename = "SuccessfulOperations")]
    pub successful_operations: i32,
    #[serde(rename = "FailedOperations")]
    pub failed_operations: i32,
    #[serde(rename = "Details")]
    pub details: Vec<OperationOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOutcome {
    #[serde(rename = "Index")]
    pub index: i32,
    #[serde(rename = "OperationType")]
    pub operation_type: String,
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "Message")]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelRangeFormattingResult {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "ErrorMessage", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(rename = "Cells", skip_serializing_if = "Option::is_none")]
    pub cells: Option<Vec<ExcelCellInfo>>,
    #[serde(rename = "SheetName", skip_serializing_if = "Option::is_none")]
    pub sheet_name: Option<String>,
    #[serde(rename = "Range", skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelCellInfo {
    #[serde(rename = "CellReference")]
    pub cell_reference: String,
    #[serde(rename = "Value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "Formula", skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    #[serde(rename = "Formatting", skip_serializing_if = "Option::is_none")]
    pub formatting: Option<ExcelCellFormattingInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelCellFormattingInfo {
    #[serde(rename = "Bold")]
    pub bold: bool,
    #[serde(rename = "Italic")]
    pub italic: bool,
    #[serde(rename = "Underline")]
    pub underline: bool,
    #[serde(rename = "FontName", skip_serializing_if = "Option::is_none")]
    pub font_name: Option<String>,
    #[serde(rename = "FontSize", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(rename = "FontColor", skip_serializing_if = "Option::is_none")]
    pub font_color: Option<String>,
    #[serde(rename = "BackgroundColor", skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(rename = "NumberFormat", skip_serializing_if = "Option::is_none")]
    pub number_format: Option<String>,
    #[serde(rename = "HorizontalAlignment")]
    pub horizontal_alignment: String,
    #[serde(rename = "VerticalAlignment")]
    pub vertical_alignment: String,
    #[serde(rename = "WrapText")]
    pub wrap_text: bool,
    #[serde(rename = "HasBorder")]
    pub has_border: bool,
    #[serde(rename = "BorderStyle", skip_serializing_if = "Option::is_none")]
    pub border_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageExtractionResult {
    #[serde(rename = "Index")]
    pub index: i32,
    #[serde(rename = "MimeType")]
    pub mime_type: String,
    #[serde(rename = "ImageBase64")]
    pub image_base64: String,
    #[serde(rename = "AltText")]
    pub alt_text: String,
    #[serde(rename = "ContextBefore")]
    pub context_before: String,
    #[serde(rename = "ContextAfter")]
    pub context_after: String,
    #[serde(rename = "WidthPx", skip_serializing_if = "Option::is_none")]
    pub width_px: Option<i32>,
    #[serde(rename = "HeightPx", skip_serializing_if = "Option::is_none")]
    pub height_px: Option<i32>,
    #[serde(rename = "PageOrSlideNumber", skip_serializing_if = "Option::is_none")]
    pub page_or_slide_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkOptions {
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "default_rotation")]
    pub rotation: f64,
    #[serde(default)]
    pub font_color: Option<String>,
    #[serde(default)]
    pub font_size: Option<i32>,
}
fn default_opacity() -> f64 { 0.3 }
fn default_rotation() -> f64 { -45.0 }
impl Default for WatermarkOptions {
    fn default() -> Self { Self { opacity: 0.3, rotation: -45.0, font_color: None, font_size: None } }
}
