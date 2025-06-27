use pulldown_cmark::{Parser, Event, Tag, HeadingLevel, Options, TagEnd};
use genpdf::{elements, style, Element, Document, Alignment};
use std::fs;

#[derive(Default)]
struct DocumentMetadata {
    title: Option<String>,
    subtitle: Option<String>,
    author: Option<String>,
    date: Option<String>,
    version: Option<String>,
}

fn parse_yaml_frontmatter(content: &str) -> (DocumentMetadata, String) {
    let mut metadata = DocumentMetadata::default();
    
    if content.starts_with("---\n") {
        if let Some(end_pos) = content[4..].find("\n---\n") {
            let yaml_content = &content[4..end_pos + 4];
            let remaining_content = &content[end_pos + 8..];
            
            // Simple YAML parsing for our specific fields
            for line in yaml_content.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"');
                    
                    match key {
                        "title" => metadata.title = Some(value.to_string()),
                        "subtitle" => metadata.subtitle = Some(value.to_string()),
                        "author" => metadata.author = Some(value.to_string()),
                        "date" => metadata.date = Some(value.to_string()),
                        "version" => metadata.version = Some(value.to_string()),
                        _ => {}
                    }
                }
            }
            
            return (metadata, remaining_content.to_string());
        }
    }
    
    (metadata, content.to_string())
}

