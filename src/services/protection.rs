use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct FileProtectionInfo {
    pub is_protected: bool,
    pub is_encrypted: bool,
    pub may_have_sensitivity_label: bool,
    pub is_valid_office_format: bool,
    pub protection_type: String,
    pub error_message: Option<String>,
}

pub fn check_file_protection(file_path: &str) -> FileProtectionInfo {
    if !Path::new(file_path).exists() {
        return FileProtectionInfo {
            is_protected: false,
            is_encrypted: false,
            may_have_sensitivity_label: false,
            is_valid_office_format: false,
            protection_type: "Unknown".to_string(),
            error_message: Some(format!("File not found: {}", file_path)),
        };
    }
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "docx" | "xlsx" | "pptx" => check_openxml_protection(file_path),
        "pdf" => check_pdf_protection(file_path),
        _ => FileProtectionInfo {
            is_protected: false,
            is_encrypted: false,
            may_have_sensitivity_label: false,
            is_valid_office_format: false,
            protection_type: "Unknown".to_string(),
            error_message: Some(format!("Unsupported file type: .{}", ext)),
        },
    }
}

fn check_openxml_protection(file_path: &str) -> FileProtectionInfo {
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            return FileProtectionInfo {
                is_protected: true,
                is_encrypted: true,
                may_have_sensitivity_label: false,
                is_valid_office_format: false,
                protection_type: "Error".to_string(),
                error_message: Some(e.to_string()),
            }
        }
    };
    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => {
            return FileProtectionInfo {
                is_protected: true,
                is_encrypted: true,
                may_have_sensitivity_label: true,
                is_valid_office_format: false,
                protection_type: "Encrypted".to_string(),
                error_message: Some("File is encrypted and cannot be opened directly".to_string()),
            }
        }
    };
    // check content types
    let has_content_types = (0..archive.len()).any(|i| {
        archive.by_index(i).ok().map(|f| f.name() == "[Content_Types].xml").unwrap_or(false)
    });
    if !has_content_types {
        return FileProtectionInfo {
            is_protected: true,
            is_encrypted: true,
            may_have_sensitivity_label: false,
            is_valid_office_format: false,
            protection_type: "Encrypted".to_string(),
            error_message: Some("File appears to be encrypted (invalid structure)".to_string()),
        };
    }
    let mut may_have_label = false;
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_lowercase();
            if name.contains("labelinfo") || name.contains("protection") {
                may_have_label = true;
                break;
            }
        }
    }
    FileProtectionInfo {
        is_protected: may_have_label,
        is_encrypted: false,
        may_have_sensitivity_label: may_have_label,
        is_valid_office_format: true,
        protection_type: if may_have_label { "SensitivityLabel".to_string() } else { "None".to_string() },
        error_message: None,
    }
}

fn check_pdf_protection(file_path: &str) -> FileProtectionInfo {
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => return FileProtectionInfo {
            is_protected: true, is_encrypted: true, may_have_sensitivity_label: false,
            is_valid_office_format: false, protection_type: "Unknown".to_string(),
            error_message: Some(e.to_string()),
        },
    };
    let mut buf = vec![0u8; 4096];
    let n = file.read(&mut buf).unwrap_or(0);
    let content = String::from_utf8_lossy(&buf[..n]);
    let is_encrypted = content.to_lowercase().contains("/encrypt");
    FileProtectionInfo {
        is_protected: is_encrypted,
        is_encrypted,
        may_have_sensitivity_label: false,
        is_valid_office_format: true,
        protection_type: if is_encrypted { "Password".to_string() } else { "None".to_string() },
        error_message: None,
    }
}

pub fn get_protection_error_message(info: &FileProtectionInfo, file_path: &str) -> String {
    if !info.is_protected { return String::new(); }
    let file_name = Path::new(file_path).file_name().and_then(|n| n.to_str()).unwrap_or(file_path);
    match info.protection_type.as_str() {
        "Encrypted" if info.may_have_sensitivity_label => format!("'{}' is protected with a Microsoft sensitivity label. You need appropriate permissions to access this file.", file_name),
        "Encrypted" => format!("'{}' is encrypted or password-protected. Remove the protection in the native application before using this tool.", file_name),
        "SensitivityLabel" => format!("'{}' has a sensitivity label applied. Content may be readable but some operations may be restricted.", file_name),
        "Password" => format!("'{}' is password-protected. Provide the password or remove protection in the native application.", file_name),
        _ => format!("'{}' has unknown protection: {}", file_name, info.protection_type),
    }
}
