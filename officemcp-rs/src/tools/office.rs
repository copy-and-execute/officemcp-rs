use serde_json::{json, Value};
use crate::models::*;
use crate::services::{format_detector, word, excel, powerpoint, pdf, protection};
use std::path::Path;

fn err(msg: String) -> Value { json!({"Success": false, "Message": msg}) }
fn ok(msg: String, path: Option<String>, fmt: Option<String>) -> Value {
    json!({"Success": true, "Message": msg, "FilePath": path, "Format": fmt})
}

pub fn handle_office_create(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    if file_path.is_empty() { return err("filePath is required".to_string()); }
    let format = match format_detector::detect_format(file_path) {
        Ok(f) => f,
        Err(e) => return err(e),
    };
    let title = args.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    let markdown = args.get("markdown").and_then(|v| v.as_str()).map(|s| s.to_string());
    let base = args.get("baseImagePath").and_then(|v| v.as_str()).map(|s| s.to_string());
    let orientation = args.get("orientation").and_then(|v| v.as_str()).unwrap_or("Portrait").to_string();
    let page_size = args.get("pageSize").and_then(|v| v.as_str()).unwrap_or("Letter").to_string();
    let template = args.get("templatePath").and_then(|v| v.as_str()).map(|s| s.to_string());
    match format.as_str() {
        "xlsx" => {
            let res = excel::create_workbook(file_path, title);
            if !res.success { return serde_json::to_value(res).unwrap(); }
            if let Some(md) = markdown {
                if !md.trim().is_empty() {
                    // try parse as JSON 2D array for excel initial data
                    if let Ok(data) = serde_json::from_str::<Vec<Vec<String>>>(&md) {
                        let sheet = args.get("title").and_then(|v| v.as_str()).unwrap_or("Sheet1").to_string();
                        let r = excel::add_table(file_path, &sheet, "A1", data, true);
                        if !r.success { return err(format!("Created but data failed: {}", r.message)); }
                    }
                }
            }
            serde_json::to_value(res).unwrap()
        },
        "pptx" => {
            let res = powerpoint::create_presentation(file_path, title);
            serde_json::to_value(res).unwrap()
        },
        _ => {
            let layout = PageLayoutOptions { orientation, page_size, ..Default::default() };
            let res = match format.as_str() {
                "docx" => word::create_document(file_path, title, Some(layout), template),
                "pdf" => pdf::create_document(file_path, title, Some(layout), template),
                _ => word::create_document(file_path, title, Some(layout), template),
            };
            if !res.success { return serde_json::to_value(res).unwrap(); }
            if let Some(md) = markdown {
                if !md.trim().is_empty() {
                    let r = match format.as_str() {
                        "pdf" => pdf::add_markdown_content(file_path, &md, base),
                        _ => word::add_markdown_content(file_path, &md, base),
                    };
                    if !r.success { return err(format!("Created but content failed: {}", r.message)); }
                }
            }
            ok(format!("Document created"), Some(file_path.to_string()), Some(format))
        }
    }
}

