use serde_json::{json, Value};
use crate::services::{word as svc, format_detector, protection};

fn err_msg(msg: String) -> Value {
    json!({"Success": false, "Message": msg})
}

pub fn handle_word_read(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    if file_path.is_empty() { return err_msg("filePath is required".to_string()); }
    let read_type = args.get("readType").and_then(|v| v.as_str()).unwrap_or("all");
    let p_idx = args.get("paragraphIndex").and_then(|v| v.as_i64()).map(|v| v as usize);
    let s_idx = args.get("startIndex").and_then(|v| v.as_i64()).map(|v| v as usize);
    let e_idx = args.get("endIndex").and_then(|v| v.as_i64()).map(|v| v as usize);
    let res = match read_type.to_lowercase().as_str() {
        "paragraph" if p_idx.is_some() => svc::get_paragraph_text(file_path, p_idx.unwrap()),
        "range" if s_idx.is_some() && e_idx.is_some() => svc::get_paragraph_range(file_path, s_idx.unwrap(), e_idx.unwrap()),
        _ => svc::get_document_text(file_path),
    };
    serde_json::to_value(res).unwrap_or(json!({"Success": false}))
}

pub fn handle_word_add_content(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let markdown = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
    let base = args.get("baseImagePath").and_then(|v| v.as_str()).map(|s| s.to_string());
    let res = svc::add_markdown_content(file_path, markdown, base);
    serde_json::to_value(res).unwrap_or(json!({"Success": false}))
}

