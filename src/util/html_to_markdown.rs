use html5ever::{QualName, parse_fragment};
use html5ever::rcdom::{NodeData, RcDom, Handle};
use html5ever::tendril::TendrilSink;

error_chain! {
    errors {
        HtmlParse(errors: Vec<String>) {
            description("There was a problem parsing the HTML")
        }
    }
}

pub fn convert(html: &str) -> Result<String> {
    let mut converter = HtmlVisitor::new();

    let context = QualName::new(None, ns!(html), local_name!("body"));
    let dom = parse_fragment(RcDom::default(), Default::default(), context, vec![]).one(html);

    converter.visit(dom.document);

    if !dom.errors.is_empty() {
        bail!(ErrorKind::HtmlParse(dom.errors.into_iter().map(|e| e.into_owned()).collect()));
    }

    Ok(converter.markdown())
}

struct HtmlVisitor {
    markdown: String,
}

impl HtmlVisitor {
    fn new() -> HtmlVisitor {
        HtmlVisitor {
            markdown: String::new(),
        }
    }

    fn markdown(self) -> String {
        self.markdown
    }

    fn visit(&mut self, node: Handle) {
        match node.data {
            NodeData::Document => self.visit_children(node.clone()),
            NodeData::Text { .. } => self.visit_text(node.clone()),
            NodeData::Element { ref name, .. } => {
                match name.local {
                    local_name!("a") => self.visit_link(node.clone()),
                    local_name!("p") => self.visit_paragraph(node.clone()),
                    local_name!("code") => self.visit_code(node.clone()),
                    local_name!("i") => self.visit_italic(node.clone()),
                    _ => self.visit_children(node.clone()),
                }
            }
            NodeData::Doctype { .. } |
            NodeData::Comment { .. } |
            NodeData::ProcessingInstruction { .. } => {}
        }
    }

    fn visit_children(&mut self, node: Handle) {
        for child in node.children.borrow().iter() {
            self.visit(child.clone());
        }
    }

    fn visit_link(&mut self, node: Handle) {
        if let NodeData::Element { ref attrs, .. } = node.data {
            for attr in attrs.borrow().iter() {
                if &attr.name.local == "href" {
                    self.markdown.push_str(&*attr.value);
                }
            }
        }
    }

    fn visit_text(&mut self, node: Handle) {
        if let NodeData::Text { ref contents } = node.data {
            self.markdown.push_str(&contents.borrow());
        }
    }

    fn visit_paragraph(&mut self, node: Handle) {
        self.markdown.push_str("\n\n");

        self.visit_children(node);
    }

    fn visit_code(&mut self, node: Handle) {
        self.markdown.push_str("```\n");

        self.visit_children(node);

        self.markdown.push_str("```");
    }

    fn visit_italic(&mut self, node: Handle) {
        self.markdown.push_str("*");

        self.visit_children(node);

        self.markdown.push_str("*");
    }
}

#[test]
fn test_html_to_markdown() {
    use std::path::Path;
    use std::fs::File;
    use std::io::Read;

    let fixtures =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hacker_news/html_to_markdown/");

    let mut input_file = File::open(fixtures.join("input.html")).expect("Couldn't open fixture");
    let mut input = String::new();
    input_file
        .read_to_string(&mut input)
        .expect("Couldn't read fixture");

    let parsed = convert(&input).expect("Couldn't convert HTML to Markdown");

    let mut expected_file =
        File::open(fixtures.join("expected.markdown")).expect("Couldn't open fixture");
    let mut expected = String::new();
    expected_file
        .read_to_string(&mut expected)
        .expect("Couldn't read fixture");

    assert_eq!(expected, parsed);
}