pub fn handle_office_read(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    if !Path::new(file_path).exists() {
        return err(format!("File not found: {}", file_path));
    }
    let format = match format_detector::detect_format(file_path) {
        Ok(f) => f,
        Err(e) => return err(e),
    };
    let read_type = args.get("readType").and_then(|v| v.as_str()).unwrap_or("all");
    let element_id = args.get("elementId").and_then(|v| v.as_str()).map(|s| s.to_string());
    let start_ref = args.get("startRef").and_then(|v| v.as_str()).map(|s| s.to_string());
    let end_ref = args.get("endRef").and_then(|v| v.as_str()).map(|s| s.to_string());
    let include_images = args.get("includeImages").and_then(|v| v.as_bool()).unwrap_or(true);
    match format.as_str() {
        "xlsx" => {
            let res = match read_type.to_lowercase().as_str() {
                "sheet" if element_id.is_some() => excel::get_sheet_text(file_path, &element_id.unwrap()),
                "cell" if element_id.is_some() && start_ref.is_some() => excel::get_cell_value(file_path, &element_id.unwrap(), &start_ref.unwrap()),
                "range" if element_id.is_some() && start_ref.is_some() && end_ref.is_some() => excel::get_range_values(file_path, &element_id.unwrap(), &start_ref.unwrap(), &end_ref.unwrap()),
                _ => excel::get_all_sheets_text(file_path),
            };
            serde_json::to_value(res).unwrap()
        },
        "pptx" => {
            let res = match read_type.to_lowercase().as_str() {
                "slide" | "element" if element_id.is_some() => {
                    if let Ok(idx) = element_id.unwrap().parse::<usize>() { powerpoint::get_slide_text(file_path, idx) } else { powerpoint::get_all_slides_text(file_path) }
                },
                "count" => powerpoint::get_slide_count(file_path),
                _ => powerpoint::get_all_slides_text(file_path),
            };
            serde_json::to_value(res).unwrap()
        },
        "docx" if include_images => {
            let items = word::get_rich_content(file_path);
            let resp = json!({
                "Success": true,
                "Format": "docx",
                "FilePath": file_path,
                "TemplatePath": file_path,
                "ItemCount": items.len(),
                "Note": "Content is in document reading order. Images appear immediately after their containing paragraph. Preceding headings and paragraphs provide section context. Use MimeType + ImageBase64 for AI vision analysis, OCR, and caption generation. IMPORTANT: When recreating or rewriting this document, pass the TemplatePath value to the templatePath parameter of office_create to preserve the original styles, fonts, and formatting.",
                "Content": items
            });
            resp
        },
        _ => {
            let res = match read_type.to_lowercase().as_str() {
                "element" if element_id.is_some() => {
                    if let Ok(idx) = element_id.unwrap().parse::<usize>() {
                        match format.as_str() {
                            "pdf" => pdf::get_paragraph_text(file_path, idx),
                            _ => word::get_paragraph_text(file_path, idx),
                        }
                    } else { word::get_document_text(file_path) }
                },
                "range" if start_ref.is_some() && end_ref.is_some() => {
                    if let (Ok(s), Ok(e)) = (start_ref.unwrap().parse::<usize>(), end_ref.unwrap().parse::<usize>()) {
                        match format.as_str() {
                            "pdf" => pdf::get_paragraph_range(file_path, s, e),
                            _ => word::get_paragraph_range(file_path, s, e),
                        }
                    } else { word::get_document_text(file_path) }
                },
                _ => match format.as_str() {
                    "pdf" => pdf::get_document_text(file_path),
                    _ => word::get_document_text(file_path),
                },
            };
            serde_json::to_value(res).unwrap()
        }
    }
}