fn create_title_page(metadata: &DocumentMetadata) -> elements::LinearLayout {
    let mut title_page = elements::LinearLayout::vertical();
    
    // Add some top spacing
    title_page.push(elements::Break::new(8));
    
    // Title
    if let Some(title) = &metadata.title {
        let title_style = style::Style::new()
            .bold()
            .with_font_size(24);
        
        title_page.push(
            elements::Paragraph::new(title)
                .aligned(Alignment::Center)
                .styled(title_style)
        );
        title_page.push(elements::Break::new(2));
    }
    
    // Subtitle
    if let Some(subtitle) = &metadata.subtitle {
        let subtitle_style = style::Style::new()
            .with_font_size(16);
        
        title_page.push(
            elements::Paragraph::new(subtitle)
                .aligned(Alignment::Center)
                .styled(subtitle_style)
        );
        title_page.push(elements::Break::new(4));
    }
    
    // Author
    if let Some(author) = &metadata.author {
        let author_style = style::Style::new()
            .with_font_size(14);
        
        title_page.push(
            elements::Paragraph::new(author)
                .aligned(Alignment::Center)
                .styled(author_style)
        );
        title_page.push(elements::Break::new(2));
    }
    
    // Date and Version
    if let Some(date) = &metadata.date {
        let date_style = style::Style::new()
            .with_font_size(12);
        
        let mut date_text = date.clone();
        if let Some(version) = &metadata.version {
            date_text.push_str(&format!(" - Version {}", version));
        }
        
        title_page.push(
            elements::Paragraph::new(date_text)
                .aligned(Alignment::Center)
                .styled(date_style)
        );
    }
    
    title_page
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = fs::read_to_string("../SWTCH-Whitepaper.md")
        .expect("Failed to read SWTCH-Whitepaper.md file");
    
    // Parse YAML frontmatter and get remaining content
    let (metadata, content) = parse_yaml_frontmatter(&markdown);
    
    let parser = Parser::new_ext(&content, Options::all());

    // Create document with single Times New Roman font file
    let font_data = std::fs::read("../fonts/times.ttf")?;
    let font = genpdf::fonts::FontData::new(font_data, None)?;
    let font_family = genpdf::fonts::FontFamily {
        regular: font.clone(),
        bold: font.clone(),
        italic: font.clone(), 
        bold_italic: font.clone(),
    };
    let mut doc = Document::new(font_family);
    
    doc.set_title(metadata.title.as_deref().unwrap_or("SWTCH WhitePaper"));
    doc.set_minimal_conformance();
    
    // Set page size to Letter and margins (0.375 inch = 27 points)
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(genpdf::Margins::trbl(27, 27, 27, 27));
    doc.set_page_decorator(decorator);
    
    // Set paper size explicitly
    doc.set_paper_size(genpdf::PaperSize::Letter);

    // Create title page
    let title_page = create_title_page(&metadata);
    doc.push(title_page);
    doc.push(elements::PageBreak::new());

    let mut section = elements::LinearLayout::vertical();
    let mut current_paragraph: Option<String> = None;
    let mut in_code_block = false;
    let mut code_content = String::new();
    let mut list_items: Vec<String> = Vec::new();
    let mut in_list = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // Finish any pending paragraph
                if let Some(text) = current_paragraph.take() {
                    if !text.trim().is_empty() {
                        section.push(elements::Paragraph::new(text.trim())
                            .aligned(Alignment::Left));
                    }
                }
                
                current_paragraph = Some(String::new());
            },
            
            Event::End(TagEnd::Heading(level)) => {
                if let Some(heading_text) = current_paragraph.take() {
                    let heading_text = heading_text.trim();
                    
                    let font_size = match level {
                        HeadingLevel::H1 => 18,
                        HeadingLevel::H2 => 16,
                        HeadingLevel::H3 => 14,
                        HeadingLevel::H4 => 12,
                        HeadingLevel::H5 => 11,
                        HeadingLevel::H6 => 10,
                    };
                    
                    let heading_style = style::Style::new()
                        .bold()
                        .with_font_size(font_size);
                    
                    section.push(elements::Paragraph::new(heading_text)
                        .aligned(Alignment::Left)
                        .styled(heading_style));
                    
                    // Add spacing after headings
                    section.push(elements::Break::new(1));
                }
            },
            
            Event::Start(Tag::CodeBlock(_)) => {
                // Finish any pending paragraph
                if let Some(text) = current_paragraph.take() {
                    if !text.trim().is_empty() {
                        section.push(elements::Paragraph::new(text.trim())
                            .aligned(Alignment::Left));
                    }
                }
                
                in_code_block = true;
                code_content.clear();
            },
            
            Event::End(TagEnd::CodeBlock) => {
                if in_code_block {
                    let code_style = style::Style::new()
                        .with_font_size(9);
                    
                    section.push(elements::Paragraph::new(&code_content)
                        .aligned(Alignment::Left)
                        .styled(code_style));
                    section.push(elements::Break::new(1));
                    
                    in_code_block = false;
                    code_content.clear();
                }
            },
            
            Event::Start(Tag::List(_)) => {
                // Finish any pending paragraph
                if let Some(text) = current_paragraph.take() {
                    if !text.trim().is_empty() {
                        section.push(elements::Paragraph::new(text.trim())
                            .aligned(Alignment::Left));
                    }
                }
                
                in_list = true;
                list_items.clear();
            },
            
            Event::End(TagEnd::List(_)) => {
                if in_list {
                    for item in &list_items {
                        section.push(elements::Paragraph::new(format!("• {}", item))
                            .aligned(Alignment::Left));
                    }
                    section.push(elements::Break::new(1));
                    in_list = false;
                    list_items.clear();
                }
            },
            
            Event::Start(Tag::Item) => {
                current_paragraph = Some(String::new());
            },
            
            Event::End(TagEnd::Item) => {
                if let Some(item_text) = current_paragraph.take() {
                    list_items.push(item_text.trim().to_string());
                }
            },
            
            Event::Text(text) => {
                if in_code_block {
                    code_content.push_str(&text);
                } else {
                    if current_paragraph.is_none() {
                        current_paragraph = Some(String::new());
                    }
                    if let Some(ref mut para) = current_paragraph {
                        para.push_str(&text);
                    }
                }
            },
            
            Event::Code(text) => {
                if let Some(ref mut para) = current_paragraph {
                    para.push_str(&format!("`{}`", text));
                }
            },
            
            Event::SoftBreak => {
                if let Some(ref mut para) = current_paragraph {
                    para.push(' ');
                }
            },
            
            Event::HardBreak => {
                if let Some(text) = current_paragraph.take() {
                    if !text.trim().is_empty() {
                        section.push(elements::Paragraph::new(text.trim())
                            .aligned(Alignment::Left));
                    }
                }
            },
            
            Event::Start(Tag::Paragraph) => {
                current_paragraph = Some(String::new());
            },
            
            Event::End(TagEnd::Paragraph) => {
                if let Some(text) = current_paragraph.take() {
                    let text = text.trim();
                    if !text.is_empty() {
                        // Handle LaTeX commands
                        if text == "\\newpage" {
                            section.push(elements::PageBreak::new());
                        } else {
                            section.push(elements::Paragraph::new(text)
                                .aligned(Alignment::Left));
                            section.push(elements::Break::new(1));
                        }
                    }
                }
            },
            
            Event::Rule => {
                section.push(elements::Break::new(2));
            },
            
            Event::Start(Tag::Strong) | Event::End(TagEnd::Strong) => {
                // Bold text - would need more complex handling for proper formatting
            },
            
            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {
                // Italic text - would need more complex handling for proper formatting
            },
            
            _ => {
                // Handle other events as needed
            }
        }
    }
    
    // Handle any remaining paragraph
    if let Some(text) = current_paragraph {
        if !text.trim().is_empty() {
            section.push(elements::Paragraph::new(text.trim())
                .aligned(Alignment::Left));
        }
    }

    // Add the section with padding to force full width usage
    doc.push(section.padded(genpdf::Margins::vh(0, 0)));
    
    // Render to PDF
    doc.render_to_file("../SWTCH-Whitepaper.pdf")?;
    println!("PDF generated successfully: SWTCH-Whitepaper.pdf");
    println!("Title: {}", metadata.title.as_deref().unwrap_or("N/A"));
    println!("Author: {}", metadata.author.as_deref().unwrap_or("N/A"));
    
    Ok(())
}
