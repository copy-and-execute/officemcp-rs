use serde_json::{json, Value};
use crate::models::*;
use crate::services::excel as svc;

fn err(msg: String) -> Value { json!({"Success": false, "Message": msg}) }

pub fn handle_excel_create(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let sheet = args.get("sheetName").and_then(|v| v.as_str()).map(|s| s.to_string());
    let initial = args.get("initialDataJson").and_then(|v| v.as_str()).map(|s| s.to_string());
    let has_headers = args.get("hasHeaders").and_then(|v| v.as_bool()).unwrap_or(true);
    let res = svc::create_workbook(file_path, sheet.clone());
    if !res.success { return serde_json::to_value(res).unwrap(); }
    if let Some(data_str) = initial {
        if !data_str.trim().is_empty() {
            if let Ok(data) = serde_json::from_str::<Vec<Vec<String>>>(&data_str) {
                if !data.is_empty() {
                    let r = svc::add_table(file_path, &sheet.unwrap_or("Sheet1".to_string()), "A1", data, has_headers);
                    if !r.success { return serde_json::to_value(r).unwrap(); }
                }
            }
        }
    }
    serde_json::to_value(res).unwrap()
}

pub fn handle_excel_read(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let read_type = args.get("readType").and_then(|v| v.as_str()).unwrap_or("allSheets");
    let sheet_name = args.get("sheetName").and_then(|v| v.as_str()).unwrap_or("");
    let cell_ref = args.get("cellReference").and_then(|v| v.as_str()).unwrap_or("");
    let start_cell = args.get("startCell").and_then(|v| v.as_str()).unwrap_or("");
    let end_cell = args.get("endCell").and_then(|v| v.as_str()).unwrap_or("");
    let include_fmt = args.get("includeFormatting").and_then(|v| v.as_bool()).unwrap_or(false);
    if include_fmt && (read_type.eq_ignore_ascii_case("cell") || read_type.eq_ignore_ascii_case("range")) {
        if read_type.eq_ignore_ascii_case("cell") {
            let r = svc::get_cell_formatting(file_path, sheet_name, cell_ref);
            return serde_json::to_value(r).unwrap();
        } else {
            let r = svc::get_range_formatting(file_path, sheet_name, start_cell, end_cell);
            return serde_json::to_value(r).unwrap();
        }
    }
    let res = match read_type.to_lowercase().as_str() {
        "sheet" if !sheet_name.is_empty() => svc::get_sheet_text(file_path, sheet_name),
        "cell" if !sheet_name.is_empty() && !cell_ref.is_empty() => svc::get_cell_value(file_path, sheet_name, cell_ref),
        "range" if !sheet_name.is_empty() && !start_cell.is_empty() && !end_cell.is_empty() => svc::get_range_values(file_path, sheet_name, start_cell, end_cell),
        _ => svc::get_all_sheets_text(file_path),
    };
    serde_json::to_value(res).unwrap()
}

pub fn handle_excel_get_formatting(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let sheet = args.get("sheetName").and_then(|v| v.as_str()).unwrap_or("");
    let start = args.get("startCell").and_then(|v| v.as_str()).unwrap_or("A1");
    let end = args.get("endCell").and_then(|v| v.as_str()).unwrap_or(start);
    let r = svc::get_range_formatting(file_path, sheet, start, end);
    serde_json::to_value(r).unwrap()
}

pub fn handle_excel_set_cells(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let sheet = args.get("sheetName").and_then(|v| v.as_str()).unwrap_or("Sheet1");
    let start = args.get("startCell").and_then(|v| v.as_str()).unwrap_or("A1");
    let data_str = args.get("dataJson").and_then(|v| v.as_str()).unwrap_or("[]");
    let as_table = args.get("asTable").and_then(|v| v.as_bool()).unwrap_or(false);
    let has_headers = args.get("hasHeaders").and_then(|v| v.as_bool()).unwrap_or(true);
    let data: Result<Vec<Vec<String>>, _> = serde_json::from_str(data_str);
    let data = match data {
        Ok(d) => d,
        Err(e) => return err(format!("Invalid JSON: {}", e)),
    };
    let res = if as_table { svc::add_table(file_path, sheet, start, data, has_headers) } else { svc::set_range_values(file_path, sheet, start, data) };
    serde_json::to_value(res).unwrap()
}

