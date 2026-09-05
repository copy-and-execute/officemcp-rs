use crate::models::*;
use crate::services::markdown_parser::{parse as parse_markdown, MarkdownElement, MarkdownInline};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::{ZipArchive, ZipWriter};
use zip::write::SimpleFileOptions;

const EMUS_PER_INCH: i64 = 914400;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// Minimal docx skeleton
fn content_types_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="png" ContentType="image/png"/>
  <Default Extension="jpg" ContentType="image/jpeg"/>
  <Default Extension="jpeg" ContentType="image/jpeg"/>
  <Default Extension="gif" ContentType="image/gif"/>
  <Default Extension="bmp" ContentType="image/bmp"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>"#.to_string()
}

fn rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#.to_string()
}

fn core_xml(title: &Option<String>) -> String {
    let t = title.as_deref().unwrap_or("Document");
    format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>{}</dc:title>
  <dc:creator>OfficeMCP</dc:creator>
  <cp:revision>1</cp:revision>
</cp:coreProperties>"#, xml_escape(t))
}

fn app_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
  <Application>OfficeMCP</Application>
  <DocSecurity>0</DocSecurity>
  <ScaleCrop>false</ScaleCrop>
  <Company></Company>
  <LinksUpToDate>false</LinksUpToDate>
  <SharedDoc>false</SharedDoc>
  <HyperlinksChanged>false</HyperlinksChanged>
  <AppVersion>16.0000</AppVersion>
</Properties>"#.to_string()
}

fn styles_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:cs="Calibri"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults>
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="160"/></w:pPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:keepNext/><w:keepLines/><w:spacing w:before="240" w:after="0"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:color w:val="2E74B5"/><w:sz w:val="32"/><w:szCs w:val="32"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:keepNext/><w:keepLines/><w:spacing w:before="240" w:after="0"/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:color w:val="2E74B5"/><w:sz w:val="26"/><w:szCs w:val="26"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:spacing w:before="240" w:after="0"/><w:outlineLvl w:val="2"/></w:pPr><w:rPr><w:b/><w:color w:val="1F4D78"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading4"><w:name w:val="heading 4"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:outlineLvl w:val="3"/></w:pPr><w:rPr><w:b/><w:sz w:val="20"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading5"><w:name w:val="heading 5"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:outlineLvl w:val="4"/></w:pPr><w:rPr><w:b/><w:sz w:val="18"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading6"><w:name w:val="heading 6"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:outlineLvl w:val="5"/></w:pPr><w:rPr><w:b/><w:sz w:val="16"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:spacing w:after="0"/><w:jc w:val="center"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="56"/><w:szCs w:val="56"/></w:rPr></w:style>
  <w:style w:type="character" w:styleId="DefaultParagraphFont"><w:name w:val="Default Paragraph Font"/><w:semiHidden/><w:unhideWhenUsed/></w:style>
</w:styles>"#.to_string()
}

fn numbering_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml">
  <w:abstractNum w:abstractNumId="0"><w:nsid w:val="2A5B3C4D"/><w:multiLevelType w:val="hybridMultilevel"/><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val=""/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr><w:rPr><w:rFonts w:ascii="Symbol" w:hAnsi="Symbol" w:hint="default"/></w:rPr></w:lvl><w:lvl w:ilvl="1"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="o"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="1440" w:hanging="360"/></w:pPr></w:lvl></w:abstractNum>
  <w:abstractNum w:abstractNumId="1"><w:nsid w:val="3B6C4D5E"/><w:multiLevelType w:val="hybridMultilevel"/><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr></w:lvl></w:abstractNum>
  <w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>
  <w:num w:numId="2"><w:abstractNumId w:val="1"/></w:num>
</w:numbering>"#.to_string()
}

fn document_rels_xml(extra: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>
{}
</Relationships>"#, extra)
}

fn document_xml(body_content: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
  <w:body>
{}
    <w:sectPr>
      <w:pgSz w:w="12240" w:h="15840" w:orient="portrait"/>
      <w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>
      <w:cols w:space="720"/>
      <w:docGrid w:linePitch="360"/>
    </w:sectPr>
  </w:body>
</w:document>"#, body_content)
}

fn heading_xml(text: &str, level: usize) -> String {
    let lvl = level.clamp(1, 6);
    format!(r#"    <w:p><w:pPr><w:pStyle w:val="Heading{}"/><w:spacing w:before="240" w:after="120"/></w:pPr><w:r><w:rPr><w:b/><w:sz w:val="{}"/><w:szCs w:val="{}"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#,
        lvl,
        match lvl { 1=>32, 2=>26, 3=>22, 4=>20, 5=>18, _=>16 },
        match lvl { 1=>32, 2=>26, 3=>22, 4=>20, 5=>18, _=>16 },
        xml_escape(text)
    )
}

