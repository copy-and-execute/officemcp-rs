use serde_json::{json, Value};
use crate::models::*;
use crate::services::powerpoint as svc;

pub fn handle_pptx_create(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let title = args.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    let subtitle = args.get("subtitle").and_then(|v| v.as_str()).map(|s| s.to_string());
    let res = svc::create_presentation(file_path, title.clone());
    if !res.success { return serde_json::to_value(res).unwrap(); }
    if let Some(sub) = subtitle {
        if title.is_some() {
            let _ = svc::add_text_box(file_path, 0, &sub, TextBoxOptions { x: 914400, y: 3657600, width: 7315200, height: 914400, ..Default::default() });
        }
    }
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_read(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let read_type = args.get("readType").and_then(|v| v.as_str()).unwrap_or("all");
    let idx = args.get("slideIndex").and_then(|v| v.as_i64()).map(|v| v as usize);
    let res = match read_type.to_lowercase().as_str() {
        "slide" if idx.is_some() => svc::get_slide_text(file_path, idx.unwrap()),
        "count" => svc::get_slide_count(file_path),
        _ => svc::get_all_slides_text(file_path),
    };
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_slide(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let bg = args.get("backgroundColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let opts = bg.map(|c| SlideLayoutOptions { background_color: Some(c), ..Default::default() });
    let res = svc::add_slide(file_path, opts);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_manage_slide(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let op = args.get("operation").and_then(|v| v.as_str()).unwrap_or("");
    let idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let target = args.get("targetIndex").and_then(|v| v.as_i64()).map(|v| v as usize);
    let res = match op.to_lowercase().as_str() {
        "delete" => svc::delete_slide(file_path, idx),
        "duplicate" => svc::duplicate_slide(file_path, idx),
        "reorder" if target.is_some() => svc::reorder_slide(file_path, idx, target.unwrap()),
        _ => DocumentResult { success:false, message: format!("Unknown operation: {}", op), file_path:None, format:None, suggestion: None },
    };
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_title(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let subtitle = args.get("subtitle").and_then(|v| v.as_str()).map(|s| s.to_string());
    let bold = args.get("bold").and_then(|v| v.as_bool()).unwrap_or(true);
    let font_size = args.get("fontSize").and_then(|v| v.as_i64()).map(|v| v as i32);
    let font_color = args.get("fontColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let fmt = TextFormatting { bold, font_size, font_color, ..Default::default() };
    let res = svc::add_title(file_path, slide_idx, title, subtitle, Some(fmt));
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_text(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let text = args.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());
    let paragraphs_json = args.get("paragraphsJson").and_then(|v| v.as_str()).map(|s| s.to_string());
    let additional = args.get("additionalPointsJson").and_then(|v| v.as_str()).map(|s| s.to_string());
    let x = args.get("xInches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let y = args.get("yInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let w = args.get("widthInches").and_then(|v| v.as_f64()).unwrap_or(8.0);
    let h = args.get("heightInches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let font_size = args.get("fontSize").and_then(|v| v.as_i64()).unwrap_or(18) as i32;
    let font_color = args.get("fontColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let font_name = args.get("fontName").and_then(|v| v.as_str()).map(|s| s.to_string());
    let bold = args.get("bold").and_then(|v| v.as_bool()).unwrap_or(false);
    let italic = args.get("italic").and_then(|v| v.as_bool()).unwrap_or(false);
    let alignment = args.get("alignment").and_then(|v| v.as_str()).unwrap_or("Left").to_string();
    let vertical = args.get("verticalAlignment").and_then(|v| v.as_str()).unwrap_or("Top").to_string();
    let bg = args.get("backgroundColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let border = args.get("borderColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let bw = args.get("borderWidth").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let word_wrap = args.get("wordWrap").and_then(|v| v.as_bool()).unwrap_or(true);
    let auto_fit = args.get("autoFit").and_then(|v| v.as_str()).unwrap_or("None").to_string();
    let rotation = args.get("rotation").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let gradient_json = args.get("gradientJson").and_then(|v| v.as_str()).map(|s| s.to_string());

    let text_format = TextFormatting { bold, italic, font_size: Some(font_size), font_color, font_name, ..Default::default() };
    let gradient: Option<GradientFillOptions> = gradient_json.and_then(|s| serde_json::from_str(&s).ok());
    let paragraphs: Option<Vec<RichParagraph>> = paragraphs_json.and_then(|s| serde_json::from_str(&s).ok());
    let opts = TextBoxOptions {
        x: (x*914400.0) as i64,
        y: (y*914400.0) as i64,
        width: (w*914400.0) as i64,
        height: (h*914400.0) as i64,
        background_color: bg,
        border_color: border,
        border_width: bw,
        text_format: Some(text_format),
        vertical_alignment: vertical,
        word_wrap,
        auto_fit,
        rotation,
        alignment,
        paragraphs,
        gradient_fill: gradient,
        ..Default::default()
    };
    if let Some(paras) = &opts.paragraphs {
        if !paras.is_empty() {
            let res = svc::add_rich_text_box(file_path, slide_idx, opts);
            return serde_json::to_value(res).unwrap();
        }
    }
    if let Some(add) = additional {
        if let Ok(add_points) = serde_json::from_str::<Vec<String>>(&add) {
            let mut all = vec![text.unwrap_or_default()];
            all.extend(add_points);
            let res = svc::add_bullet_points(file_path, slide_idx, all, opts);
            return serde_json::to_value(res).unwrap();
        }
    }
    let res = svc::add_text_box(file_path, slide_idx, &text.unwrap_or_default(), opts);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_image(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let image_path = args.get("imagePath").and_then(|v| v.as_str()).unwrap_or("");
    let x = args.get("xInches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let y = args.get("yInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let w = args.get("widthInches").and_then(|v| v.as_f64()).unwrap_or(4.0);
    let h = args.get("heightInches").and_then(|v| v.as_f64()).unwrap_or(3.0);
    let alt = args.get("altText").and_then(|v| v.as_str()).map(|s| s.to_string());
    let crop = args.get("cropShape").and_then(|v| v.as_str()).unwrap_or("Rectangle").to_string();
    let opts = ImageOptions { width_emu: (w*914400.0) as i64, height_emu: (h*914400.0) as i64, alt_text: alt, crop_shape: crop, ..Default::default() };
    let res = svc::add_image(file_path, slide_idx, image_path, (x*914400.0) as i64, (y*914400.0) as i64, Some(opts));
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_table(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let data_str = args.get("tableDataJson").and_then(|v| v.as_str()).unwrap_or("[]");
    let x = args.get("xInches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let y = args.get("yInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let w = args.get("widthInches").and_then(|v| v.as_f64()).unwrap_or(8.0);
    let h = args.get("heightInches").and_then(|v| v.as_f64()).unwrap_or(3.0);
    let data: Vec<Vec<String>> = serde_json::from_str(data_str).unwrap_or_default();
    let res = svc::add_table(file_path, slide_idx, data, (x*914400.0) as i64, (y*914400.0) as i64, (w*914400.0) as i64, (h*914400.0) as i64);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_shape(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let shape_type = args.get("shapeType").and_then(|v| v.as_str()).unwrap_or("rectangle").to_string();
    let x = args.get("xInches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let y = args.get("yInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let w = args.get("widthInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let h = args.get("heightInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let fill = args.get("fillColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let border = args.get("borderColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let bw = args.get("borderWidth").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let text = args.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());
    let font_size = args.get("fontSize").and_then(|v| v.as_i64()).unwrap_or(14) as i32;
    let font_color = args.get("fontColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let font_name = args.get("fontName").and_then(|v| v.as_str()).map(|s| s.to_string());
    let bold = args.get("bold").and_then(|v| v.as_bool()).unwrap_or(false);
    let italic = args.get("italic").and_then(|v| v.as_bool()).unwrap_or(false);
    let opts = ShapeOptions {
        shape_type, x: (x*914400.0) as i64, y: (y*914400.0) as i64, width: (w*914400.0) as i64, height: (h*914400.0) as i64,
        fill_color: fill, border_color: border, border_width: bw, text, text_format: Some(TextFormatting { bold, italic, font_size: Some(font_size), font_color, font_name, ..Default::default()}),
        ..Default::default()
    };
    let res = svc::add_shape(file_path, slide_idx, opts);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_line(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let x1 = args.get("x1Inches").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y1 = args.get("y1Inches").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let x2 = args.get("x2Inches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let y2 = args.get("y2Inches").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let opts = LineOptions { x1: (x1*914400.0) as i64, y1: (y1*914400.0) as i64, x2: (x2*914400.0) as i64, y2: (y2*914400.0) as i64, ..Default::default() };
    // For compat, check alternative param names
    let res = svc::add_line(file_path, slide_idx, opts);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_connector(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let x1 = args.get("x1Inches").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y1 = args.get("y1Inches").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let x2 = args.get("x2Inches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let y2 = args.get("y2Inches").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let opts = ConnectorOptions { x1: (x1*914400.0) as i64, y1: (y1*914400.0) as i64, x2: (x2*914400.0) as i64, y2: (y2*914400.0) as i64, ..Default::default() };
    let res = svc::add_connector(file_path, slide_idx, opts);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_image_base64(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let b64 = args.get("base64Data").and_then(|v| v.as_str()).unwrap_or("");
    let mime = args.get("mimeType").and_then(|v| v.as_str()).unwrap_or("png");
    let x = args.get("xInches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let y = args.get("yInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let w = args.get("widthInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let h = args.get("heightInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let res = svc::add_image_base64(file_path, slide_idx, b64, mime, (x*914400.0) as i64, (y*914400.0) as i64, None);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_z_order(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let shape_idx = args.get("shapeIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let pos = args.get("position").and_then(|v| v.as_str()).unwrap_or("front");
    let res = svc::set_shape_z_order(file_path, slide_idx, shape_idx, pos);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_reorder_shape(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let from = args.get("fromIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let to = args.get("toIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let res = svc::reorder_shape(file_path, slide_idx, from, to);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_add_group(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let x = args.get("xInches").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y = args.get("yInches").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let w = args.get("widthInches").and_then(|v| v.as_f64()).unwrap_or(5.0);
    let h = args.get("heightInches").and_then(|v| v.as_f64()).unwrap_or(5.0);
    let items_json = args.get("groupItemsJson").and_then(|v| v.as_str()).unwrap_or("[]");
    let items: Vec<GroupShapeItem> = serde_json::from_str(items_json).unwrap_or_default();
    let res = svc::add_group_shape(file_path, slide_idx, (x*914400.0) as i64, (y*914400.0) as i64, (w*914400.0) as i64, (h*914400.0) as i64, items);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_set_slide_size(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let size = args.get("size").and_then(|v| v.as_str()).unwrap_or("Widescreen");
    let res = svc::set_slide_size(file_path, size);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_set_background(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let color = args.get("color").and_then(|v| v.as_str()).map(|s| s.to_string());
    let gradient_json = args.get("gradientJson").and_then(|v| v.as_str()).map(|s| s.to_string());
    if let Some(gj) = gradient_json {
        if let Ok(g) = serde_json::from_str::<GradientFillOptions>(&gj) {
            let res = svc::set_slide_background_gradient(file_path, slide_idx, g);
            return serde_json::to_value(res).unwrap();
        }
    }
    if let Some(c) = color {
        let res = svc::set_slide_background(file_path, slide_idx, &c);
        return serde_json::to_value(res).unwrap();
    }
    json!({"Success": false, "Message": "Provide either color or gradientJson"})
}

pub fn handle_pptx_add_notes(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let slide_idx = args.get("slideIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let notes = args.get("notes").and_then(|v| v.as_str()).unwrap_or("");
    let res = svc::add_speaker_notes(file_path, slide_idx, notes);
    serde_json::to_value(res).unwrap()
}

pub fn handle_pptx_batch(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let ops_json = args.get("operationsJson").and_then(|v| v.as_str()).unwrap_or("[]");
    let ops: Result<Vec<PowerPointOperation>, _> = serde_json::from_str(ops_json);
    let ops = match ops {
        Ok(o) => o,
        Err(e) => return json!({"Success": false, "Message": format!("Invalid JSON: {}", e)}),
    };
    let mut details = Vec::new();
    let mut succ=0; let mut fail=0;
    for (i, op) in ops.iter().enumerate() {
        let r = match op.r#type.to_lowercase().as_str() {
            "addslide" => svc::add_slide(file_path, None),
            "deleteslide" => svc::delete_slide(file_path, op.slide_index.unwrap_or(0) as usize),
            "duplicateslide" => svc::duplicate_slide(file_path, op.source_index.unwrap_or(0) as usize),
            "reorderslide" => svc::reorder_slide(file_path, op.from_index.unwrap_or(0) as usize, op.to_index.unwrap_or(0) as usize),
            "setbackground" => svc::set_slide_background(file_path, op.slide_index.unwrap_or(0) as usize, &op.background_color.clone().unwrap_or("FFFFFF".to_string())),
            "setbackgroundgradient" => {
                if let Some(gj) = &op.gradient_json {
                    if let Ok(g) = serde_json::from_str::<GradientFillOptions>(gj) { svc::set_slide_background_gradient(file_path, op.slide_index.unwrap_or(0) as usize, g) } else { DocumentResult { success:false, message: "Invalid gradient".to_string(), file_path:None, format:None, suggestion:None } }
                } else { DocumentResult { success:false, message: "gradientJson required".to_string(), file_path:None, format:None, suggestion:None } }
            },
            "setslidesize" => svc::set_slide_size(file_path, &op.slide_size.clone().unwrap_or("Widescreen".to_string())),
            "addtitle" => svc::add_title(file_path, op.slide_index.unwrap_or(0) as usize, &op.title.clone().unwrap_or_default(), op.subtitle.clone(), None),
            "addtextbox" => {
                let opts = TextBoxOptions { x: ((op.x_inches.unwrap_or(1.0))*914400.0) as i64, y: ((op.y_inches.unwrap_or(2.0))*914400.0) as i64, width: ((op.width_inches.unwrap_or(8.0))*914400.0) as i64, height: ((op.height_inches.unwrap_or(1.0))*914400.0) as i64, ..Default::default() };
                svc::add_text_box(file_path, op.slide_index.unwrap_or(0) as usize, &op.text.clone().unwrap_or_default(), opts)
            },
            "addrichtextbox" => {
                let opts = TextBoxOptions { x: ((op.x_inches.unwrap_or(1.0))*914400.0) as i64, y: ((op.y_inches.unwrap_or(2.0))*914400.0) as i64, width: ((op.width_inches.unwrap_or(8.0))*914400.0) as i64, height: ((op.height_inches.unwrap_or(1.0))*914400.0) as i64, ..Default::default() };
                svc::add_rich_text_box(file_path, op.slide_index.unwrap_or(0) as usize, opts)
            },
            "addbulletpoints" => {
                let opts = TextBoxOptions { x: ((op.x_inches.unwrap_or(1.0))*914400.0) as i64, y: ((op.y_inches.unwrap_or(2.0))*914400.0) as i64, width: ((op.width_inches.unwrap_or(8.0))*914400.0) as i64, height: ((op.height_inches.unwrap_or(4.0))*914400.0) as i64, ..Default::default() };
                svc::add_bullet_points(file_path, op.slide_index.unwrap_or(0) as usize, op.points.clone().unwrap_or_default(), opts)
            },
            "addimage" => svc::add_image(file_path, op.slide_index.unwrap_or(0) as usize, &op.image_path.clone().unwrap_or_default(), ((op.x_inches.unwrap_or(1.0))*914400.0) as i64, ((op.y_inches.unwrap_or(2.0))*914400.0) as i64, None),
            "addimagebase64" => svc::add_image_base64(file_path, op.slide_index.unwrap_or(0) as usize, &op.image_base64.clone().unwrap_or_default(), &op.image_mime_type.clone().unwrap_or("png".to_string()), ((op.x_inches.unwrap_or(1.0))*914400.0) as i64, ((op.y_inches.unwrap_or(2.0))*914400.0) as i64, None),
            "addshape" => {
                let opts = ShapeOptions { shape_type: op.shape_type.clone().unwrap_or("rectangle".to_string()), x: ((op.x_inches.unwrap_or(1.0))*914400.0) as i64, y: ((op.y_inches.unwrap_or(2.0))*914400.0) as i64, width: ((op.width_inches.unwrap_or(2.0))*914400.0) as i64, height: ((op.height_inches.unwrap_or(2.0))*914400.0) as i64, ..Default::default() };
                svc::add_shape(file_path, op.slide_index.unwrap_or(0) as usize, opts)
            },
            "addline" => svc::add_line(file_path, op.slide_index.unwrap_or(0) as usize, LineOptions { x1: ((op.x_inches.unwrap_or(0.0))*914400.0) as i64, y1: ((op.y_inches.unwrap_or(0.0))*914400.0) as i64, x2: ((op.x2_inches.unwrap_or(1.0))*914400.0) as i64, y2: ((op.y2_inches.unwrap_or(0.0))*914400.0) as i64, ..Default::default() }),
            "addconnector" => svc::add_connector(file_path, op.slide_index.unwrap_or(0) as usize, ConnectorOptions { x1: ((op.x_inches.unwrap_or(0.0))*914400.0) as i64, y1: ((op.y_inches.unwrap_or(0.0))*914400.0) as i64, x2: ((op.x2_inches.unwrap_or(1.0))*914400.0) as i64, y2: ((op.y2_inches.unwrap_or(0.0))*914400.0) as i64, ..Default::default() }),
            "addgroupshape" => {
                let items: Vec<GroupShapeItem> = op.group_items_json.as_ref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default();
                svc::add_group_shape(file_path, op.slide_index.unwrap_or(0) as usize, ((op.x_inches.unwrap_or(0.0))*914400.0) as i64, ((op.y_inches.unwrap_or(0.0))*914400.0) as i64, ((op.width_inches.unwrap_or(5.0))*914400.0) as i64, ((op.height_inches.unwrap_or(5.0))*914400.0) as i64, items)
            },
            "addtable" => {
                let data = op.table_data.clone().unwrap_or_default();
                svc::add_table(file_path, op.slide_index.unwrap_or(0) as usize, data, 914400, 1828800, 7315200, 2743200)
            },
            "addspeakernotes" => svc::add_speaker_notes(file_path, op.slide_index.unwrap_or(0) as usize, &op.notes.clone().unwrap_or_default()),
            "setzorder" => svc::set_shape_z_order(file_path, op.slide_index.unwrap_or(0) as usize, op.from_index.unwrap_or(0) as usize, &op.dash_style.clone().unwrap_or("front".to_string())),
            "reordershape" => svc::reorder_shape(file_path, op.slide_index.unwrap_or(0) as usize, op.from_index.unwrap_or(0) as usize, op.to_index.unwrap_or(0) as usize),
            _ => DocumentResult { success:false, message: format!("Unknown type: {}", op.r#type), file_path:None, format:None, suggestion:None },
        };
        if r.success { succ+=1; details.push(OperationOutcome { index: i as i32, operation_type: op.r#type.clone(), success:true, message:"Success".to_string()}); } else { fail+=1; details.push(OperationOutcome { index: i as i32, operation_type: op.r#type.clone(), success:false, message:r.message.clone()}); }
    }
    let res = BatchOperationResult { success: fail==0, message: if fail==0 {format!("All {} operations completed", succ)} else {format!("{} succeeded, {} failed", succ, fail)}, total_operations: ops.len() as i32, successful_operations: succ, failed_operations: fail, details };
    serde_json::to_value(res).unwrap()
}