pub fn handle_office_write(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let target_id = args.get("targetId").and_then(|v| v.as_str()).map(|s| s.to_string());
    let position = args.get("position").and_then(|v| v.as_str()).map(|s| s.to_string());
    let base = args.get("baseImagePath").and_then(|v| v.as_str()).map(|s| s.to_string());
    let y_inches = args.get("yInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let width_inches = args.get("widthInches").and_then(|v| v.as_f64()).unwrap_or(8.0);
    let height_inches = args.get("heightInches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let font_size = args.get("fontSize").and_then(|v| v.as_i64()).map(|v| v as i32);
    let font_color = args.get("fontColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let bold = args.get("bold").and_then(|v| v.as_bool()).unwrap_or(false);
    let font_name = args.get("fontName").and_then(|v| v.as_str()).map(|s| s.to_string());

    if !Path::new(file_path).exists() {
        return err(format!("File not found: {}", file_path));
    }
    let format = match format_detector::detect_format(file_path) {
        Ok(f) => f,
        Err(e) => return err(e),
    };
    match format.as_str() {
        "xlsx" => {
            let sheet = target_id.unwrap_or("Sheet1".to_string());
            let cell = position.unwrap_or("A1".to_string());
            let res = excel::set_cell_value(file_path, &sheet, &cell, content, None);
            serde_json::to_value(res).unwrap()
        },
        "pptx" => {
            let idx = target_id.and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let x = position.and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
            let fmt = if font_size.is_some() || font_color.is_some() || bold || font_name.is_some() {
                Some(TextFormatting { bold, font_size, font_color, font_name, ..Default::default() })
            } else { None };
            let opts = crate::models::TextBoxOptions { x: (x*914400.0) as i64, y: (y_inches*914400.0) as i64, width: (width_inches*914400.0) as i64, height: (height_inches*914400.0) as i64, text_format: fmt, ..Default::default() };
            let res = powerpoint::add_text_box(file_path, idx, content, opts);
            serde_json::to_value(res).unwrap()
        },
        _ => {
            let res = match format.as_str() {
                "pdf" => pdf::add_markdown_content(file_path, content, base),
                _ => word::add_markdown_content(file_path, content, base),
            };
            serde_json::to_value(res).unwrap()
        }
    }
}

pub fn handle_office_convert(args: &Value) -> Value {
    let source = args.get("sourcePath").and_then(|v| v.as_str()).unwrap_or("");
    let target = args.get("targetFormat").and_then(|v| v.as_str()).unwrap_or("");
    let output = args.get("outputPath").and_then(|v| v.as_str()).map(|s| s.to_string());
    if !Path::new(source).exists() {
        return err(format!("Source file not found: {}", source));
    }
    if target.to_lowercase()=="md" {
        let fmt = match format_detector::detect_format(source) { Ok(f)=>f, Err(e)=> return err(e) };
        if format_detector::uses_unified_interface(&fmt) {
            let res = match fmt.as_str() {
                "pdf" => pdf::convert_to_markdown(source),
                _ => word::convert_to_markdown(source),
            };
            if !res.success { return serde_json::to_value(res).unwrap(); }
            let out = output.unwrap_or_else(|| format!("{}.md", source.trim_end_matches(&format!(".{}", fmt))));
            if let Some(txt) = res.content {
                let _ = std::fs::write(&out, txt);
                return ok(format!("Converted to markdown"), Some(out), Some(fmt));
            }
            return err("Conversion produced no content".to_string());
        }
    }
    err(format!("Conversion from {} to {} not yet implemented", source, target))
}

pub fn handle_office_metadata(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let include_structure = args.get("includeStructure").and_then(|v| v.as_bool()).unwrap_or(false);
    if !Path::new(file_path).exists() {
        return err(format!("File not found: {}", file_path));
    }
    let format = match format_detector::detect_format(file_path) {
        Ok(f) => f,
        Err(e) => return err(e),
    };
    let meta = std::fs::metadata(file_path).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let created = meta.as_ref().and_then(|m| m.created().ok()).map(|t| format!("{:?}", t)).unwrap_or("".to_string());
    let modified = meta.as_ref().and_then(|m| m.modified().ok()).map(|t| format!("{:?}", t)).unwrap_or("".to_string());
    let mut map = json!({
        "Success": true,
        "Format": format,
        "FilePath": file_path,
        "FileSize": size,
        "FileSizeFormatted": format_file_size(size),
        "CreatedDate": created,
        "ModifiedDate": modified
    });
    if include_structure {
        match format.as_str() {
            "xlsx" => {
                let r = excel::get_all_sheets_text(file_path);
                map["SheetInfo"] = json!(r.content.unwrap_or("Unable to read sheets".to_string()));
            },
            "pptx" => {
                let r = powerpoint::get_slide_count(file_path);
                map["SlideCount"] = json!(r.content.unwrap_or("0".to_string()));
            },
            "pdf" => {
                let r = pdf::get_document_text(file_path);
                map["PageCount"] = json!(r.total_pages.unwrap_or(0));
            },
            _ => {
                let r = word::get_document_text(file_path);
                map["ParagraphCount"] = json!(r.total_paragraphs.unwrap_or(0));
            }
        }
    }
    map
}

fn format_file_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < 3 { size/=1024.0; unit+=1; }
    format!("{:.2} {}", size, UNITS[unit])
}

pub fn handle_office_add_element(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let element_type = args.get("elementType").and_then(|v| v.as_str()).unwrap_or("");
    let content = args.get("content").and_then(|v| v.as_str()).map(|s| s.to_string());
    let image_path = args.get("imagePath").and_then(|v| v.as_str()).map(|s| s.to_string());
    let level = args.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    let width = args.get("widthInches").and_then(|v| v.as_f64()).unwrap_or(4.0);
    let height = args.get("heightInches").and_then(|v| v.as_f64()).unwrap_or(3.0);
    let sheet_name = args.get("sheetName").and_then(|v| v.as_str()).map(|s| s.to_string());
    let start_cell = args.get("startCell").and_then(|v| v.as_str()).map(|s| s.to_string());
    let x_inches = args.get("xInches").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let y_inches = args.get("yInches").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let font_size = args.get("fontSize").and_then(|v| v.as_i64()).map(|v| v as i32);
    let font_color = args.get("fontColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let fill_color = args.get("fillColor").and_then(|v| v.as_str()).map(|s| s.to_string());
    let bold = args.get("bold").and_then(|v| v.as_bool()).unwrap_or(false);
    let font_name = args.get("fontName").and_then(|v| v.as_str()).map(|s| s.to_string());
    let shape_type = args.get("shapeType").and_then(|v| v.as_str()).map(|s| s.to_string());

    if !Path::new(file_path).exists() { return err(format!("File not found: {}", file_path)); }
    let format = match format_detector::detect_format(file_path) { Ok(f)=>f, Err(e)=> return err(e) };

    if element_type.eq_ignore_ascii_case("table") && content.is_some() {
        if let Some(ref c) = content {
            if let Ok(data) = serde_json::from_str::<Vec<Vec<String>>>(c) {
                return match format.as_str() {
                    "xlsx" => {
                        let res = excel::add_table(file_path, &sheet_name.unwrap_or("Sheet1".to_string()), &start_cell.unwrap_or("A1".to_string()), data, true);
                        serde_json::to_value(res).unwrap()
                    },
                    "pptx" => {
                        let idx = level as usize;
                        let res = powerpoint::add_table(file_path, idx, data, (x_inches*914400.0) as i64, (y_inches*914400.0) as i64, (width*914400.0) as i64, (height*914400.0) as i64);
                        serde_json::to_value(res).unwrap()
                    },
                    _ => {
                        let res = word::add_table(file_path, data, None);
                        serde_json::to_value(res).unwrap()
                    }
                };
            }
        }
    }

    match format.as_str() {
        "xlsx" => {
            match element_type.to_lowercase().as_str() {
                "image" if image_path.is_some() => {
                    let ip = image_path.clone().unwrap();
                    let res = excel::add_image(file_path, &sheet_name.clone().unwrap_or("Sheet1".to_string()), &ip, &start_cell.clone().unwrap_or("A1".to_string()), None);
                    serde_json::to_value(res).unwrap()
                },
                _ => err(format!("Element type '{}' not supported for Excel", element_type)),
            }
        },
        "pptx" => {
            let fmt = if font_size.is_some() || font_color.is_some() || bold || font_name.is_some() {
                Some(TextFormatting { bold, font_size: font_size.clone(), font_color: font_color.clone(), font_name: font_name.clone(), ..Default::default() })
            } else { None };
            match element_type.to_lowercase().as_str() {
                "image" if image_path.is_some() => {
                    let ip = image_path.clone().unwrap();
                    let res = powerpoint::add_image(file_path, level as usize, &ip, (x_inches*914400.0) as i64, (y_inches*914400.0) as i64, None);
                    serde_json::to_value(res).unwrap()
                },
                "paragraph" | "text" if content.is_some() => {
                    let c = content.clone().unwrap();
                    let opts = TextBoxOptions { x: (x_inches*914400.0) as i64, y: (y_inches*914400.0) as i64, width: (width*914400.0) as i64, height: (height*914400.0) as i64, text_format: fmt, background_color: fill_color.clone(), ..Default::default() };
                    let res = powerpoint::add_text_box(file_path, level as usize, &c, opts);
                    serde_json::to_value(res).unwrap()
                },
                "heading" | "title" if content.is_some() => {
                    let c = content.clone().unwrap();
                    let res = powerpoint::add_title(file_path, level as usize, &c, None, fmt);
                    serde_json::to_value(res).unwrap()
                },
                "bulletlist" if content.is_some() => {
                    let c = content.clone().unwrap();
                    let points: Vec<String> = c.split('\n').map(|s| s.to_string()).collect();
                    let opts = TextBoxOptions { x: (x_inches*914400.0) as i64, y: (y_inches*914400.0) as i64, width: (width*914400.0) as i64, height: (4.0*914400.0) as i64, ..Default::default() };
                    let res = powerpoint::add_bullet_points(file_path, level as usize, points, opts);
                    serde_json::to_value(res).unwrap()
                },
                "shape" => {
                    let opts = ShapeOptions { shape_type: shape_type.clone().unwrap_or("rectangle".to_string()), x: (x_inches*914400.0) as i64, y: (y_inches*914400.0) as i64, width: (width*914400.0) as i64, height: (height*914400.0) as i64, text: content.clone(), fill_color: fill_color.clone(), ..Default::default() };
                    let res = powerpoint::add_shape(file_path, level as usize, opts);
                    serde_json::to_value(res).unwrap()
                },
                "line" => {
                    let res = powerpoint::add_line(file_path, level as usize, LineOptions { x1: (x_inches*914400.0) as i64, y1: (y_inches*914400.0) as i64, x2: ((x_inches+width)*914400.0) as i64, y2: (y_inches*914400.0) as i64, ..Default::default() });
                    serde_json::to_value(res).unwrap()
                },
                _ => err(format!("Element type '{}' requires content", element_type)),
            }
        },
        _ => {
            let res = match element_type.to_lowercase().as_str() {
                "paragraph" if content.is_some() => word::add_paragraph(file_path, &content.unwrap(), None, None),
                "heading" if content.is_some() => word::add_heading(file_path, &content.unwrap(), level, None),
                "image" if image_path.is_some() => word::add_image(file_path, &image_path.unwrap(), Some(ImageOptions { width_emu: (width*914400.0) as i64, height_emu: (height*914400.0) as i64, ..Default::default() })),
                "pagebreak" => word::add_page_break(file_path),
                "bulletlist" if content.is_some() => word::add_bullet_list(file_path, content.unwrap().split('\n').map(|s| s.to_string()).collect(), None),
                "numberedlist" if content.is_some() => word::add_numbered_list(file_path, content.unwrap().split('\n').map(|s| s.to_string()).collect(), None),
                _ => DocumentResult { success:false, message: format!("Element type '{}' requires content or valid image path", element_type), file_path:None, format:None, suggestion:None },
            };
            serde_json::to_value(res).unwrap()
        }
    }
}

pub fn handle_office_add_header_footer(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let location = args.get("location").and_then(|v| v.as_str()).unwrap_or("");
    let opts = HeaderFooterOptions {
        left_content: args.get("leftContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
        center_content: args.get("centerContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
        right_content: args.get("rightContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
        include_page_number: args.get("includePageNumber").and_then(|v| v.as_bool()).unwrap_or(false),
        include_date: args.get("includeDate").and_then(|v| v.as_bool()).unwrap_or(false),
    };
    if !Path::new(file_path).exists() { return err(format!("File not found: {}", file_path)); }
    let format = match format_detector::detect_format(file_path) { Ok(f)=>f, Err(e)=> return err(e) };
    if format!="docx" && format!="pdf" { return err(format!("Header/footer not supported for {}", format)); }
    let res = match location.to_lowercase().as_str() {
        "header" => if format=="pdf" { pdf::add_header(file_path, opts) } else { word::add_header(file_path, opts) },
        "footer" => if format=="pdf" { pdf::add_footer(file_path, opts) } else { word::add_footer(file_path, opts) },
        _ => DocumentResult { success:false, message: format!("Invalid location: {}", location), file_path:None, format:None, suggestion: Some("Use 'header' or 'footer'".to_string()) },
    };
    serde_json::to_value(res).unwrap()
}

pub fn handle_office_extract(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let extract_type = args.get("extractType").and_then(|v| v.as_str()).unwrap_or("");
    let scope = args.get("scope").and_then(|v| v.as_str()).unwrap_or("all");
    let start = args.get("startIndex").and_then(|v| v.as_i64()).map(|v| v as usize);
    let end = args.get("endIndex").and_then(|v| v.as_i64()).map(|v| v as usize);
    if !Path::new(file_path).exists() { return err(format!("File not found: {}", file_path)); }
    let format = match format_detector::detect_format(file_path) { Ok(f)=>f, Err(e)=> return err(e) };
    match extract_type.to_lowercase().as_str() {
        "text" => {
            let rt = match scope { "element" => "element", "range" => "range", _ => "all" };
            let mut m = json!({"filePath": file_path, "readType": rt});
            if let Some(s) = start { m["elementId"] = json!(s.to_string()); m["startRef"] = json!(s.to_string()); }
            if let Some(e) = end { m["endRef"] = json!(e.to_string()); }
            handle_office_read(&m)
        },
        "metadata" => handle_office_metadata(&json!({"filePath": file_path, "includeStructure": true})),
        "images" => {
            let images = match format.as_str() {
                "pptx" => powerpoint::extract_images(file_path),
                "pdf" => pdf::extract_images(file_path),
                _ => word::extract_images(file_path),
            };
            json!({
                "Success": true,
                "Format": format,
                "ImageCount": images.len(),
                "Note": if images.is_empty() { "No embedded images found in this document" } else { "Each image includes base64-encoded data (ImageBase64), alt text, and surrounding text context. Use the image data for OCR and caption generation." },
                "Images": images
            })
        },
        _ => err(format!("Unknown extract type: {}", extract_type)),
    }
}

pub fn handle_office_batch(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let ops_json = args.get("operationsJson").and_then(|v| v.as_str()).unwrap_or("[]");
    let ops: Result<Vec<Value>, _> = serde_json::from_str(ops_json);
    let ops = match ops { Ok(o)=>o, Err(e)=> return err(format!("Invalid JSON: {}", e)) };
    if !Path::new(file_path).exists() { return err(format!("File not found: {}", file_path)); }
    let format = match format_detector::detect_format(file_path) { Ok(f)=>f, Err(e)=> return err(e) };
    let mut details = Vec::new();
    let mut succ=0; let mut fail=0;
    for (i, op) in ops.iter().enumerate() {
        let op_type = op.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let content = op.get("content").and_then(|v| v.as_str()).or_else(|| op.get("text").and_then(|v| v.as_str()));
        let level = op.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        let image_path = op.get("imagePath").and_then(|v| v.as_str());
        let res_json = match op_type.to_lowercase().as_str() {
            "paragraph" | "heading" | "image" | "pagebreak" | "bulletlist" | "numberedlist" | "table" => {
                let mut m = json!({"filePath": file_path, "elementType": op_type});
                if let Some(c) = content { m["content"] = json!(c); }
                if let Some(p) = image_path { m["imagePath"] = json!(p); }
                m["level"] = json!(level);
                handle_office_add_element(&m)
            },
            "markdown" if content.is_some() => handle_office_write(&json!({"filePath": file_path, "content": content.unwrap()})),
            _ => err(format!("Unknown batch operation type: {}", op_type)),
        };
        let success = res_json.get("Success").and_then(|v| v.as_bool()).unwrap_or(false);
        if success { succ+=1; } else { fail+=1; }
        details.push(OperationOutcome { index: i as i32, operation_type: op_type.to_string(), success, message: if success {"OK".to_string()} else {"Failed".to_string()} });
    }
    let res = BatchOperationResult { success: fail==0, message: if fail==0 {"All operations completed".to_string()} else {format!("{} operation(s) failed", fail)}, total_operations: ops.len() as i32, successful_operations: succ, failed_operations: fail, details };
    serde_json::to_value(res).unwrap()
}

pub fn handle_office_merge(args: &Value) -> Value {
    let output = args.get("outputPath").and_then(|v| v.as_str()).unwrap_or("");
    let input_json = args.get("inputPathsJson").and_then(|v| v.as_str()).unwrap_or("[]");
    let inputs: Result<Vec<String>, _> = serde_json::from_str(input_json);
    let inputs = match inputs { Ok(v)=>v, Err(e)=> return err(format!("Invalid inputPathsJson: {}", e)) };
    let format = match format_detector::detect_format(output) { Ok(f)=>f, Err(e)=> return err(e) };
    if format=="pdf" {
        let res = pdf::merge_documents(output, inputs);
        serde_json::to_value(res).unwrap()
    } else {
        err(format!("Merge not supported for {}", format))
    }
}

pub fn handle_office_pdf_pages(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let operation = args.get("operation").and_then(|v| v.as_str()).unwrap_or("");
    let page_numbers = args.get("pageNumbers").and_then(|v| v.as_str()).map(|s| s.to_string());
    let output_path = args.get("outputPath").and_then(|v| v.as_str()).map(|s| s.to_string());
    let watermark_text = args.get("watermarkText").and_then(|v| v.as_str()).map(|s| s.to_string());
    let opacity = args.get("opacity").and_then(|v| v.as_f64()).unwrap_or(0.3);
    let rotation = args.get("rotation").and_then(|v| v.as_f64()).unwrap_or(-45.0);
    if !Path::new(file_path).exists() { return err(format!("File not found: {}", file_path)); }
    match operation.to_lowercase().as_str() {
        "extract_pages" if page_numbers.is_some() && output_path.is_some() => {
            let pages_str = page_numbers.unwrap();
            let pages: Result<Vec<i32>, _> = serde_json::from_str(&pages_str);
            let pages = match pages {
                Ok(p)=>p,
                Err(_) => {
                    if let Ok(n) = pages_str.parse::<i32>() { vec![n] } else { return err("Invalid pageNumbers".to_string()); }
                }
            };
            let res = pdf::extract_pages(file_path, pages, &output_path.unwrap());
            serde_json::to_value(res).unwrap()
        },
        "watermark" if watermark_text.is_some() => {
            let res = pdf::add_watermark(file_path, &watermark_text.unwrap(), Some(WatermarkOptions { opacity, rotation, ..Default::default() }));
            serde_json::to_value(res).unwrap()
        },
        "get_page" if page_numbers.is_some() => {
            if let Ok(num) = page_numbers.unwrap().parse::<i32>() {
                let res = pdf::get_page_text(file_path, num);
                serde_json::to_value(res).unwrap()
            } else { err("Valid page number required for get_page".to_string()) }
        },
        _ => err(format!("Unknown operation: {}", operation)),
    }
}
