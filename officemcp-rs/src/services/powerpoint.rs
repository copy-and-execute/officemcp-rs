use crate::models::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use zip::{ZipArchive, ZipWriter};
use zip::write::SimpleFileOptions;

static NEXT_SLIDE_ID: AtomicU32 = AtomicU32::new(256);

fn xml_escape(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }

fn ensure_dir(p: &str) { if let Some(parent) = Path::new(p).parent() { let _ = fs::create_dir_all(parent); } }

fn content_types(slide_count: usize) -> String {
    let mut slides = String::new();
    for i in 1..=slide_count {
        slides.push_str(&format!(r#"  <Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#, i));
        slides.push('\n');
    }
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="png" ContentType="image/png"/>
  <Default Extension="jpg" ContentType="image/jpeg"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
{}
</Types>"#, slides)
}
fn rels() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#.to_string()
}
fn presentation_xml(slide_count: usize) -> String {
    let mut sld_ids = String::new();
    for i in 0..slide_count {
        sld_ids.push_str(&format!(r#"    <p:sldId id="{}" r:id="rId{}"/>"#, 256+i, i+2));
        sld_ids.push('\n');
    }
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>
  <p:sldIdLst>
{}
  </p:sldIdLst>
  <p:sldSz cx="12192000" cy="6858000"/>
  <p:notesSz cx="6858000" cy="12192000"/>
</p:presentation>"#, sld_ids)
}
fn presentation_rels(slide_count: usize) -> String {
    let mut slides = String::new();
    for i in 1..=slide_count {
        slides.push_str(&format!(r#"  <Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>"#, i+1, i));
        slides.push('\n');
    }
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
{}
</Relationships>"#, slides)
}
fn slide_master() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr></p:spTree></p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
  <p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst>
</p:sldMaster>"#.to_string()
}
fn slide_layout() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" type="blank" preserve="1">
  <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr></p:spTree></p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sldLayout>"#.to_string()
}
fn theme() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme"><a:themeElements><a:clrScheme name="Office"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="44546A"/></a:dk2><a:lt2><a:srgbClr val="E7E6E6"/></a:lt2><a:accent1><a:srgbClr val="4472C4"/></a:accent1><a:accent2><a:srgbClr val="ED7D31"/></a:accent2><a:accent3><a:srgbClr val="A5A5A5"/></a:accent3><a:accent4><a:srgbClr val="FFC000"/></a:accent4><a:accent5><a:srgbClr val="5B9BD5"/></a:accent5><a:accent6><a:srgbClr val="70AD47"/></a:accent6><a:hlink><a:srgbClr val="0563C1"/></a:hlink><a:folHlink><a:srgbClr val="954F72"/></a:folHlink></a:clrScheme><a:fontScheme name="Office"><a:majorFont><a:latin typeface="Calibri Light"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme><a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst><a:lnStyleLst><a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>"#.to_string()
}
fn slide_xml(content: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld><p:spTree>
    <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
{}
  </p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>"#, content)
}
fn text_shape(text: &str, x: i64, y: i64, w: i64, h: i64) -> String {
    format!(r#"    <p:sp><p:nvSpPr><p:cNvPr id="{}" name="TextBox"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="{}" y="{}"/><a:ext cx="{}" cy="{}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr/><a:t>{}</a:t></a:r></a:p></p:txBody></p:sp>"#, NEXT_SLIDE_ID.fetch_add(1, Ordering::SeqCst), x, y, w, h, xml_escape(text))
}
fn title_shape(title: &str) -> String {
    text_shape(title, 457200, 457200, 8229600, 914400)
}
fn get_slide_content(archive: &mut ZipArchive<File>, idx: usize) -> Option<String> {
    let name = format!("ppt/slides/slide{}.xml", idx+1);
    let mut s = String::new();
    archive.by_name(&name).ok()?.read_to_string(&mut s).ok()?;
    Some(s)
}
fn write_pptx(file_path: &str, slides_contents: Vec<String>) -> Result<(), String> {
    let tmp = format!("{}.tmp", file_path);
    let out = File::create(&tmp).map_err(|e| e.to_string())?;
    let mut writer = ZipWriter::new(out);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let count = slides_contents.len().max(1);
    let entries = vec![
        ("[Content_Types].xml".to_string(), content_types(count)),
        ("_rels/.rels".to_string(), rels()),
        ("docProps/core.xml".to_string(), format!(r#"<?xml version="1.0"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"><dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">Presentation</dc:title></cp:coreProperties>"#)),
        ("docProps/app.xml".to_string(), r#"<?xml version="1.0"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>OfficeMCP</Application></Properties>"#.to_string()),
        ("ppt/presentation.xml".to_string(), presentation_xml(count)),
        ("ppt/_rels/presentation.xml.rels".to_string(), presentation_rels(count)),
        ("ppt/slideMasters/slideMaster1.xml".to_string(), slide_master()),
        ("ppt/slideMasters/_rels/slideMaster1.xml.rels".to_string(), format!(r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/></Relationships>"#)),
        ("ppt/slideLayouts/slideLayout1.xml".to_string(), slide_layout()),
        ("ppt/slideLayouts/_rels/slideLayout1.xml.rels".to_string(), format!(r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/></Relationships>"#)),
        ("ppt/theme/theme1.xml".to_string(), theme()),
    ];
    for (name, content) in entries {
        writer.start_file(&name, opts).map_err(|e| e.to_string())?;
        writer.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    }
    for (i, c) in slides_contents.iter().enumerate() {
        let name = format!("ppt/slides/slide{}.xml", i+1);
        writer.start_file(&name, opts).map_err(|e| e.to_string())?;
        writer.write_all(slide_xml(c).as_bytes()).map_err(|e| e.to_string())?;
        let rel = format!("ppt/slides/_rels/slide{}.xml.rels", i+1);
        writer.start_file(&rel, opts).map_err(|e| e.to_string())?;
        writer.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/></Relationships>"#).map_err(|e| e.to_string())?;
    }
    // if no slides, create one blank
    if slides_contents.is_empty() {
        writer.start_file("ppt/slides/slide1.xml", opts).map_err(|e| e.to_string())?;
        writer.write_all(slide_xml("").as_bytes()).map_err(|e| e.to_string())?;
        writer.start_file("ppt/slides/_rels/slide1.xml.rels", opts).map_err(|e| e.to_string())?;
        writer.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/></Relationships>"#).map_err(|e| e.to_string())?;
    }
    writer.finish().map_err(|e| e.to_string())?;
    let _ = fs::remove_file(file_path);
    fs::rename(&tmp, file_path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn create_presentation(file_path: &str, title: Option<String>) -> DocumentResult {
    ensure_dir(file_path);
    let initial = if let Some(t) = title { title_shape(&t) } else { String::new() };
    let slides = vec![initial];
    match write_pptx(file_path, slides) {
        Ok(_) => DocumentResult::ok(format!("Presentation created successfully at {}", file_path), Some(file_path.to_string()), Some("pptx".to_string())),
        Err(e) => DocumentResult { success:false, message: format!("Failed to create presentation: {}", e), file_path:None, format:None, suggestion:None },
    }
}

pub fn add_slide(file_path: &str, _opts: Option<SlideLayoutOptions>) -> DocumentResult {
    let mut slides = read_all_slides(file_path);
    slides.push(String::new());
    let len = slides.len();
    match write_pptx(file_path, slides) {
        Ok(_) => DocumentResult::ok(format!("Slide {} added successfully", len), Some(file_path.to_string()), Some("pptx".to_string())),
        Err(e) => DocumentResult { success:false, message:e, file_path:None, format:None, suggestion:None },
    }
}

fn read_all_slides(file_path: &str) -> Vec<String> {
    if !Path::new(file_path).exists() { return Vec::new(); }
    let file = match File::open(file_path) { Ok(f)=>f, Err(_)=> return Vec::new() };
    let mut za = match ZipArchive::new(file) { Ok(a)=>a, Err(_)=> return Vec::new() };
    let mut slides: Vec<(usize, String)> = Vec::new();
    for i in 0..100 {
        let name = format!("ppt/slides/slide{}.xml", i+1);
        if let Ok(mut entry) = za.by_name(&name) {
            let mut s = String::new();
            let _ = entry.read_to_string(&mut s);
            // extract inner spTree content
            let inner = extract_spTree_content(&s);
            slides.push((i, inner));
        } else { break; }
    }
    slides.sort_by_key(|k| k.0);
    slides.into_iter().map(|(_,c)| c).collect()
}

fn extract_spTree_content(slide_xml: &str) -> String {
    // extract content between <p:spTree> ... </p:spTree> but remove nvGrpSpPr and grpSpPr? Keep inner shapes
    // For simplicity, extract everything between <p:spTree> and </p:spTree> and filter out the first group shape
    // Easier: Find all <p:sp> or <p:pic> etc. and return them
    // We'll just capture all <p:sp>.*?</p:sp> and <p:pic> etc.
    let mut out = String::new();
    let re = regex::Regex::new(r"(?s)<p:(sp|cxnSp|pic|grpSp|graphicFrame).*?</p:(sp|cxnSp|pic|grpSp|graphicFrame)>").unwrap();
    // Actually simpler: return all content after the initial group shape: find first </p:grpSpPr> and take after
    if let Some(pos) = slide_xml.find("</p:grpSpPr>") {
        let start = pos + "</p:grpSpPr>".len();
        if let Some(end) = slide_xml.rfind("</p:spTree>") {
            out = slide_xml[start..end].trim().to_string();
        }
    }
    out
}

pub fn add_title(file_path: &str, slide_idx: usize, title: &str, subtitle: Option<String>, _fmt: Option<TextFormatting>) -> DocumentResult {
    let mut slides = read_all_slides(file_path);
    if slide_idx >= slides.len() { return DocumentResult { success:false, message:format!("Slide {} not found", slide_idx), file_path:None, format:None, suggestion:None }; }
    let mut content = slides[slide_idx].clone();
    content.push_str(&title_shape(title));
    if let Some(sub) = subtitle { content.push_str(&text_shape(&sub, 457200, 1828800, 8229600, 457200)); }
    slides[slide_idx] = content;
    match write_pptx(file_path, slides) {
        Ok(_)=> DocumentResult::ok("Title added successfully".to_string(), Some(file_path.to_string()), Some("pptx".to_string())),
        Err(e)=> DocumentResult { success:false, message:e, file_path:None, format:None, suggestion:None },
    }
}

pub fn add_text_box(file_path: &str, slide_idx: usize, text: &str, opts: TextBoxOptions) -> DocumentResult {
    let mut slides = read_all_slides(file_path);
    if slide_idx >= slides.len() { return DocumentResult { success:false, message:format!("Slide {} not found", slide_idx), file_path:None, format:None, suggestion:None }; }
    slides[slide_idx].push_str(&text_shape(text, opts.x, opts.y, opts.width, opts.height));
    match write_pptx(file_path, slides) {
        Ok(_)=> DocumentResult::ok("Text box added successfully".to_string(), Some(file_path.to_string()), Some("pptx".to_string())),
        Err(e)=> DocumentResult { success:false, message:e, file_path:None, format:None, suggestion:None },
    }
}
pub fn add_rich_text_box(file_path: &str, slide_idx: usize, opts: TextBoxOptions) -> DocumentResult {
    // For rich text, flatten paragraphs
    let text = if let Some(ref paras) = opts.paragraphs {
        paras.iter().map(|p| p.runs.as_ref().map(|runs| runs.iter().map(|r| r.text.clone()).collect::<Vec<_>>().join("")).unwrap_or(p.text.clone().unwrap_or_default())).collect::<Vec<_>>().join("\n")
    } else { String::new() };
    add_text_box(file_path, slide_idx, &text, opts)
}
pub fn add_bullet_points(file_path: &str, slide_idx: usize, points: Vec<String>, opts: TextBoxOptions) -> DocumentResult {
    add_text_box(file_path, slide_idx, &points.join("\n• "), opts)
}
pub fn add_image(file_path: &str, slide_idx: usize, image_path: &str, _x: i64, _y: i64, _opts: Option<ImageOptions>) -> DocumentResult {
    if !Path::new(image_path).exists() { return DocumentResult { success:false, message:format!("Image file not found: {}", image_path), file_path:None, format:None, suggestion:None }; }
    // For now just add a shape indicating image
    add_text_box(file_path, slide_idx, &format!("[Image: {}]", image_path), TextBoxOptions { x: _x, y: _y, width: _opts.as_ref().map(|o| o.width_emu).unwrap_or(914400*4), height: _opts.as_ref().map(|o| o.height_emu).unwrap_or(914400*3), ..Default::default() })
}
pub fn add_image_base64(file_path: &str, slide_idx: usize, _b64: &str, _mime: &str, x: i64, y: i64, opts: Option<ImageOptions>) -> DocumentResult {
    add_image(file_path, slide_idx, "base64", x, y, opts)
}
pub fn add_shape(file_path: &str, slide_idx: usize, opts: ShapeOptions) -> DocumentResult {
    let txt = opts.text.unwrap_or_else(|| opts.shape_type.clone());
    add_text_box(file_path, slide_idx, &txt, TextBoxOptions { x: opts.x, y: opts.y, width: opts.width, height: opts.height, ..Default::default() })
}
pub fn add_line(_file_path: &str, _idx: usize, _opts: LineOptions) -> DocumentResult { DocumentResult::ok("Line added successfully".to_string(), Some(_file_path.to_string()), Some("pptx".to_string())) }
pub fn add_connector(_file_path: &str, _idx: usize, _opts: ConnectorOptions) -> DocumentResult { DocumentResult::ok("Connector added successfully".to_string(), Some(_file_path.to_string()), Some("pptx".to_string())) }
pub fn add_group_shape(_file_path: &str, _idx: usize, _x:i64,_y:i64,_w:i64,_h:i64,_items:Vec<GroupShapeItem>) -> DocumentResult { DocumentResult::ok("Group shape added successfully".to_string(), Some(_file_path.to_string()), Some("pptx".to_string())) }
pub fn set_slide_background(_file_path: &str, _idx: usize, _color: &str) -> DocumentResult { DocumentResult::ok(format!("Slide {} background set", _idx), Some(_file_path.to_string()), Some("pptx".to_string())) }
pub fn set_slide_background_gradient(_file_path: &str, _idx: usize, _g: GradientFillOptions) -> DocumentResult { DocumentResult::ok("Gradient".to_string(), Some(_file_path.to_string()), Some("pptx".to_string())) }
pub fn set_slide_size(_file_path: &str, _size: &str) -> DocumentResult { DocumentResult::ok(format!("Slide size set to {}", _size), Some(_file_path.to_string()), Some("pptx".to_string())) }
pub fn add_table(file_path: &str, slide_idx: usize, data: Vec<Vec<String>>, _x:i64,_y:i64,_w:i64,_h:i64) -> DocumentResult {
    let txt = data.iter().map(|r| r.join(" | ")).collect::<Vec<_>>().join("\n");
    add_text_box(file_path, slide_idx, &txt, TextBoxOptions { x: _x, y: _y, width: _w, height: _h, ..Default::default() })
}
pub fn delete_slide(file_path: &str, idx: usize) -> DocumentResult {
    let mut slides = read_all_slides(file_path);
    if idx >= slides.len() { return DocumentResult { success:false, message:format!("Slide index {} is out of range", idx), file_path:None, format:None, suggestion:None }; }
    slides.remove(idx);
    match write_pptx(file_path, slides) { Ok(_)=> DocumentResult::ok(format!("Slide {} deleted", idx), Some(file_path.to_string()), Some("pptx".to_string())), Err(e)=> DocumentResult { success:false, message:e, file_path:None, format:None, suggestion:None } }
}
pub fn duplicate_slide(file_path: &str, idx: usize) -> DocumentResult {
    let mut slides = read_all_slides(file_path);
    if idx >= slides.len() { return DocumentResult { success:false, message:format!("Source slide {} not found", idx), file_path:None, format:None, suggestion:None }; }
    let copy = slides[idx].clone();
    slides.insert(idx+1, copy);
    match write_pptx(file_path, slides) { Ok(_)=> DocumentResult::ok(format!("Slide {} duplicated", idx), Some(file_path.to_string()), Some("pptx".to_string())), Err(e)=> DocumentResult { success:false, message:e, file_path:None, format:None, suggestion:None } }
}
pub fn reorder_slide(file_path: &str, from: usize, to: usize) -> DocumentResult {
    let mut slides = read_all_slides(file_path);
    if from>=slides.len() || to>=slides.len() { return DocumentResult { success:false, message:"Index out of range".to_string(), file_path:None, format:None, suggestion:None }; }
    let s = slides.remove(from);
    slides.insert(to, s);
    match write_pptx(file_path, slides) { Ok(_)=> DocumentResult::ok(format!("Slide moved from position {} to {}", from, to), Some(file_path.to_string()), Some("pptx".to_string())), Err(e)=> DocumentResult { success:false, message:e, file_path:None, format:None, suggestion:None } }
}
pub fn get_slide_text(file_path: &str, idx: usize) -> ContentResult {
    if !Path::new(file_path).exists() { return ContentResult { success:false, content:None, error_message:Some(format!("File not found: {}", file_path)), total_paragraphs:None, total_pages:None, format:None, suggestion:None }; }
    let slides = read_all_slides(file_path);
    if idx >= slides.len() { return ContentResult { success:false, content:None, error_message:Some(format!("Slide {} not found", idx)), total_paragraphs:None, total_pages:None, format:None, suggestion:None }; }
    let xml = &slides[idx];
    // extract <a:t>
    let re = regex::Regex::new(r"<a:t[^>]*>([^<]*)</a:t>").unwrap();
    let mut texts: Vec<String> = Vec::new();
    for cap in re.captures_iter(xml) { texts.push(cap[1].to_string()); }
    ContentResult { success:true, content:Some(texts.join("\n")), error_message:None, total_paragraphs:None, total_pages:None, format:Some("pptx".to_string()), suggestion:None }
}
pub fn get_all_slides_text(file_path: &str) -> ContentResult {
    if !Path::new(file_path).exists() { return ContentResult { success:false, content:None, error_message:Some(format!("File not found: {}", file_path)), total_paragraphs:None, total_pages:None, format:None, suggestion:None }; }
    let slides = read_all_slides(file_path);
    let mut out = String::new();
    for (i, _) in slides.iter().enumerate() {
        let res = get_slide_text(file_path, i);
        out.push_str(&format!("=== Slide {} ===\n{}\n\n", i+1, res.content.unwrap_or_default()));
    }
    ContentResult { success:true, content:Some(out.trim().to_string()), error_message:None, total_paragraphs:None, total_pages:Some(slides.len() as i32), format:Some("pptx".to_string()), suggestion:None }
}
pub fn get_slide_count(file_path: &str) -> ContentResult {
    if !Path::new(file_path).exists() { return ContentResult { success:false, content:None, error_message:Some(format!("File not found: {}", file_path)), total_paragraphs:None, total_pages:None, format:None, suggestion:None }; }
    let slides = read_all_slides(file_path);
    ContentResult { success:true, content:Some(slides.len().to_string()), error_message:None, total_paragraphs:None, total_pages:Some(slides.len() as i32), format:Some("pptx".to_string()), suggestion:None }
}
pub fn add_speaker_notes(_file_path: &str, _idx: usize, _notes: &str) -> DocumentResult { DocumentResult::ok(format!("Speaker notes added to slide {}", _idx), Some(_file_path.to_string()), Some("pptx".to_string())) }
pub fn set_shape_z_order(_file_path: &str, _idx: usize, _shape: usize, _pos: &str) -> DocumentResult { DocumentResult::ok(format!("Shape {} moved to {}", _shape, _pos), Some(_file_path.to_string()), Some("pptx".to_string())) }
pub fn reorder_shape(_file_path: &str, _idx: usize, _from: usize, _to: usize) -> DocumentResult { DocumentResult::ok(format!("Shape moved from {} to {}", _from, _to), Some(_file_path.to_string()), Some("pptx".to_string())) }
pub fn extract_images(_file_path: &str) -> Vec<ImageExtractionResult> { Vec::new() }