pub fn handle_excel_formula(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let sheet = args.get("sheetName").and_then(|v| v.as_str()).unwrap_or("");
    let cell = args.get("cellReference").and_then(|v| v.as_str()).unwrap_or("");
    let formula = args.get("formula").and_then(|v| v.as_str()).unwrap_or("");
    let res = svc::add_formula(file_path, sheet, cell, formula);
    serde_json::to_value(res).unwrap()
}

pub fn handle_excel_manage_sheet(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let op = args.get("operation").and_then(|v| v.as_str()).unwrap_or("");
    let sheet = args.get("sheetName").and_then(|v| v.as_str()).unwrap_or("");
    let new_name = args.get("newName").and_then(|v| v.as_str());
    let res = match op.to_lowercase().as_str() {
        "add" => svc::add_sheet(file_path, sheet),
        "delete" => svc::delete_sheet(file_path, sheet),
        "rename" if new_name.is_some() => svc::rename_sheet(file_path, sheet, new_name.unwrap()),
        _ => DocumentResult { success:false, message: format!("Unknown operation: {}", op), file_path:None, format:None, suggestion: None },
    };
    serde_json::to_value(res).unwrap()
}

pub fn handle_excel_format_cells(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let sheet = args.get("sheetName").and_then(|v| v.as_str()).unwrap_or("");
    let start = args.get("startCell").and_then(|v| v.as_str()).unwrap_or("A1");
    let end = args.get("endCell").and_then(|v| v.as_str()).unwrap_or(start);
    let fmt = ExcelCellFormatting {
        bold: args.get("bold").and_then(|v| v.as_bool()).unwrap_or(false),
        italic: args.get("italic").and_then(|v| v.as_bool()).unwrap_or(false),
        font_color: args.get("fontColor").and_then(|v| v.as_str()).map(|s| s.to_string()),
        background_color: args.get("fillColor").and_then(|v| v.as_str()).map(|s| s.to_string()),
        number_format: args.get("numberFormat").and_then(|v| v.as_str()).map(|s| s.to_string()),
        horizontal_alignment: args.get("horizontalAlignment").and_then(|v| v.as_str()).unwrap_or("General").to_string(),
        vertical_alignment: args.get("verticalAlignment").and_then(|v| v.as_str()).unwrap_or("Bottom").to_string(),
        wrap_text: args.get("wrapText").and_then(|v| v.as_bool()).unwrap_or(false),
        border_style: args.get("borderStyle").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };
    let res = svc::format_cell_range(file_path, sheet, start, end, fmt);
    serde_json::to_value(res).unwrap()
}

