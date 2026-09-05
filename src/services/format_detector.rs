use std::path::Path;

pub fn detect_format(file_path: &str) -> Result<String, String> {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "docx" => Ok("docx".to_string()),
        "xlsx" => Ok("xlsx".to_string()),
        "pptx" => Ok("pptx".to_string()),
        "pdf" => Ok("pdf".to_string()),
        "md" => Ok("md".to_string()),
        _ => Err(format!(
            "Unsupported format: .{}. Supported: .docx, .xlsx, .pptx, .pdf, .md",
            ext
        )),
    }
}

pub fn is_supported(format: &str) -> bool {
    matches!(
        format.to_lowercase().as_str(),
        "docx" | "xlsx" | "pptx" | "pdf" | "md"
    )
}

pub fn uses_unified_interface(format: &str) -> bool {
    matches!(format.to_lowercase().as_str(), "docx" | "pdf")
}