pub fn handle_word_add_element(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let op = args.get("operation").and_then(|v| v.as_str()).unwrap_or("");
    let res = match op.to_lowercase().as_str() {
        "pagebreak" => svc::add_page_break(file_path),
        "header" => {
            let opts = crate::models::HeaderFooterOptions {
                left_content: args.get("leftContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
                center_content: args.get("centerContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
                right_content: args.get("rightContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
                include_page_number: args.get("includePageNumber").and_then(|v| v.as_bool()).unwrap_or(false),
                include_date: args.get("includeDate").and_then(|v| v.as_bool()).unwrap_or(false),
            };
            svc::add_header(file_path, opts)
        },
        "footer" => {
            let opts = crate::models::HeaderFooterOptions {
                left_content: args.get("leftContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
                center_content: args.get("centerContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
                right_content: args.get("rightContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
                include_page_number: args.get("includePageNumber").and_then(|v| v.as_bool()).unwrap_or(false),
                include_date: args.get("includeDate").and_then(|v| v.as_bool()).unwrap_or(false),
            };
            svc::add_footer(file_path, opts)
        },
        _ => crate::models::DocumentResult { success:false, message: format!("Unknown operation: {}", op), file_path:None, format:None, suggestion: Some("Valid operations: pageBreak, header, footer".to_string()) },
    };
    serde_json::to_value(res).unwrap_or(json!({"Success": false}))
}

pub fn handle_word_add_image(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let image_path = args.get("imagePath").and_then(|v| v.as_str()).unwrap_or("");
    let w = args.get("widthInches").and_then(|v| v.as_f64()).unwrap_or(4.0);
    let h = args.get("heightInches").and_then(|v| v.as_f64()).unwrap_or(3.0);
    let alt = args.get("altText").and_then(|v| v.as_str()).map(|s| s.to_string());
    let opts = crate::models::ImageOptions { width_emu: (w*914400.0) as i64, height_emu: (h*914400.0) as i64, alt_text: alt, ..Default::default() };
    let res = svc::add_image(file_path, image_path, Some(opts));
    serde_json::to_value(res).unwrap_or(json!({"Success": false}))
}

pub fn handle_word_convert(args: &Value) -> Value {
    let source = args.get("sourcePath").and_then(|v| v.as_str()).unwrap_or("");
    let direction = args.get("direction").and_then(|v| v.as_str()).unwrap_or("");
    let output = args.get("outputPath").and_then(|v| v.as_str()).map(|s| s.to_string());
    match direction.to_lowercase().as_str() {
        "word_to_md" => {
            let res = svc::convert_to_markdown(source);
            serde_json::to_value(res).unwrap_or(json!({"Success": false}))
        },
        "word_to_md_file" => {
            let res = svc::convert_to_markdown(source);
            if !res.success { return serde_json::to_value(res).unwrap(); }
            let out = output.unwrap_or_else(|| format!("{}.md", source.trim_end_matches(".docx")));
            match std::fs::write(&out, res.content.unwrap_or_default()) {
                Ok(_) => serde_json::to_value(crate::models::DocumentResult::ok(format!("Converted to '{}'", out), Some(out), Some("md".to_string()))).unwrap(),
                Err(e) => json!({"Success": false, "Message": e.to_string()}),
            }
        },
        "md_to_word" => {
            if !std::path::Path::new(source).exists() {
                return json!({"Success": false, "Message": format!("File not found: {}", source)});
            }
            let md = std::fs::read_to_string(source).unwrap_or_default();
            let out = output.unwrap_or_else(|| source.replace(".md", ".docx"));
            let fmt = format_detector::detect_format(&out).unwrap_or("docx".to_string());
            let res = svc::create_document(&out, None, None, None);
            if !res.success { return serde_json::to_value(res).unwrap(); }
            let r2 = svc::add_markdown_content(&out, &md, None);
            serde_json::to_value(r2).unwrap()
        },
        _ => json!({"Success": false, "Message": format!("Unknown direction: {}", direction)}),
    }
}

pub fn handle_word_batch(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let ops_json = args.get("operationsJson").and_then(|v| v.as_str()).unwrap_or("[]");
    let ops: Result<Vec<crate::models::WordOperation>, _> = serde_json::from_str(ops_json);
    let ops = match ops {
        Ok(o) => o,
        Err(e) => return json!({"Success": false, "Message": format!("Invalid JSON: {}", e)}),
    };
    let mut details = Vec::new();
    let mut succ = 0; let mut fail = 0;
    for (i, op) in ops.iter().enumerate() {
        let r = match op.r#type.to_lowercase().as_str() {
            "markdown" => {
                let md = op.markdown.clone().unwrap_or_default();
                svc::add_markdown_content(file_path, &md, op.base_image_path.clone())
            },
            "heading" => {
                let txt = op.text.clone().unwrap_or_default();
                svc::add_heading(file_path, &txt, op.level.unwrap_or(1), None)
            },
            "paragraph" => {
                let txt = op.text.clone().unwrap_or_default();
                svc::add_paragraph(file_path, &txt, None, None)
            },
            "bulletlist" => {
                let items = op.items.clone().unwrap_or_default();
                svc::add_bullet_list(file_path, items, None)
            },
            "numberedlist" => {
                let items = op.items.clone().unwrap_or_default();
                svc::add_numbered_list(file_path, items, None)
            },
            "table" => {
                let data = op.table_data.clone().unwrap_or_default();
                svc::add_table(file_path, data, None)
            },
            "image" => {
                let p = op.image_path.clone().unwrap_or_default();
                svc::add_image(file_path, &p, None)
            },
            "pagebreak" => svc::add_page_break(file_path),
            "header" => {
                let opts = crate::models::HeaderFooterOptions {
                    left_content: op.left_content.clone(),
                    center_content: op.center_content.clone(),
                    right_content: op.right_content.clone(),
                    include_page_number: op.include_page_number.unwrap_or(false),
                    include_date: op.include_date.unwrap_or(false),
                };
                svc::add_header(file_path, opts)
            },
            "footer" => {
                let opts = crate::models::HeaderFooterOptions {
                    left_content: op.left_content.clone(),
                    center_content: op.center_content.clone(),
                    right_content: op.right_content.clone(),
                    include_page_number: op.include_page_number.unwrap_or(true),
                    include_date: op.include_date.unwrap_or(false),
                };
                svc::add_footer(file_path, opts)
            },
            _ => crate::models::DocumentResult { success:false, message: format!("Unknown type: {}", op.r#type), file_path:None, format:None, suggestion:None },
        };
        if r.success { succ+=1; details.push(crate::models::OperationOutcome { index: i as i32, operation_type: op.r#type.clone(), success: true, message: "Success".to_string() }); }
        else { fail+=1; details.push(crate::models::OperationOutcome { index: i as i32, operation_type: op.r#type.clone(), success: false, message: r.message.clone() }); }
    }
    let res = crate::models::BatchOperationResult { success: fail==0, message: if fail==0 { format!("All {} operations completed", succ) } else { format!("{} succeeded, {} failed", succ, fail) }, total_operations: ops.len() as i32, successful_operations: succ, failed_operations: fail, details };
    serde_json::to_value(res).unwrap()
}