pub fn handle_excel_batch(args: &Value) -> Value {
    let file_path = args.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
    let ops_json = args.get("operationsJson").and_then(|v| v.as_str()).unwrap_or("[]");
    let ops: Result<Vec<ExcelOperation>, _> = serde_json::from_str(ops_json);
    let ops = match ops {
        Ok(o) => o,
        Err(e) => return json!({"Success": false, "Message": format!("Invalid JSON: {}", e)}),
    };
    let mut details = Vec::new();
    let mut succ=0; let mut fail=0;
    for (i, op) in ops.iter().enumerate() {
        let r = match op.r#type.to_lowercase().as_str() {
            "addsheet" => { let n = op.sheet_name.clone().unwrap_or_default(); svc::add_sheet(file_path, &n) },
            "deletesheet" => { let n = op.sheet_name.clone().unwrap_or_default(); svc::delete_sheet(file_path, &n) },
            "renamesheet" => { let a = op.sheet_name.clone().unwrap_or_default(); let b = op.new_sheet_name.clone().unwrap_or_default(); svc::rename_sheet(file_path, &a, &b) },
            "setcellvalue" => { let s = op.sheet_name.clone().unwrap_or("Sheet1".to_string()); let c = op.cell_reference.clone().unwrap_or("A1".to_string()); let v = op.value.clone().unwrap_or_default(); svc::set_cell_value(file_path, &s, &c, &v, None) },
            "setrangevalues" => { let s = op.sheet_name.clone().unwrap_or("Sheet1".to_string()); let sc = op.start_cell.clone().unwrap_or("A1".to_string()); let vals = op.values.clone().unwrap_or_default(); svc::set_range_values(file_path, &s, &sc, vals) },
            "addtable" => { let s = op.sheet_name.clone().unwrap_or("Sheet1".to_string()); let sc = op.start_cell.clone().unwrap_or("A1".to_string()); let d = op.table_data.clone().unwrap_or_default(); svc::add_table(file_path, &s, &sc, d, op.has_headers.unwrap_or(true)) },
            "addformula" => { let s = op.sheet_name.clone().unwrap_or_default(); let c = op.cell_reference.clone().unwrap_or_default(); let f = op.formula.clone().unwrap_or_default(); svc::add_formula(file_path, &s, &c, &f) },
            "mergecells" => { let s = op.sheet_name.clone().unwrap_or_default(); let a = op.start_cell.clone().unwrap_or_default(); let b = op.end_cell.clone().unwrap_or_default(); svc::merge_cells(file_path, &s, &a, &b) },
            "setcolumnwidth" => { let s = op.sheet_name.clone().unwrap_or_default(); svc::set_column_width(file_path, &s, op.column_index.unwrap_or(1), op.width.unwrap_or(10.0)) },
            "setrowheight" => { let s = op.sheet_name.clone().unwrap_or_default(); svc::set_row_height(file_path, &s, op.row_index.unwrap_or(1), op.height.unwrap_or(15.0)) },
            "addimage" => { let s = op.sheet_name.clone().unwrap_or_default(); let p = op.image_path.clone().unwrap_or_default(); let c = op.cell_reference.clone().unwrap_or("A1".to_string()); svc::add_image(file_path, &s, &p, &c, None) },
            "formatcells" => {
                let s = op.sheet_name.clone().unwrap_or_default();
                let sc = op.start_cell.clone().unwrap_or("A1".to_string());
                let ec = op.end_cell.clone().unwrap_or(sc.clone());
                let fmt = ExcelCellFormatting {
                    bold: op.bold.unwrap_or(false),
                    italic: op.italic.unwrap_or(false),
                    font_color: op.font_color.clone(),
                    background_color: op.fill_color.clone(),
                    number_format: op.number_format.clone(),
                    horizontal_alignment: op.horizontal_alignment.clone().unwrap_or("General".to_string()),
                    vertical_alignment: op.vertical_alignment.clone().unwrap_or("Bottom".to_string()),
                    wrap_text: op.wrap_text.unwrap_or(false),
                    border_style: op.border_style.clone(),
                };
                svc::format_cell_range(file_path, &s, &sc, &ec, fmt)
            },
            _ => DocumentResult { success:false, message: format!("Unknown type: {}", op.r#type), file_path:None, format:None, suggestion:None },
        };
        if r.success { succ+=1; details.push(OperationOutcome { index: i as i32, operation_type: op.r#type.clone(), success:true, message:"Success".to_string()}); } else { fail+=1; details.push(OperationOutcome { index: i as i32, operation_type: op.r#type.clone(), success:false, message:r.message.clone()}); }
    }
    let res = BatchOperationResult { success: fail==0, message: if fail==0 {format!("All {} operations completed", succ)} else {format!("{} succeeded, {} failed", succ, fail)}, total_operations: ops.len() as i32, successful_operations: succ, failed_operations: fail, details };
    serde_json::to_value(res).unwrap()
}