fn paragraph_xml(text: &str) -> String {
    if text.trim().is_empty() {
        return r#"    <w:p><w:r><w:t xml:space="preserve"></w:t></w:r></w:p>"#.to_string();
    }
    // handle simple inline **bold** ?? For now, treat as plain.
    format!(r#"    <w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#, xml_escape(text))
}

fn bullet_xml(text: &str, numbered: bool) -> String {
    let num_id = if numbered { 2 } else { 1 };
    format!(r#"    <w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="{}"/></w:numPr><w:ind w:left="720" w:hanging="360"/></w:pPr><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#, num_id, xml_escape(text))
}

fn table_xml(data: &[Vec<String>]) -> String {
    if data.is_empty() { return String::new(); }
    let cols = data[0].len();
    let mut out = String::new();
    out.push_str(r#"    <w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:left w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:right w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="000000"/></w:tblBorders><w:tblLook w:val="04A0"/></w:tblPr><w:tblGrid>"#);
    for _ in 0..cols { out.push_str(r#"<w:gridCol w:w="2000"/>"#); }
    out.push_str("</w:tblGrid>");
    for (ri, row) in data.iter().enumerate() {
        out.push_str("<w:tr>");
        let is_header = ri == 0;
        for cell in row {
            out.push_str("<w:tc><w:tcPr>");
            if is_header { out.push_str(r#"<w:shd w:val="clear" w:color="auto" w:fill="4472C4"/>"#); }
            out.push_str("</w:tcPr><w:p>");
            if is_header { out.push_str(r#"<w:r><w:rPr><w:b/><w:color w:val="FFFFFF"/></w:rPr>"#); } else { out.push_str("<w:r>"); }
            out.push_str(&format!("<w:t xml:space=\"preserve\">{}</w:t></w:r></w:p></w:tc>", xml_escape(cell)));
        }
        out.push_str("</w:tr>");
    }
    out.push_str("</w:tbl>");
    out
}

fn page_break_xml() -> String {
    r#"    <w:p><w:r><w:br w:type="page"/></w:r></w:p>"#.to_string()
}

fn read_document_xml(file_path: &str) -> Result<String, String> {
    let file = File::open(file_path).map_err(|e| format!("Failed to open docx: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid docx: {}", e))?;
    let mut entry = archive.by_name("word/document.xml").map_err(|_| "word/document.xml not found".to_string())?;
    let mut s = String::new();
    entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
    Ok(s)
}

fn write_document_xml(file_path: &str, new_doc_xml: &str) -> Result<(), String> {
    // read all entries, replace word/document.xml
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).map_err(|e| e.to_string())?;
        if name == "word/document.xml" {
            data = new_doc_xml.as_bytes().to_vec();
        }
        entries.push((name, data));
    }
    // also need to ensure required files exist even if template missing
    let tmp = format!("{}.tmp", file_path);
    let out = File::create(&tmp).map_err(|e| e.to_string())?;
    let mut writer = ZipWriter::new(out);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, data) in entries {
        writer.start_file(&name, opts).map_err(|e| e.to_string())?;
        writer.write_all(&data).map_err(|e| e.to_string())?;
    }
    writer.finish().map_err(|e| e.to_string())?;
    fs::remove_file(file_path).map_err(|e| e.to_string())?;
    fs::rename(&tmp, file_path).map_err(|e| e.to_string())?;
    Ok(())
}

fn insert_before_sectpr(doc_xml: &str, fragments: &[String]) -> String {
    let joined = fragments.join("\n");
    if let Some(pos) = doc_xml.rfind("<w:sectPr") {
        let mut out = String::new();
        out.push_str(&doc_xml[..pos]);
        out.push_str(&joined);
        out.push('\n');
        out.push_str(&doc_xml[pos..]);
        out
    } else if let Some(pos) = doc_xml.rfind("</w:body>") {
        let mut out = String::new();
        out.push_str(&doc_xml[..pos]);
        out.push_str(&joined);
        out.push('\n');
        out.push_str(&doc_xml[pos..]);
        out
    } else {
        doc_xml.replace("</w:body>", &format!("{}\n</w:body>", joined))
    }
}

fn ensure_dir(path: &str) {
    if let Some(parent) = Path::new(path).parent() {
        let _ = fs::create_dir_all(parent);
    }
}

pub fn create_document(file_path: &str, title: Option<String>, _layout: Option<PageLayoutOptions>, template_path: Option<String>) -> DocumentResult {
    ensure_dir(file_path);
    if let Some(tpl) = template_path {
        if !Path::new(&tpl).exists() {
            return DocumentResult { success: false, message: format!("Template not found: {}", tpl), file_path: None, format: None, suggestion: None };
        }
        // copy template and clear body but keep sectPr
        if let Err(e) = fs::copy(&tpl, file_path) {
            return DocumentResult { success: false, message: format!("Failed to copy template: {}", e), file_path: None, format: None, suggestion: None };
        }
        // clear body content but keep sectPr
        let doc_xml = match read_document_xml(file_path) {
            Ok(s) => s,
            Err(e) => return DocumentResult { success: false, message: e, file_path: None, format: None, suggestion: None },
        };
        // extract sectPr
        let sectpr = if let Some(start) = doc_xml.find("<w:sectPr") {
            if let Some(end) = doc_xml[start..].find("</w:sectPr>") {
                doc_xml[start..start+end+11].to_string()
            } else if let Some(end) = doc_xml[start..].find("/>") {
                doc_xml[start..start+end+2].to_string()
            } else { String::new() }
        } else { String::new() };
        let body = if let Some(t) = title { heading_xml(&t, 1) } else { String::new() };
        let new_body = if sectpr.is_empty() {
            body
        } else {
            format!("{}\n    {}", body, sectpr)
        };
        let new_doc = document_xml(&new_body);
        if let Err(e) = write_document_xml(file_path, &new_doc) {
            return DocumentResult { success: false, message: e, file_path: None, format: None, suggestion: None };
        }
        return DocumentResult::ok(format!("Document created from template at {}", file_path), Some(file_path.to_string()), Some("docx".to_string()));
    }
    // create minimal docx
    let body = if let Some(t) = title { heading_xml(&t, 1) } else { String::new() };
    let doc_xml = document_xml(&body);
    let tmp = format!("{}.tmp", file_path);
    let file = match File::create(&tmp) {
        Ok(f) => f,
        Err(e) => return DocumentResult { success: false, message: format!("Failed to create: {}", e), file_path: None, format: None, suggestion: None },
    };
    let mut writer = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let entries = vec![
        ("[Content_Types].xml", content_types_xml()),
        ("_rels/.rels", rels_xml()),
        ("docProps/core.xml", core_xml(&None)),
        ("docProps/app.xml", app_xml()),
        ("word/document.xml", doc_xml),
        ("word/styles.xml", styles_xml()),
        ("word/numbering.xml", numbering_xml()),
        ("word/_rels/document.xml.rels", document_rels_xml("")),
        ("word/fontTable.xml", r#"<?xml version="1.0" encoding="UTF-8"?><w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:font w:name="Calibri"/></w:fonts>"#.to_string()),
        ("word/settings.xml", r#"<?xml version="1.0" encoding="UTF-8"?><w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:compat/></w:settings>"#.to_string()),
    ];
    for (name, content) in entries {
        if let Err(e) = writer.start_file(name, opts) { return DocumentResult { success: false, message: e.to_string(), file_path: None, format: None, suggestion: None }; }
        if let Err(e) = writer.write_all(content.as_bytes()) { return DocumentResult { success: false, message: e.to_string(), file_path: None, format: None, suggestion: None }; }
    }
    if let Err(e) = writer.finish() { return DocumentResult { success: false, message: e.to_string(), file_path: None, format: None, suggestion: None }; }
    let _ = fs::remove_file(file_path);
    if let Err(e) = fs::rename(&tmp, file_path) { return DocumentResult { success: false, message: e.to_string(), file_path: None, format: None, suggestion: None }; }
    DocumentResult::ok(format!("Document created successfully at {}", file_path), Some(file_path.to_string()), Some("docx".to_string()))
}

fn add_fragments(file_path: &str, fragments: Vec<String>) -> DocumentResult {
    let doc_xml = match read_document_xml(file_path) {
        Ok(s) => s,
        Err(e) => return DocumentResult { success: false, message: e, file_path: None, format: None, suggestion: None },
    };
    let new_xml = insert_before_sectpr(&doc_xml, &fragments);
    if let Err(e) = write_document_xml(file_path, &new_xml) {
        return DocumentResult { success: false, message: e, file_path: None, format: None, suggestion: None };
    }
    DocumentResult::ok("Updated".to_string(), Some(file_path.to_string()), Some("docx".to_string()))
}

pub fn add_paragraph(file_path: &str, text: &str, _fmt: Option<TextFormatting>, _pfmt: Option<ParagraphFormatting>) -> DocumentResult {
    let mut r = add_fragments(file_path, vec![paragraph_xml(text)]);
    r.message = "Paragraph added successfully".to_string();
    r
}
pub fn add_heading(file_path: &str, text: &str, level: i32, _fmt: Option<TextFormatting>) -> DocumentResult {
    let mut r = add_fragments(file_path, vec![heading_xml(text, level as usize)]);
    r.message = format!("Heading level {} added successfully", level);
    r
}
pub fn add_table(file_path: &str, data: Vec<Vec<String>>, _fmt: Option<TableFormatting>) -> DocumentResult {
    if data.is_empty() {
        return DocumentResult { success: false, message: "Table data cannot be empty".to_string(), file_path: None, format: None, suggestion: None };
    }
    let mut r = add_fragments(file_path, vec![table_xml(&data)]);
    r.message = format!("Table with {} rows added successfully", data.len());
    r
}
pub fn add_image(file_path: &str, image_path: &str, options: Option<ImageOptions>) -> DocumentResult {
    let opt = options.unwrap_or_default();
    if !Path::new(image_path).exists() {
        return DocumentResult { success: false, message: format!("Image file not found: {}", image_path), file_path: None, format: None, suggestion: None };
    }
    // read image
    let data = match fs::read(image_path) {
        Ok(d) => d,
        Err(e) => return DocumentResult { success: false, message: e.to_string(), file_path: None, format: None, suggestion: None },
    };
    let ext = Path::new(image_path).extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
    let media_name = format!("word/media/image{}.{}", {
        // count existing media
        let f = File::open(file_path).unwrap();
        let mut za = ZipArchive::new(f).unwrap();
        let mut count = 0;
        for i in 0..za.len() { if za.by_index(i).unwrap().name().starts_with("word/media/") { count+=1; } }
        count+1
    }, ext);
    // read all entries and add new media + update rels and document
    let file = File::open(file_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut doc_xml = String::new();
    let mut rels_xml_content = String::new();
    let mut content_types = String::new();
    for i in 0..archive.len() {
        let mut e = archive.by_index(i).unwrap();
        let name = e.name().to_string();
        let mut d = Vec::new();
        e.read_to_end(&mut d).unwrap();
        if name == "word/document.xml" { doc_xml = String::from_utf8(d.clone()).unwrap(); }
        if name == "word/_rels/document.xml.rels" { rels_xml_content = String::from_utf8(d.clone()).unwrap(); }
        if name == "[Content_Types].xml" { content_types = String::from_utf8(d.clone()).unwrap(); }
        entries.push((name, d));
    }
    // update content types if needed
    if !content_types.contains(&format!("Extension=\"{}\"", ext)) {
        let insert = format!(r#"  <Default Extension="{}" ContentType="image/{}"/>"#, ext, if ext=="jpg" {"jpeg"} else {&ext});
        content_types = content_types.replace("</Types>", &format!("{}\n</Types>", insert));
        for (n,d) in entries.iter_mut() { if n=="[Content_Types].xml" { *d = content_types.clone().into_bytes(); } }
    }
    // add media entry
    entries.push((media_name.clone(), data));
    // new relationship id
    let rel_id = {
        let mut max = 2;
        for _ in rels_xml_content.matches("Id=\"rId") { max+=1; }
        // count rels
        let count = rels_xml_content.matches("Relationship").count();
        format!("rId{}", count+5)
    };
    // update rels
    let new_rel = format!(r#"  <Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="{}"/>"#, rel_id, media_name.replace("word/", ""));
    rels_xml_content = rels_xml_content.replace("</Relationships>", &format!("{}\n</Relationships>", new_rel));
    for (n,d) in entries.iter_mut() { if n=="word/_rels/document.xml.rels" { *d = rels_xml_content.clone().into_bytes(); } }
    // create drawing fragment
    let cx = opt.width_emu;
    let cy = opt.height_emu;
    let alt = xml_escape(&opt.alt_text.unwrap_or_default());
    let drawing = format!(r#"    <w:p><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{}" cy="{}"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:docPr id="1" name="Picture 1" descr="{}"/><wp:cNvGraphicFramePr><a:graphicFrameLocks xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" noChangeAspect="1"/></wp:cNvGraphicFramePr><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="Image"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{}" cy="{}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#, cx, cy, alt, rel_id, cx, cy);
    let new_doc_xml = insert_before_sectpr(&doc_xml, &[drawing]);
    for (n,d) in entries.iter_mut() { if n=="word/document.xml" { *d = new_doc_xml.clone().into_bytes(); } }
    // write temp zip
    let tmp = format!("{}.tmp", file_path);
    let out = File::create(&tmp).unwrap();
    let mut w = ZipWriter::new(out);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, data) in entries {
        w.start_file(&name, opts).unwrap();
        w.write_all(&data).unwrap();
    }
    w.finish().unwrap();
    fs::remove_file(file_path).unwrap();
    fs::rename(&tmp, file_path).unwrap();
    DocumentResult::ok("Image added successfully".to_string(), Some(file_path.to_string()), Some("docx".to_string()))
}
pub fn add_header(file_path: &str, opts: HeaderFooterOptions) -> DocumentResult {
    let header_content = {
        let mut parts = Vec::new();
        if let Some(s) = opts.left_content { parts.push(xml_escape(&s)); }
        if let Some(s) = opts.center_content { parts.push(xml_escape(&s)); }
        if let Some(s) = opts.right_content { parts.push(xml_escape(&s)); }
        if opts.include_page_number { parts.push("Page".to_string()); }
        if opts.include_date { parts.push("Date".to_string()); }
        if parts.is_empty() { "Header".to_string() } else { parts.join(" | ") }
    };
    let header_xml = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p></w:hdr>"#, xml_escape(&header_content));
    if let Err(e) = add_header_or_footer(file_path, header_xml, true) {
        return DocumentResult { success: false, message: e, file_path: None, format: None, suggestion: None };
    }
    DocumentResult::ok("Header added successfully".to_string(), Some(file_path.to_string()), Some("docx".to_string()))
}
pub fn add_footer(file_path: &str, opts: HeaderFooterOptions) -> DocumentResult {
    let footer_content = {
        let mut parts = Vec::new();
        if let Some(s) = opts.left_content { parts.push(xml_escape(&s)); }
        if let Some(s) = opts.center_content { parts.push(xml_escape(&s)); }
        if let Some(s) = opts.right_content { parts.push(xml_escape(&s)); }
        if opts.include_page_number { parts.push("Page".to_string()); }
        if opts.include_date { parts.push("Date".to_string()); }
        if parts.is_empty() { "Footer".to_string() } else { parts.join(" | ") }
    };
    let footer_xml = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p></w:ftr>"#, xml_escape(&footer_content));
    if let Err(e) = add_header_or_footer(file_path, footer_xml, false) {
        return DocumentResult { success: false, message: e, file_path: None, format: None, suggestion: None };
    }
    DocumentResult::ok("Footer added successfully".to_string(), Some(file_path.to_string()), Some("docx".to_string()))
}

fn add_header_or_footer(file_path: &str, xml: String, is_header: bool) -> Result<(), String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut content_types = String::new();
    let mut rels_content = String::new();
    let mut doc_xml = String::new();
    for i in 0..archive.len() {
        let mut e = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = e.name().to_string();
        let mut d = Vec::new();
        e.read_to_end(&mut d).map_err(|e| e.to_string())?;
        if name == "[Content_Types].xml" { content_types = String::from_utf8(d.clone()).unwrap_or_default(); }
        if name == "word/_rels/document.xml.rels" { rels_content = String::from_utf8(d.clone()).unwrap_or_default(); }
        if name == "word/document.xml" { doc_xml = String::from_utf8(d.clone()).unwrap_or_default(); }
        entries.push((name, d));
    }
    let part_name = if is_header { "word/header1.xml" } else { "word/footer1.xml" };
    let content_type = if is_header { "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml" } else { "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml" };
    let rel_type = if is_header { "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" } else { "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" };
    // update content types
    if !content_types.contains(part_name) {
        let override_entry = format!(r#"  <Override PartName="/{}" ContentType="{}"/>"#, part_name, content_type);
        content_types = content_types.replace("</Types>", &format!("{}\n</Types>", override_entry));
        for (n,d) in entries.iter_mut() { if n=="[Content_Types].xml" { *d = content_types.clone().into_bytes(); } }
    }
    // add part if not exists
    if !entries.iter().any(|(n,_)| n==part_name) {
        entries.push((part_name.to_string(), xml.into_bytes()));
    } else {
        for (n,d) in entries.iter_mut() { if n==part_name { *d = xml.clone().into_bytes(); } }
    }
    // add relationship
    let rel_id = {
        let count = rels_content.matches("Relationship").count() + 1;
        format!("rId{}", 100+count)
    };
    if !rels_content.contains(rel_type) {
        let rel_entry = format!(r#"  <Relationship Id="{}" Type="{}" Target="{}"/>"#, rel_id, rel_type, if is_header { "header1.xml" } else { "footer1.xml" });
        rels_content = rels_content.replace("</Relationships>", &format!("{}\n</Relationships>", rel_entry));
        for (n,d) in entries.iter_mut() { if n=="word/_rels/document.xml.rels" { *d = rels_content.clone().into_bytes(); } }
    }
    // update sectPr to add headerReference/footerReference
    if !doc_xml.contains("w:headerReference") && is_header {
        let header_ref = format!(r#"<w:headerReference w:type="default" r:id="{}"/>"#, rel_id);
        // insert before </w:sectPr> or inside sectPr
        if doc_xml.contains("<w:sectPr") {
            doc_xml = doc_xml.replacen("<w:sectPr", &format!("<w:sectPr>{}", header_ref), 1);
            // Actually need to insert inside sectPr, after pgSz/pgMar etc. Simpler: insert after <w:sectPr>
            // Already did, but need to ensure headerReference is inside sectPr
            // Our simple replace puts it right after <w:sectPr, need to add closing? We'll just insert before </w:sectPr>
            if doc_xml.contains("</w:sectPr>") {
                doc_xml = doc_xml.replace("</w:sectPr>", &format!("{}</w:sectPr>", header_ref));
                // remove duplicate we inserted at start
                doc_xml = doc_xml.replacen(&header_ref, "", 1);
            }
        }
        for (n,d) in entries.iter_mut() { if n=="word/document.xml" { *d = doc_xml.clone().into_bytes(); } }
    } else if !doc_xml.contains("w:footerReference") && !is_header {
        let footer_ref = format!(r#"<w:footerReference w:type="default" r:id="{}"/>"#, rel_id);
        if doc_xml.contains("<w:sectPr") {
            if doc_xml.contains("</w:sectPr>") {
                doc_xml = doc_xml.replace("</w:sectPr>", &format!("{}</w:sectPr>", footer_ref));
            }
        }
        for (n,d) in entries.iter_mut() { if n=="word/document.xml" { *d = doc_xml.clone().into_bytes(); } }
    }
    // write new zip
    let tmp = format!("{}.tmp", file_path);
    let out = File::create(&tmp).map_err(|e| e.to_string())?;
    let mut writer = ZipWriter::new(out);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, data) in entries {
        writer.start_file(&name, opts).map_err(|e| e.to_string())?;
        writer.write_all(&data).map_err(|e| e.to_string())?;
    }
    writer.finish().map_err(|e| e.to_string())?;
    std::fs::remove_file(file_path).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, file_path).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn add_page_break(file_path: &str) -> DocumentResult {
    let mut r = add_fragments(file_path, vec![page_break_xml()]);
    r.message = "Page break added successfully".to_string();
    r
}
pub fn add_bullet_list(file_path: &str, items: Vec<String>, _fmt: Option<TextFormatting>) -> DocumentResult {
    let frags: Vec<String> = items.iter().map(|t| bullet_xml(t, false)).collect();
    let mut r = add_fragments(file_path, frags);
    r.message = "Bullet list added".to_string();
    r
}
pub fn add_numbered_list(file_path: &str, items: Vec<String>, _fmt: Option<TextFormatting>) -> DocumentResult {
    let frags: Vec<String> = items.iter().map(|t| bullet_xml(t, true)).collect();
    let mut r = add_fragments(file_path, frags);
    r.message = "Numbered list added".to_string();
    r
}
pub fn get_document_text(file_path: &str) -> ContentResult {
    if !Path::new(file_path).exists() {
        return ContentResult { success: false, content: None, error_message: Some(format!("File not found: {}", file_path)), total_paragraphs: None, total_pages: None, format: Some("docx".to_string()), suggestion: Some("Verify file path".to_string()) };
    }
    let doc_xml = match read_document_xml(file_path) {
        Ok(s) => s,
        Err(e) => return ContentResult { success: false, content: None, error_message: Some(e), total_paragraphs: None, total_pages: None, format: None, suggestion: None },
    };
    // extract w:t
    let re = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
    let mut texts: Vec<String> = Vec::new();
    let mut count = 0;
    for cap in re.captures_iter(&doc_xml) {
        count+=1;
        texts.push(cap[1].to_string());
    }
    // group by paragraph: split on </w:p>
    let paras: Vec<String> = doc_xml.split("</w:p>").map(|p| {
        let mut para_text = String::new();
        for cap in re.captures_iter(p) {
            para_text.push_str(&cap[1]);
            para_text.push(' ');
        }
        para_text.trim().to_string()
    }).filter(|s| !s.is_empty()).collect();
    // paragraphs count from split
    let para_count = doc_xml.matches("<w:p").count() as i32;
    let content = paras.join("\n");
    ContentResult { success: true, content: Some(content), error_message: None, total_paragraphs: Some(para_count), total_pages: None, format: Some("docx".to_string()), suggestion: None }
}
pub fn get_paragraph_text(file_path: &str, idx: usize) -> ContentResult {
    let res = get_document_text(file_path);
    if !res.success { return res; }
    let content = res.content.unwrap_or_default();
    let paras: Vec<&str> = content.split('\n').collect();
    if idx >= paras.len() {
        return ContentResult { success: false, content: None, error_message: Some(format!("Paragraph index {} out of range. Document has {} paragraphs.", idx, paras.len())), total_paragraphs: Some(paras.len() as i32), total_pages: None, format: Some("docx".to_string()), suggestion: None };
    }
    ContentResult { success: true, content: Some(paras[idx].to_string()), error_message: None, total_paragraphs: Some(paras.len() as i32), total_pages: None, format: Some("docx".to_string()), suggestion: None }
}
pub fn get_paragraph_range(file_path: &str, start: usize, end: usize) -> ContentResult {
    let res = get_document_text(file_path);
    if !res.success { return res; }
    let content = res.content.unwrap_or_default();
    let paras: Vec<&str> = content.split('\n').collect();
    if start > end || end >= paras.len() {
        return ContentResult { success: false, content: None, error_message: Some(format!("Invalid range [{}, {}]. Document has {} paragraphs.", start, end, paras.len())), total_paragraphs: Some(paras.len() as i32), total_pages: None, format: Some("docx".to_string()), suggestion: None };
    }
    let slice = paras[start..=end].join("\n");
    ContentResult { success: true, content: Some(slice), error_message: None, total_paragraphs: Some(paras.len() as i32), total_pages: None, format: Some("docx".to_string()), suggestion: None }
}
pub fn set_page_layout(file_path: &str, opts: PageLayoutOptions) -> DocumentResult {
    let mut doc_xml = match read_document_xml(file_path) {
        Ok(s) => s,
        Err(e) => return DocumentResult { success: false, message: e, file_path: None, format: None, suggestion: None },
    };
    // Determine page size in twips (1/1440 inch)
    let (mut w, mut h) = match opts.page_size.to_uppercase().as_str() {
        "LEGAL" => (12240, 20160),
        "A4" => (11906, 16838),
        "A3" => (16838, 23811),
        _ => (12240, 15840), // Letter
    };
    let is_landscape = opts.orientation.eq_ignore_ascii_case("Landscape");
    if is_landscape {
        std::mem::swap(&mut w, &mut h);
    }
    let pg_sz = format!(r#"<w:pgSz w:w="{}" w:h="{}" w:orient="{}"/>"#, w, h, if is_landscape { "landscape" } else { "portrait" });
    let top = ((opts.margin_top.unwrap_or(1.0))*1440.0) as i32;
    let bottom = ((opts.margin_bottom.unwrap_or(1.0))*1440.0) as i32;
    let left = ((opts.margin_left.unwrap_or(1.0))*1440.0) as i32;
    let right = ((opts.margin_right.unwrap_or(1.0))*1440.0) as i32;
    let pg_mar = format!(r#"<w:pgMar w:top="{}" w:right="{}" w:bottom="{}" w:left="{}" w:header="720" w:footer="720" w:gutter="0"/>"#, top, right, bottom, left);
    // Update or insert sectPr
    if doc_xml.contains("<w:sectPr") {
        // replace existing pgSz/pgMar if present
        let re_pgsz = regex::Regex::new(r"<w:pgSz[^>]*/?>").unwrap();
        let re_pgmar = regex::Regex::new(r"<w:pgMar[^>]*/?>").unwrap();
        if re_pgsz.is_match(&doc_xml) {
            doc_xml = re_pgsz.replace(&doc_xml, pg_sz.as_str()).to_string();
        } else {
            doc_xml = doc_xml.replacen("<w:sectPr", &format!("<w:sectPr>{}", pg_sz), 1);
        }
        if re_pgmar.is_match(&doc_xml) {
            doc_xml = re_pgmar.replace(&doc_xml, pg_mar.as_str()).to_string();
        } else {
            doc_xml = doc_xml.replacen("<w:sectPr", &format!("<w:sectPr>{}", pg_mar), 1);
        }
    } else {
        // insert new sectPr before </w:body>
        let sect_pr = format!(r#"<w:sectPr>{}{}<w:cols w:space="720"/><w:docGrid w:linePitch="360"/></w:sectPr>"#, pg_sz, pg_mar);
        doc_xml = doc_xml.replace("</w:body>", &format!("{} </w:body>", sect_pr));
    }
    if let Err(e) = write_document_xml(file_path, &doc_xml) {
        return DocumentResult { success: false, message: e, file_path: None, format: None, suggestion: None };
    }
    DocumentResult::ok("Page layout updated successfully".to_string(), Some(file_path.to_string()), Some("docx".to_string()))
}
fn inline_to_run_xml(inline: &MarkdownInline) -> String {
    match inline {
        MarkdownInline::Text(t) => format!(r#"<w:r><w:t xml:space="preserve">{}</w:t></w:r>"#, xml_escape(t)),
        MarkdownInline::Bold(t) => format!(r#"<w:r><w:rPr><w:b/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#, xml_escape(t)),
        MarkdownInline::Italic(t) => format!(r#"<w:r><w:rPr><w:i/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#, xml_escape(t)),
        MarkdownInline::BoldItalic(t) => format!(r#"<w:r><w:rPr><w:b/><w:i/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#, xml_escape(t)),
        MarkdownInline::Code(t) => format!(r#"<w:r><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas"/><w:sz w:val="20"/><w:szCs w:val="20"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#, xml_escape(t)),
        MarkdownInline::Strikethrough(t) => format!(r#"<w:r><w:rPr><w:strike/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#, xml_escape(t)),
        MarkdownInline::Link { text, url: _ } => format!(r#"<w:r><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#, xml_escape(text)),
        MarkdownInline::InlineImage { alt, url: _ } => format!(r#"<w:r><w:t xml:space="preserve">[Image: {}]</w:t></w:r>"#, xml_escape(alt)),
    }
}

fn inlines_to_runs(inlines: &[MarkdownInline]) -> String {
    let mut s = String::new();
    for inline in inlines {
        if let MarkdownInline::InlineImage { .. } = inline {
            continue;
        }
        s.push_str(&inline_to_run_xml(inline));
    }
    s
}

fn heading_from_inlines(level: usize, inlines: &[MarkdownInline]) -> String {
    let lvl = level.clamp(1, 6);
    let runs = if inlines.is_empty() { r#"<w:r><w:t xml:space="preserve"></w:t></w:r>"#.to_string() } else { inlines_to_runs(inlines) };
    format!(r#"    <w:p><w:pPr><w:pStyle w:val="Heading{}"/><w:spacing w:before="240" w:after="120"/></w:pPr>{}</w:p>"#, lvl, runs)
}

fn paragraph_from_inlines(inlines: &[MarkdownInline]) -> String {
    if inlines.is_empty() {
        return r#"    <w:p><w:r><w:t xml:space="preserve"></w:t></w:r></w:p>"#.to_string();
    }
    let runs = inlines_to_runs(inlines);
    if runs.is_empty() {
        return r#"    <w:p><w:r><w:t xml:space="preserve"></w:t></w:r></w:p>"#.to_string();
    }
    format!(r#"    <w:p>{}</w:p>"#, runs)
}

fn bullet_from_inlines(inlines: &[MarkdownInline], numbered: bool) -> String {
    let num_id = if numbered { 2 } else { 1 };
    let runs = inlines_to_runs(inlines);
    format!(r#"    <w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="{}"/></w:numPr><w:ind w:left="720" w:hanging="360"/></w:pPr>{}</w:p>"#, num_id, runs)
}

pub fn add_markdown_content(file_path: &str, markdown: &str, base_image_path: Option<String>) -> DocumentResult {
    if markdown.trim().is_empty() {
        return DocumentResult { success: false, message: "Markdown content cannot be empty".to_string(), file_path: None, format: None, suggestion: None };
    }
    let elements = parse_markdown(markdown);
    let mut frags: Vec<String> = Vec::new();
    let mut image_paths: Vec<String> = Vec::new();
    for el in &elements {
        match el {
            MarkdownElement::Heading { level, text: _, inlines } => {
                if !inlines.is_empty() {
                    frags.push(heading_from_inlines(*level, inlines));
                } else {
                    frags.push(heading_xml("", *level));
                }
            },
            MarkdownElement::Paragraph(inlines) => {
                // collect inline images for later
                for inline in inlines {
                    if let MarkdownInline::InlineImage { alt: _, url } = inline {
                        let img_path = if Path::new(url).is_absolute() { url.clone() } else if let Some(base) = &base_image_path {
                            format!("{}/{}", base.trim_end_matches('/'), url)
                        } else { url.clone() };
                        image_paths.push(img_path);
                    }
                }
                frags.push(paragraph_from_inlines(inlines));
            },
            MarkdownElement::BulletList(items) => {
                for inlines in items { frags.push(bullet_from_inlines(inlines, false)); }
            },
            MarkdownElement::NumberedList(items) => {
                for inlines in items { frags.push(bullet_from_inlines(inlines, true)); }
            },
            MarkdownElement::Table { headers, rows } => {
                let mut data = vec![headers.clone()];
                data.extend(rows.clone());
                frags.push(table_xml(&data));
            },
            MarkdownElement::Image { alt: _, url } => {
                let img_path = if Path::new(url).is_absolute() { url.clone() } else if let Some(base) = &base_image_path {
                    format!("{}/{}", base.trim_end_matches('/'), url)
                } else { url.clone() };
                if Path::new(&img_path).exists() {
                    image_paths.push(img_path);
                    // no placeholder needed, image will be inserted after
                } else {
                    frags.push(paragraph_xml(&format!("[Image missing: {}]", url)));
                }
            },
            MarkdownElement::CodeBlock { code, language: _ } => {
                // code block with monospace
                let escaped = xml_escape(code);
                frags.push(format!(r#"    <w:p><w:pPr><w:shd w:val="clear" w:color="auto" w:fill="F2F2F2"/><w:spacing w:after="120"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas"/><w:sz w:val="20"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#, escaped));
            },
            MarkdownElement::Blockquote(inlines) => {
                let runs = inlines_to_runs(inlines);
                frags.push(format!(r#"    <w:p><w:pPr><w:ind w:left="720"/><w:pBdr><w:left w:val="single" w:sz="4" w:space="4" w:color="CCCCCC"/></w:pBdr></w:pPr>{}</w:p>"#, runs));
            },
            MarkdownElement::HorizontalRule => {
                frags.push(r#"    <w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="6" w:space="1" w:color="auto"/></w:pBdr></w:pPr><w:r><w:t xml:space="preserve"></w:t></w:r></w:p>"#.to_string());
            },
        }
    }
    let res = add_fragments(file_path, frags);
    // handle images after
    for img_path in image_paths {
        if Path::new(&img_path).exists() {
            let _ = add_image(file_path, &img_path, None);
        }
    }
    // also handle block images already collected? they are in image_paths
    let mut final_res = res;
    final_res.message = format!("Markdown content added successfully ({} elements)", elements.len());
    final_res
}
pub fn convert_to_markdown(file_path: &str) -> ContentResult {
    let res = get_document_text(file_path);
    if !res.success { return res; }
    let text = res.content.unwrap_or_default();
    // naive conversion: headings are not distinguished, so just return text as markdown paragraphs
    // Try to preserve headings by detecting style? Our read doesn't capture style; so we just return text separated by blank lines
    let doc_xml = read_document_xml(file_path).unwrap_or_default();
    // attempt to detect headings via w:pStyle
    let mut md = String::new();
    // split by paragraphs with style detection
    let mut idx = 0;
    for part in doc_xml.split("<w:p") {
        if idx==0 { idx+=1; continue; }
        let is_heading = part.contains("w:val=\"Heading");
        let level = if part.contains("Heading1") {1} else if part.contains("Heading2") {2} else if part.contains("Heading3") {3} else {0};
        // extract text
        let re = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
        let mut para = String::new();
        for cap in re.captures_iter(part) { para.push_str(&cap[1]); para.push(' '); }
        para = para.trim().to_string();
        if para.is_empty() { continue; }
        if is_heading && level>0 {
            md.push_str(&format!("{} {}\n\n", "#".repeat(level), para));
        } else if part.contains("<w:tbl") {
            // table already handled? skip
            continue
        } else {
            md.push_str(&format!("{}\n\n", para));
        }
        idx+=1;
    }
    if md.is_empty() { md = text; }
    ContentResult { success: true, content: Some(md.trim().to_string()), error_message: None, total_paragraphs: None, total_pages: None, format: Some("md".to_string()), suggestion: None }
}
pub fn get_rich_content(file_path: &str) -> Vec<DocumentContentItem> {
    let mut items: Vec<DocumentContentItem> = Vec::new();
    let doc_xml = match read_document_xml(file_path) {
        Ok(s) => s,
        Err(_) => return items,
    };
    // parse body: we need to iterate over <w:p> and <w:tbl> and <w:drawing> in order
    // For simplicity, split by <w:p and <w:tbl and detect image after paragraph
    // We'll use regex to find each top-level element
    // Approach: find all <w:p ...>...</w:p> and <w:tbl ...>...</w:tbl> in order via scanning doc_xml body
    let body_start = doc_xml.find("<w:body>").unwrap_or(0);
    let body_end = doc_xml.find("</w:body>").unwrap_or(doc_xml.len());
    let body = &doc_xml[body_start..body_end];
    // use regex for p, tbl
    let re_para = regex::Regex::new(r"(?s)<w:p\b.*?</w:p>").unwrap();
    let re_table = regex::Regex::new(r"(?s)<w:tbl\b.*?</w:tbl>").unwrap();
    // We'll collect matches with positions
    let mut matches: Vec<(usize, String, String)> = Vec::new(); // pos, type, xml
    for m in re_para.find_iter(body) {
        matches.push((m.start(), "para".to_string(), m.as_str().to_string()));
    }
    for m in re_table.find_iter(body) {
        matches.push((m.start(), "table".to_string(), m.as_str().to_string()));
    }
    matches.sort_by_key(|k| k.0);
    let re_text = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
    let re_style = regex::Regex::new(r#"<w:pStyle w:val="([^"]+)""#).unwrap();
    let re_drawing = regex::Regex::new(r"(?s)<w:drawing>.*?</w:drawing>").unwrap();
    // For images, need to extract base64
    // We'll lazily extract images via zip reading
    let has_images = body.contains("<w:drawing>");
    let mut image_entries: Vec<(String, String, i32, i32)> = Vec::new(); // mime, base64, w, h
    if has_images {
        // read zip media
        if let Ok(file) = File::open(file_path) {
            if let Ok(mut za) = ZipArchive::new(file) {
                // collect images
                let mut rels_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                if let Ok(mut rels) = za.by_name("word/_rels/document.xml.rels") {
                    let mut s = String::new();
                    let _ = rels.read_to_string(&mut s);
                    let re_rel = regex::Regex::new(r#"<Relationship Id="([^"]+)"[^>]*Target="([^"]+)"[^>]*Type="[^"]*image[^"]*""#).unwrap();
                    for cap in re_rel.captures_iter(&s) {
                        rels_map.insert(cap[1].to_string(), cap[2].to_string());
                    }
                }
                // For each drawing, find r:embed
                // This will be done per paragraph below
            }
        }
    }
    for (_, typ, xml) in matches {
        if typ=="table" {
            // extract table text
            let re_cell = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
            let mut rows: Vec<String> = Vec::new();
            let re_tr = regex::Regex::new(r"(?s)<w:tr.*?</w:tr>").unwrap();
            for tr in re_tr.find_iter(&xml) {
                let mut cells: Vec<String> = Vec::new();
                let re_tc = regex::Regex::new(r"(?s)<w:tc.*?</w:tc>").unwrap();
                for tc in re_tc.find_iter(tr.as_str()) {
                    let mut txt = String::new();
                    for cap in re_cell.captures_iter(tc.as_str()) { txt.push_str(&cap[1]); txt.push(' '); }
                    cells.push(txt.trim().to_string());
                }
                rows.push(cells.join(" | "));
            }
            let tbl_text = rows.join("\n");
            if !tbl_text.trim().is_empty() {
                items.push(DocumentContentItem { r#type: "table".to_string(), text: Some(tbl_text), level: None, style: None, mime_type: None, image_base64: None, alt_text: None, width_px: None, height_px: None });
            }
        } else {
            // paragraph
            let style = re_style.captures(&xml).and_then(|c| Some(c[1].to_string()));
            let is_heading = style.as_deref().map(|s| s.starts_with("Heading") || s=="Title" || s=="Subtitle").unwrap_or(false);
            let level = if let Some(s) = &style {
                if s.starts_with("Heading") { s[7..].parse::<i32>().unwrap_or(1) } else if s=="Title" {1} else if s=="Subtitle" {2} else {1}
            } else {1};
            let mut para_text = String::new();
            // exclude drawings text
            let mut xml_no_drawing = xml.clone();
            // remove drawings before extracting text
            xml_no_drawing = re_drawing.replace_all(&xml_no_drawing, "").to_string();
            for cap in re_text.captures_iter(&xml_no_drawing) {
                para_text.push_str(&cap[1]);
                para_text.push(' ');
            }
            para_text = para_text.trim().to_string();
            if !para_text.is_empty() {
                items.push(DocumentContentItem { r#type: if is_heading {"heading".to_string()} else {"paragraph".to_string()}, text: Some(para_text), level: if is_heading {Some(level)} else {None}, style: style.clone(), mime_type: None, image_base64: None, alt_text: None, width_px: None, height_px: None });
            }
            // images in this paragraph
            for draw in re_drawing.find_iter(&xml) {
                let draw_str = draw.as_str();
                // find embed
                let re_embed = regex::Regex::new(r#"r:embed="([^"]+)""#).unwrap();
                let re_descr = regex::Regex::new(r#"descr="([^"]*)""#).unwrap();
                let re_cx = regex::Regex::new(r#"<wp:extent cx="([^"]+)" cy="([^"]+)""#).unwrap();
                let embed = re_embed.captures(draw_str).map(|c| c[1].to_string());
                let descr = re_descr.captures(draw_str).map(|c| c[1].to_string()).unwrap_or_default();
                let (cx, cy) = re_cx.captures(draw_str).map(|c| (c[1].parse::<i64>().unwrap_or(914400), c[2].parse::<i64>().unwrap_or(914400))).unwrap_or((914400,914400));
                if let Some(eid) = embed {
                    // resolve image
                    if let Ok(file) = File::open(file_path) {
                        if let Ok(mut za) = ZipArchive::new(file) {
                            let target = {
                                if let Ok(mut rels) = za.by_name("word/_rels/document.xml.rels") {
                                    let mut s = String::new();
                                    let _ = rels.read_to_string(&mut s);
                                    let re = regex::Regex::new(&format!(r#"<Relationship Id="{}"[^>]*Target="([^"]+)""#, regex::escape(&eid))).unwrap();
                                    re.captures(&s).map(|c| c[1].to_string()).unwrap_or_default()
                                } else { String::new() }
                            };
                            if !target.is_empty() {
                                let media_path = if target.starts_with("media/") { format!("word/{}", target) } else { format!("word/{}", target) };
                                let ext = Path::new(&media_path).extension().and_then(|e| e.to_str()).unwrap_or("png").to_string();
                                let mime = match ext.to_lowercase().as_str() { "jpg"|"jpeg" => "image/jpeg", "gif" => "image/gif", "bmp" => "image/bmp", _ => "image/png" };
                                // Need to open again because za moved
                                if let Ok(file2) = File::open(file_path) {
                                    if let Ok(mut za2) = ZipArchive::new(file2) {
                                        if let Ok(mut img) = za2.by_name(&media_path) {
                                            let mut data = Vec::new();
                                            let _ = img.read_to_end(&mut data);
                                            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
                                            let wpx = (cx as f64 *96.0/EMUS_PER_INCH as f64) as i32;
                                            let hpx = (cy as f64 *96.0/EMUS_PER_INCH as f64) as i32;
                                            items.push(DocumentContentItem { r#type: "image".to_string(), text: None, level: None, style: None, mime_type: Some(mime.to_string()), image_base64: Some(b64), alt_text: Some(descr), width_px: Some(wpx), height_px: Some(hpx) });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    items
}

pub fn extract_images(file_path: &str) -> Vec<ImageExtractionResult> {
    let rich = get_rich_content(file_path);
    // For word, ImageExtractionResult expects ContextBefore/After etc.
    // We will reconstruct via simple approach: use doc xml paragraphs order
    let doc_text = get_document_text(file_path).content.unwrap_or_default();
    let paras: Vec<String> = doc_text.split('\n').map(|s| s.to_string()).collect();
    let mut results: Vec<ImageExtractionResult> = Vec::new();
    let mut idx = 0;
    // Need to get images via zip directly with context
    let file = match File::open(file_path) { Ok(f)=>f, Err(_)=> return results };
    let mut archive = match ZipArchive::new(file) { Ok(a)=>a, Err(_)=> return results };
    // collect paragraphs for context: need document.xml paragraphs list
    let doc_xml = read_document_xml(file_path).unwrap_or_default();
    let re_para = regex::Regex::new(r"(?s)<w:p\b.*?</w:p>").unwrap();
    let re_text = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
    let re_drawing = regex::Regex::new(r"(?s)<w:drawing>.*?</w:drawing>").unwrap();
    let mut para_texts: Vec<String> = Vec::new();
    let mut drawing_paras: Vec<Option<String>> = Vec::new(); // alt?
    for m in re_para.find_iter(&doc_xml) {
        let xml = m.as_str();
        let mut txt = String::new();
        let no_draw = re_drawing.replace_all(xml, "");
        for cap in re_text.captures_iter(&no_draw) { txt.push_str(&cap[1]); txt.push(' '); }
        para_texts.push(txt.trim().to_string());
        if xml.contains("<w:drawing>") { drawing_paras.push(Some(xml.to_string())); } else { drawing_paras.push(None); }
    }
    // find each drawing and create result
    // Need to open archive again for media
    for (i, maybe_draw_xml) in drawing_paras.iter().enumerate() {
        if let Some(draw_xml) = maybe_draw_xml {
            for draw in re_drawing.find_iter(draw_xml) {
                let draw_str = draw.as_str();
                let re_embed = regex::Regex::new(r#"r:embed="([^"]+)""#).unwrap();
                let re_descr = regex::Regex::new(r#"descr="([^"]*)""#).unwrap();
                let re_cx = regex::Regex::new(r#"<wp:extent cx="([^"]+)" cy="([^"]+)""#).unwrap();
                let embed = re_embed.captures(draw_str).map(|c| c[1].to_string());
                let descr = re_descr.captures(draw_str).map(|c| c[1].to_string()).unwrap_or_default();
                let (cx, cy) = re_cx.captures(draw_str).map(|c| (c[1].parse::<i64>().unwrap_or(914400), c[2].parse::<i64>().unwrap_or(914400))).unwrap_or((914400,914400));
                if let Some(eid) = embed {
                    // find image
                    // reopen archive for each
                    let target = {
                        let f = File::open(file_path).unwrap();
                        let mut za = ZipArchive::new(f).unwrap();
                        let mut s = String::new();
                        if let Ok(mut rels) = za.by_name("word/_rels/document.xml.rels") {
                            let _ = rels.read_to_string(&mut s);
                        }
                        let re = regex::Regex::new(&format!(r#"<Relationship Id="{}"[^>]*Target="([^"]+)""#, regex::escape(&eid))).unwrap();
                        re.captures(&s).map(|c| c[1].to_string()).unwrap_or_default()
                    };
                    if target.is_empty() { continue; }
                    let media_path = if target.starts_with("media/") { format!("word/{}", target) } else { format!("word/{}", target) };
                    let f = File::open(file_path).unwrap();
                    let mut za = ZipArchive::new(f).unwrap();
                    let img_data = {
                        let mut tmp: Vec<u8> = Vec::new();
                        let read_ok = match za.by_name(&media_path) {
                            Ok(mut img) => { let _ = img.read_to_end(&mut tmp); true },
                            Err(_) => false,
                        };
                        if !read_ok { continue; }
                        tmp
                    };
                    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &img_data);
                    let ext = Path::new(&media_path).extension().and_then(|e| e.to_str()).unwrap_or("png");
                    let mime = match ext.to_lowercase().as_str() { "jpg"|"jpeg" => "image/jpeg", "gif" => "image/gif", "bmp" => "image/bmp", _ => "image/png" };
                    let wpx = (cx as f64 *96.0/EMUS_PER_INCH as f64) as i32;
                    let hpx = (cy as f64 *96.0/EMUS_PER_INCH as f64) as i32;
                    let before = if i>0 { para_texts[i-1].clone() } else { String::new() };
                    let after = if i+1 < para_texts.len() { para_texts[i+1].clone() } else { String::new() };
                    results.push(ImageExtractionResult { index: idx, mime_type: mime.to_string(), image_base64: b64, alt_text: descr, context_before: before, context_after: after, width_px: Some(wpx), height_px: Some(hpx), page_or_slide_number: None });
                    idx+=1;
                }
            }
        }
    }
    results
}
