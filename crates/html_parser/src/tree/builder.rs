use crate::error::ParseError;
use crate::input::InputStream;
use crate::tokenizer::states::State;
use crate::tokenizer::tokens::{Attribute as TokenAttribute, Token};
use crate::tokenizer::Tokenizer;
use lyng_dom::document::QuirksMode;
use lyng_dom::element::{Attribute, Namespace};
use lyng_dom::node::{Arena, NodeData, NodeId};

use super::active::ActiveFormattingElements;
use super::foreign;
use super::insertion::InsertionMode;
use super::open::OpenElementsStack;

/// Foster parenting insert location.
enum FosterLocation {
    /// Insert before the given node (parent, before_node).
    InsertBefore(NodeId, NodeId),
    /// Append as a child of the given node.
    AppendTo(NodeId),
    /// No location found.
    None,
}

/// The result of parsing an HTML document.
pub struct ParseResult {
    pub arena: Arena,
    pub document: NodeId,
    pub errors: Vec<ParseError>,
    /// Present when the parser was run in fragment mode; points to the synthetic fragment root element.
    pub fragment_root: Option<NodeId>,
    /// The context element used for fragment parsing (if any).
    pub fragment_context: Option<NodeId>,
}

/// Describes the context element for fragment parsing.
#[derive(Clone)]
pub struct FragmentContext {
    pub tag_name: String,
    pub namespace: Namespace,
}

/// The HTML tree builder. Consumes tokens and builds the DOM tree.
pub struct TreeBuilder<'a> {
    pub arena: Arena,
    pub tokenizer: Tokenizer<'a>,
    pub insertion_mode: InsertionMode,
    pub original_insertion_mode: Option<InsertionMode>,
    pub template_modes: Vec<InsertionMode>,
    pub open_elements: OpenElementsStack,
    pub document: NodeId,
    pub head_pointer: Option<NodeId>,
    pub form_pointer: Option<NodeId>,
    pub foster_parenting: bool,
    pub scripting_enabled: bool,
    pub frameset_ok: bool,
    pub errors: Vec<ParseError>,
    pub active_formatting: ActiveFormattingElements,
    /// Pending table text tokens for InTableText mode
    pub pending_table_chars: Vec<Token>,
    /// Skip next LF character (after pre/listing/textarea start tags)
    skip_next_lf: bool,
    /// Synthetic root node used for fragment parsing (if any).
    fragment_root: Option<NodeId>,
    /// Fragment parsing context metadata.
    fragment_context: Option<FragmentContext>,
    /// Synthetic context element kept on the stack for fragment parsing (if any).
    fragment_context_node: Option<NodeId>,
    /// Extra synthetic ancestors added for fragment parsing to satisfy scope checks.
    fragment_extra_context: Vec<NodeId>,
    /// Tracks whether a selectedcontent post-processing pass is needed.
    needs_selectedcontent_population: bool,
}

impl<'a> TreeBuilder<'a> {
    pub fn new(input: &'a str) -> Self {
        let stream = InputStream::new(input);
        let tokenizer = Tokenizer::new(stream);
        let mut arena = Arena::new();

        let document = arena.create_node(NodeData::Document {
            quirks_mode: QuirksMode::NoQuirks,
        });

        TreeBuilder {
            arena,
            tokenizer,
            insertion_mode: InsertionMode::Initial,
            original_insertion_mode: None,
            template_modes: Vec::new(),
            open_elements: OpenElementsStack::new(),
            document,
            head_pointer: None,
            form_pointer: None,
            foster_parenting: false,
            scripting_enabled: false,
            frameset_ok: true,
            errors: Vec::new(),
            active_formatting: ActiveFormattingElements::new(),
            pending_table_chars: Vec::new(),
            skip_next_lf: false,
            fragment_root: None,
            fragment_context: None,
            fragment_context_node: None,
            fragment_extra_context: Vec::new(),
            needs_selectedcontent_population: false,
        }
    }

    /// Parse the document and return the result.
    pub fn run(mut self) -> ParseResult {
        loop {
            // Update the tokenizer's adjusted current node flag before getting next token
            if let Some(current) = self.adjusted_current_node() {
                let ns = self.open_elements.namespace(&self.arena, current);
                self.tokenizer.adjusted_current_node_not_in_html = ns != Namespace::Html;
            } else {
                self.tokenizer.adjusted_current_node_not_in_html = false;
            }

            let token = self.tokenizer.next_token();

            let is_eof = token == Token::EndOfFile;
            self.dispatch(token);
            if is_eof {
                break;
            }
        }

        // Normalize fragment trees if needed.
        self.normalize_fragment_tree();
        // Post-parse: populate <selectedcontent> elements
        if self.needs_selectedcontent_population {
            self.populate_selectedcontent();
        }

        // Collect errors from tokenizer
        let mut errors = std::mem::take(&mut self.tokenizer.errors);
        errors.extend(std::mem::take(&mut self.tokenizer.input.errors));
        errors.extend(std::mem::take(&mut self.errors));

        ParseResult {
            document: self.document,
            arena: self.arena,
            errors,
            fragment_root: self.fragment_root,
            fragment_context: self.fragment_context_node,
        }
    }

    /// Main dispatch: process a token according to the current insertion mode.
    fn dispatch(&mut self, token: Token) {
        if self.fragment_root.is_some() && self.should_ignore_table_fragment_token(&token) {
            return;
        }

        // Handle skip_next_lf for pre/listing/textarea
        if self.skip_next_lf {
            self.skip_next_lf = false;
            if let Token::Character { data: '\n' } = &token {
                return;
            }
        }

        if let Token::EndTag { name } = &token {
            if self.should_ignore_fragment_context_end_tag(name) {
                return;
            }
        }

        // Check if we should use foreign content rules
        if self.should_process_as_foreign(&token) {
            self.handle_foreign_content(token);
        } else {
            self.process_token(token, self.insertion_mode);
        }
    }

    fn should_ignore_table_fragment_token(&self, token: &Token) -> bool {
        let Token::StartTag { name, .. } = token else {
            return false;
        };
        if !matches_table_fragment_tag(name) {
            return false;
        }
        let current = match self.adjusted_current_node() {
            Some(id) => id,
            None => return false,
        };
        let ns = self.open_elements.namespace(&self.arena, current);
        let tag = self.open_elements.tag_name(&self.arena, current);
        (ns == Namespace::MathML && foreign::is_mathml_text_integration_point(tag))
            || (ns == Namespace::Svg && foreign::is_html_integration_point_svg(tag))
    }

    /// Determine if a token should be processed using the foreign content rules.
    fn should_process_as_foreign(&self, token: &Token) -> bool {
        let current = match self.adjusted_current_node() {
            Some(id) => id,
            None => return false,
        };
        let ns = self.open_elements.namespace(&self.arena, current);
        if ns == Namespace::Html {
            return false;
        }
        let tag = self.open_elements.tag_name(&self.arena, current);

        // MathML text integration points
        if ns == Namespace::MathML && foreign::is_mathml_text_integration_point(tag) {
            match token {
                Token::StartTag { name, .. } if name != "mglyph" && name != "malignmark" => {
                    return false;
                }
                Token::Character { .. } => return false,
                _ => {}
            }
        }

        // MathML annotation-xml with encoding
        if ns == Namespace::MathML && tag == "annotation-xml" {
            if let Token::StartTag { .. } = token {
                // Check encoding attribute for text/html or application/xhtml+xml
                if let NodeData::Element { attributes, .. } = &self.arena.get(current).data {
                    for attr in attributes {
                        if attr.name == "encoding" {
                            let val = attr.value.to_ascii_lowercase();
                            if val == "text/html" || val == "application/xhtml+xml" {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        // HTML integration points
        if ns == Namespace::Svg && foreign::is_html_integration_point_svg(tag) {
            match token {
                Token::StartTag { .. } | Token::Character { .. } => return false,
                _ => {}
            }
        }

        // EOF is never foreign
        if matches!(token, Token::EndOfFile) {
            return false;
        }

        true
    }

    /// Process a token in a given insertion mode. May be called recursively
    /// (e.g., "reprocess the token" in the spec).
    fn process_token(&mut self, token: Token, mode: InsertionMode) {
        match mode {
            InsertionMode::Initial => self.handle_initial(token),
            InsertionMode::BeforeHtml => self.handle_before_html(token),
            InsertionMode::BeforeHead => self.handle_before_head(token),
            InsertionMode::InHead => self.handle_in_head(token),
            InsertionMode::InHeadNoscript => self.handle_in_head_noscript(token),
            InsertionMode::AfterHead => self.handle_after_head(token),
            InsertionMode::InBody => self.handle_in_body(token),
            InsertionMode::Text => self.handle_text(token),
            InsertionMode::AfterBody => self.handle_after_body(token),
            InsertionMode::AfterAfterBody => self.handle_after_after_body(token),
            InsertionMode::InTable => self.handle_in_table(token),
            InsertionMode::InTableText => self.handle_in_table_text(token),
            InsertionMode::InCaption => self.handle_in_caption(token),
            InsertionMode::InColumnGroup => self.handle_in_column_group(token),
            InsertionMode::InTableBody => self.handle_in_table_body(token),
            InsertionMode::InRow => self.handle_in_row(token),
            InsertionMode::InCell => self.handle_in_cell(token),
            InsertionMode::InSelect => self.handle_in_select(token),
            InsertionMode::InSelectInTable => self.handle_in_select_in_table(token),
            InsertionMode::InTemplate => self.handle_in_template(token),
            InsertionMode::InFrameset => self.handle_in_frameset(token),
            InsertionMode::AfterFrameset => self.handle_after_frameset(token),
            InsertionMode::AfterAfterFrameset => self.handle_after_after_frameset(token),
        }
    }

    // -----------------------------------------------------------------------
    // Insertion mode handlers (stubs — Task 9 will implement the key ones)
    // -----------------------------------------------------------------------

    fn handle_initial(&mut self, token: Token) {
        match &token {
            Token::Character { .. } if token_is_all_whitespace(&token) => {
                // Ignore
            }
            Token::Comment { data } => {
                let comment = self.arena.create_node(NodeData::Comment {
                    content: data.clone(),
                });
                self.arena.append_child(self.document, comment);
            }
            Token::Doctype {
                name,
                public_id,
                system_id,
                force_quirks,
            } => {
                let doctype = self.arena.create_node(NodeData::Doctype {
                    name: name.clone().unwrap_or_default(),
                    public_id: public_id.clone().unwrap_or_default(),
                    system_id: system_id.clone().unwrap_or_default(),
                });
                self.arena.append_child(self.document, doctype);

                // Determine quirks mode
                let quirks = determine_quirks_mode(
                    name.as_deref(),
                    public_id.as_deref(),
                    system_id.as_deref(),
                    *force_quirks,
                );
                if let NodeData::Document { quirks_mode } =
                    &mut self.arena.get_mut(self.document).data
                {
                    *quirks_mode = quirks;
                }

                self.insertion_mode = InsertionMode::BeforeHtml;
            }
            _ => {
                // Parse error, switch to BeforeHtml and reprocess
                if let NodeData::Document { quirks_mode } =
                    &mut self.arena.get_mut(self.document).data
                {
                    *quirks_mode = QuirksMode::Quirks;
                }
                self.insertion_mode = InsertionMode::BeforeHtml;
                self.process_token(token, InsertionMode::BeforeHtml);
            }
        }
    }

    fn handle_before_html(&mut self, mut token: Token) {
        match &token {
            Token::Doctype { .. } => {
                // Parse error, ignore
            }
            Token::Comment { data } => {
                let comment = self.arena.create_node(NodeData::Comment {
                    content: data.clone(),
                });
                self.arena.append_child(self.document, comment);
            }
            Token::Character { .. } if token_is_all_whitespace(&token) => {
                // Ignore
            }
            Token::StartTag { name, .. } if name == "html" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.arena.append_child(self.document, element);
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::BeforeHead;
            }
            Token::EndTag { name }
                if name != "head" && name != "body" && name != "html" && name != "br" =>
            {
                // Parse error, ignore
            }
            _ => {
                let html = self.arena.create_node(NodeData::Element {
                    tag_name: "html".to_string(),
                    namespace: Namespace::Html,
                    attributes: vec![],
                });
                self.arena.append_child(self.document, html);
                self.open_elements.push(html);
                self.insertion_mode = InsertionMode::BeforeHead;
                self.process_token(token, InsertionMode::BeforeHead);
            }
        }
    }

    fn handle_before_head(&mut self, mut token: Token) {
        match &token {
            Token::Character { .. } if token_is_all_whitespace(&token) => {}
            Token::Comment { data } => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {}
            Token::StartTag { name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. } if name == "head" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.head_pointer = Some(element);
                self.insertion_mode = InsertionMode::InHead;
            }
            Token::EndTag { name }
                if name != "head" && name != "body" && name != "html" && name != "br" =>
            {
                // Parse error, ignore
            }
            _ => {
                let head = self.arena.create_node(NodeData::Element {
                    tag_name: "head".to_string(),
                    namespace: Namespace::Html,
                    attributes: vec![],
                });
                let target = self.current_node_or_document();
                self.arena.append_child(target, head);
                self.open_elements.push(head);
                self.head_pointer = Some(head);
                self.insertion_mode = InsertionMode::InHead;
                self.process_token(token, InsertionMode::InHead);
            }
        }
    }

    fn handle_in_head(&mut self, mut token: Token) {
        match &token {
            Token::Character { data } if token_is_all_whitespace(&token) => {
                self.insert_character(*data);
            }
            Token::Comment { data } => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {}
            Token::StartTag { name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. }
                if name == "base" || name == "basefont" || name == "bgsound" || name == "link" =>
            {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                // Void element — don't push to stack
            }
            Token::StartTag { name, .. } if name == "meta" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
            }
            Token::StartTag { name, .. } if name == "title" => {
                self.parse_generic_rcdata(&mut token);
            }
            Token::StartTag { name, .. } if name == "noscript" && self.scripting_enabled => {
                self.parse_generic_raw_text(&mut token);
            }
            Token::StartTag { name, .. } if name == "noframes" || name == "style" => {
                self.parse_generic_raw_text(&mut token);
            }
            Token::StartTag { name, .. } if name == "noscript" && !self.scripting_enabled => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InHeadNoscript;
            }
            Token::StartTag { name, .. } if name == "script" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.tokenizer.set_state(State::ScriptData);
                self.original_insertion_mode = Some(self.insertion_mode);
                self.insertion_mode = InsertionMode::Text;
            }
            Token::EndTag { name } if name == "head" => {
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::AfterHead;
            }
            Token::StartTag { name, .. } if name == "template" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                // Create template content document fragment
                let content = self.arena.create_node(NodeData::Document {
                    quirks_mode: QuirksMode::NoQuirks,
                });
                self.arena.append_child(element, content);
                self.open_elements.push(element);
                self.active_formatting.push_marker();
                self.template_modes.push(InsertionMode::InTemplate);
                self.insertion_mode = InsertionMode::InTemplate;
            }
            Token::EndTag { name } if name == "template" => {
                if !self
                    .open_elements
                    .contains_html_tag(&self.arena, "template")
                {
                    // Parse error, ignore
                    return;
                }
                self.open_elements
                    .generate_all_implied_end_tags_thoroughly(&self.arena);
                self.open_elements.pop_until_html(&self.arena, "template");
                self.active_formatting.clear_up_to_last_marker();
                self.template_modes.pop();
                self.reset_insertion_mode();
            }
            Token::EndTag { name } if name != "body" && name != "html" && name != "br" => {
                // Parse error, ignore
            }
            Token::StartTag { name, .. } if name == "head" => {
                // Parse error, ignore
            }
            _ => {
                self.open_elements.pop(); // pop head
                self.insertion_mode = InsertionMode::AfterHead;
                self.process_token(token, InsertionMode::AfterHead);
            }
        }
    }

    fn handle_in_head_noscript(&mut self, token: Token) {
        match &token {
            Token::Doctype { .. } => {}
            Token::StartTag { name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::EndTag { name } if name == "noscript" => {
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::InHead;
            }
            Token::Character { .. } if token_is_all_whitespace(&token) => {
                self.handle_in_head(token);
            }
            Token::Comment { .. } => {
                self.handle_in_head(token);
            }
            Token::StartTag { name, .. }
                if name == "basefont"
                    || name == "bgsound"
                    || name == "link"
                    || name == "meta"
                    || name == "noframes"
                    || name == "style" =>
            {
                self.handle_in_head(token);
            }
            Token::StartTag { name, .. } if name == "head" || name == "noscript" => {
                // Parse error, ignore
            }
            Token::EndTag { name } if name != "br" => {
                // Parse error, ignore
            }
            _ => {
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::InHead;
                self.process_token(token, InsertionMode::InHead);
            }
        }
    }

    fn handle_after_head(&mut self, mut token: Token) {
        match &token {
            Token::Character { data } if token_is_all_whitespace(&token) => {
                self.insert_character(*data);
            }
            Token::Comment { data } => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {}
            Token::StartTag { name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. } if name == "body" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.frameset_ok = false;
                self.insertion_mode = InsertionMode::InBody;
            }
            Token::StartTag { name, .. } if name == "frameset" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InFrameset;
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "base"
                        | "basefont"
                        | "bgsound"
                        | "link"
                        | "meta"
                        | "noframes"
                        | "script"
                        | "style"
                        | "template"
                        | "title"
                ) =>
            {
                // Push head back, process in InHead, then remove head
                if let Some(head) = self.head_pointer {
                    self.open_elements.push(head);
                    self.handle_in_head(token);
                    self.open_elements.remove(head);
                }
            }
            Token::EndTag { name } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::EndTag { name } if name != "body" && name != "html" && name != "br" => {
                // Parse error, ignore
            }
            Token::StartTag { name, .. } if name == "head" => {
                // Parse error, ignore
            }
            _ => {
                if token == Token::EndOfFile
                    && self.fragment_root.is_some()
                    && !matches!(
                        self.fragment_context.as_ref(),
                        Some(context)
                            if context.namespace == Namespace::Html && context.tag_name == "html"
                    )
                {
                    return;
                }
                let body = self.arena.create_node(NodeData::Element {
                    tag_name: "body".to_string(),
                    namespace: Namespace::Html,
                    attributes: vec![],
                });
                let target = self.current_node_or_document();
                self.arena.append_child(target, body);
                self.open_elements.push(body);
                self.insertion_mode = InsertionMode::InBody;
                self.process_token(token, InsertionMode::InBody);
            }
        }
    }

    fn handle_in_body(&mut self, mut token: Token) {
        match &token {
            Token::Character { data } if token_is_all_whitespace(&token) => {
                self.reconstruct_active_formatting();
                self.insert_character(*data);
            }
            Token::Character { data } => {
                self.reconstruct_active_formatting();
                if *data == '\0' {
                    return;
                }
                self.insert_character(*data);
                self.frameset_ok = false;
            }
            Token::Comment { data } => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // Parse error, ignore
            }
            Token::StartTag { name, .. } if name == "html" => {
                // Parse error. If there is a template element on the stack, ignore.
                if self.open_elements.contains_tag(&self.arena, "template") {
                    return;
                }
                // Transfer attributes to existing html element
                if let Some(&html_id) = self.open_elements.elements.first() {
                    if let Token::StartTag { attributes, .. } = &token {
                        self.transfer_attributes(html_id, attributes);
                    }
                }
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "base"
                        | "basefont"
                        | "bgsound"
                        | "link"
                        | "meta"
                        | "noframes"
                        | "script"
                        | "style"
                        | "template"
                        | "title"
                ) =>
            {
                self.handle_in_head(token);
            }
            Token::EndTag { name } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::StartTag { name, .. } if name == "body" => {
                // If there is a template element on the stack, ignore.
                if self.open_elements.contains_tag(&self.arena, "template") {
                    return;
                }
                // Transfer attributes to body element
                if self.open_elements.len() >= 2 {
                    let body_id = self.open_elements.elements[1];
                    if let Token::StartTag { attributes, .. } = &token {
                        if let NodeData::Element { tag_name, .. } = &self.arena.get(body_id).data {
                            if tag_name == "body" {
                                self.frameset_ok = false;
                                self.transfer_attributes(body_id, attributes);
                            }
                        }
                    }
                }
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "address"
                        | "article"
                        | "aside"
                        | "blockquote"
                        | "center"
                        | "details"
                        | "dialog"
                        | "dir"
                        | "div"
                        | "dl"
                        | "fieldset"
                        | "figcaption"
                        | "figure"
                        | "footer"
                        | "header"
                        | "hgroup"
                        | "main"
                        | "menu"
                        | "nav"
                        | "ol"
                        | "p"
                        | "search"
                        | "section"
                        | "summary"
                        | "ul"
                ) =>
            {
                if self.open_elements.has_in_button_scope(&self.arena, "p") {
                    self.close_p_element();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::StartTag { name, .. }
                if matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") =>
            {
                if self.open_elements.has_in_button_scope(&self.arena, "p") {
                    self.close_p_element();
                }
                // If current node is h1-h6, pop it (parse error)
                if let Some(current) = self.open_elements.current_node() {
                    if let NodeData::Element { tag_name, .. } = &self.arena.get(current).data {
                        if matches!(tag_name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                            self.open_elements.pop();
                        }
                    }
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::StartTag { name, .. } if name == "pre" || name == "listing" => {
                if self.open_elements.has_in_button_scope(&self.arena, "p") {
                    self.close_p_element();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.skip_next_lf = true;
                self.frameset_ok = false;
            }
            Token::StartTag { name, .. } if name == "plaintext" => {
                if self.open_elements.has_in_button_scope(&self.arena, "p") {
                    self.close_p_element();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.tokenizer.set_state(State::PlainText);
            }
            Token::StartTag { name, .. } if name == "li" => {
                self.frameset_ok = false;
                // Close any open li
                for i in (0..self.open_elements.elements.len()).rev() {
                    let id = self.open_elements.elements[i];
                    let tag = self.open_elements.tag_name(&self.arena, id);
                    if tag == "li" {
                        self.open_elements
                            .generate_implied_end_tags(&self.arena, Some("li"));
                        self.open_elements.pop_until(&self.arena, "li");
                        break;
                    }
                    if is_special(tag) && tag != "address" && tag != "div" && tag != "p" {
                        break;
                    }
                }
                if self.open_elements.has_in_button_scope(&self.arena, "p") {
                    self.close_p_element();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::StartTag { name, .. } if name == "dd" || name == "dt" => {
                self.frameset_ok = false;
                for i in (0..self.open_elements.elements.len()).rev() {
                    let id = self.open_elements.elements[i];
                    let tag = self.open_elements.tag_name(&self.arena, id).to_string();
                    if tag == "dd" || tag == "dt" {
                        self.open_elements
                            .generate_implied_end_tags(&self.arena, Some(&tag));
                        self.open_elements.pop_until(&self.arena, &tag);
                        break;
                    }
                    if is_special(&tag) && tag != "address" && tag != "div" && tag != "p" {
                        break;
                    }
                }
                if self.open_elements.has_in_button_scope(&self.arena, "p") {
                    self.close_p_element();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::StartTag { name, .. } if name == "form" => {
                if self.form_pointer.is_some()
                    && !self.open_elements.contains_tag(&self.arena, "template")
                {
                    // Parse error, ignore
                    return;
                }
                if self.open_elements.has_in_button_scope(&self.arena, "p") {
                    self.close_p_element();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                if !self.open_elements.contains_tag(&self.arena, "template") {
                    self.form_pointer = Some(element);
                }
            }
            Token::StartTag { name, .. } if name == "a" => {
                // If there's an <a> in the active formatting list, run adoption agency
                if let Some((_idx, existing_a)) =
                    self.active_formatting.find_by_tag(&self.arena, "a")
                {
                    self.run_adoption_agency("a");
                    // If the element is still in the AFL/stack after adoption agency,
                    // remove it (spec step: "remove the element from the list" and
                    // "if the element is also in the stack of open elements, remove it")
                    self.active_formatting.remove(existing_a);
                    self.open_elements.remove(existing_a);
                }
                self.reconstruct_active_formatting();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.active_formatting.push(&self.arena, element);
            }
            Token::StartTag { name, .. } if is_formatting_element(name) => {
                self.reconstruct_active_formatting();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.active_formatting.push(&self.arena, element);
            }
            Token::StartTag { name, .. } if name == "nobr" => {
                self.reconstruct_active_formatting();
                if self.open_elements.has_in_scope(&self.arena, "nobr") {
                    self.run_adoption_agency("nobr");
                    self.reconstruct_active_formatting();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.active_formatting.push(&self.arena, element);
            }
            Token::StartTag { name, .. } if name == "button" => {
                if self.open_elements.has_in_scope(&self.arena, "button") {
                    self.open_elements
                        .generate_implied_end_tags(&self.arena, None);
                    self.open_elements.pop_until(&self.arena, "button");
                }
                self.reconstruct_active_formatting();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.frameset_ok = false;
            }
            Token::StartTag { name, .. }
                if matches!(name.as_str(), "applet" | "marquee" | "object") =>
            {
                self.reconstruct_active_formatting();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.active_formatting.push_marker();
                self.frameset_ok = false;
            }
            Token::StartTag { name, .. } if name == "rb" || name == "rtc" => {
                if self.open_elements.has_in_scope(&self.arena, "ruby") {
                    self.open_elements
                        .generate_implied_end_tags(&self.arena, None);
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::StartTag { name, .. } if name == "rp" || name == "rt" => {
                if self.open_elements.has_in_scope(&self.arena, "ruby") {
                    self.open_elements
                        .generate_implied_end_tags(&self.arena, Some("rtc"));
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "area" | "br" | "embed" | "img" | "keygen" | "wbr"
                ) =>
            {
                self.reconstruct_active_formatting();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.frameset_ok = false;
            }
            Token::StartTag { name, .. } if name == "input" => {
                self.reconstruct_active_formatting();
                let is_hidden = if let Token::StartTag { attributes, .. } = &token {
                    attributes
                        .iter()
                        .any(|a| a.name == "type" && a.value.eq_ignore_ascii_case("hidden"))
                } else {
                    false
                };
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                if !is_hidden {
                    self.frameset_ok = false;
                }
            }
            Token::StartTag { name, .. }
                if matches!(name.as_str(), "param" | "source" | "track") =>
            {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                // Void elements — don't push to stack
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "caption"
                        | "col"
                        | "colgroup"
                        | "frame"
                        | "head"
                        | "tbody"
                        | "td"
                        | "tfoot"
                        | "th"
                        | "thead"
                        | "tr"
                ) =>
            {
                // Parse error, ignore in InBody
            }
            Token::StartTag { name, .. } if name == "hr" => {
                if self.open_elements.has_in_button_scope(&self.arena, "p") {
                    self.close_p_element();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.frameset_ok = false;
            }
            Token::StartTag { name, .. } if name == "image" => {
                // Parse error: correct to "img"
                let corrected = Token::StartTag {
                    name: "img".to_string(),
                    attributes: if let Token::StartTag { attributes, .. } = &token {
                        attributes.clone()
                    } else {
                        vec![]
                    },
                    self_closing: if let Token::StartTag { self_closing, .. } = &token {
                        *self_closing
                    } else {
                        false
                    },
                };
                self.process_token(corrected, InsertionMode::InBody);
            }
            Token::StartTag { name, .. } if name == "textarea" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.skip_next_lf = true;
                self.tokenizer.set_state(State::RcData);
                self.original_insertion_mode = Some(self.insertion_mode);
                self.frameset_ok = false;
                self.insertion_mode = InsertionMode::Text;
            }
            Token::StartTag { name, .. } if name == "xmp" => {
                if self.open_elements.has_in_button_scope(&self.arena, "p") {
                    self.close_p_element();
                }
                // TODO: reconstruct active formatting elements
                self.frameset_ok = false;
                self.parse_generic_raw_text(&mut token);
            }
            Token::StartTag { name, .. } if name == "iframe" => {
                self.frameset_ok = false;
                self.parse_generic_raw_text(&mut token);
            }
            Token::StartTag { name, .. }
                if name == "noembed" || (name == "noscript" && self.scripting_enabled) =>
            {
                self.parse_generic_raw_text(&mut token);
            }
            Token::StartTag { name, .. } if name == "select" => {
                self.reconstruct_active_formatting();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.frameset_ok = false;
                if matches!(
                    self.insertion_mode,
                    InsertionMode::InTable
                        | InsertionMode::InCaption
                        | InsertionMode::InTableBody
                        | InsertionMode::InRow
                        | InsertionMode::InCell
                ) {
                    self.insertion_mode = InsertionMode::InSelectInTable;
                } else {
                    self.insertion_mode = InsertionMode::InSelect;
                }
            }
            Token::StartTag { name, .. } if name == "optgroup" || name == "option" => {
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "option" {
                        self.open_elements.pop();
                    }
                }
                self.reconstruct_active_formatting();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::StartTag { name, .. } if name == "frameset" => {
                if !self.frameset_ok {
                    return;
                }
                // If there is a template element on the stack, ignore.
                if self.open_elements.contains_tag(&self.arena, "template") {
                    return;
                }
                // Remove body from stack if present
                if self.open_elements.len() >= 2 {
                    let body_id = self.open_elements.elements[1];
                    if self.open_elements.tag_name(&self.arena, body_id) == "body" {
                        self.arena.detach(body_id);
                    }
                }
                // Pop all but html
                while self.open_elements.len() > 1 {
                    self.open_elements.pop();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InFrameset;
            }
            Token::StartTag { name, .. } if name == "table" => {
                // If quirks mode is not "quirks" and p in button scope, close p
                let quirks = if let NodeData::Document { quirks_mode } =
                    &self.arena.get(self.document).data
                {
                    *quirks_mode
                } else {
                    QuirksMode::NoQuirks
                };
                if quirks != QuirksMode::Quirks
                    && self.open_elements.has_in_button_scope(&self.arena, "p")
                {
                    self.close_p_element();
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.frameset_ok = false;
                self.insertion_mode = InsertionMode::InTable;
            }
            // End tags
            Token::EndTag { name } if name == "body" => {
                if !self.open_elements.has_in_scope(&self.arena, "body") {
                    return;
                }
                self.insertion_mode = InsertionMode::AfterBody;
            }
            Token::EndTag { name } if name == "html" => {
                if !self.open_elements.has_in_scope(&self.arena, "body") {
                    return;
                }
                self.insertion_mode = InsertionMode::AfterBody;
                self.process_token(token, InsertionMode::AfterBody);
            }
            Token::EndTag { name }
                if matches!(
                    name.as_str(),
                    "address"
                        | "article"
                        | "aside"
                        | "blockquote"
                        | "button"
                        | "center"
                        | "details"
                        | "dialog"
                        | "dir"
                        | "div"
                        | "dl"
                        | "fieldset"
                        | "figcaption"
                        | "figure"
                        | "footer"
                        | "header"
                        | "hgroup"
                        | "listing"
                        | "main"
                        | "menu"
                        | "nav"
                        | "ol"
                        | "pre"
                        | "search"
                        | "section"
                        | "summary"
                        | "ul"
                ) =>
            {
                let name = name.clone();
                if !self.open_elements.has_in_scope(&self.arena, &name) {
                    return;
                }
                self.open_elements
                    .generate_implied_end_tags(&self.arena, None);
                self.open_elements.pop_until(&self.arena, &name);
            }
            Token::EndTag { name } if name == "form" => {
                if !self.open_elements.contains_tag(&self.arena, "template") {
                    let node = self.form_pointer.take();
                    if let Some(id) = node {
                        if !self.open_elements.has_in_scope(&self.arena, "form") {
                            return;
                        }
                        self.open_elements
                            .generate_implied_end_tags(&self.arena, None);
                        self.open_elements.remove(id);
                    }
                } else {
                    if !self.open_elements.has_in_scope(&self.arena, "form") {
                        return;
                    }
                    self.open_elements
                        .generate_implied_end_tags(&self.arena, None);
                    self.open_elements.pop_until(&self.arena, "form");
                }
            }
            Token::EndTag { name } if name == "p" => {
                if !self.open_elements.has_in_button_scope(&self.arena, "p") {
                    let p = self.arena.create_node(NodeData::Element {
                        tag_name: "p".to_string(),
                        namespace: Namespace::Html,
                        attributes: vec![],
                    });
                    self.insert_node_at_appropriate_location(p);
                    self.open_elements.push(p);
                }
                self.close_p_element();
            }
            Token::EndTag { name } if name == "li" => {
                if !self.open_elements.has_in_list_item_scope(&self.arena, "li") {
                    return;
                }
                self.open_elements
                    .generate_implied_end_tags(&self.arena, Some("li"));
                self.open_elements.pop_until(&self.arena, "li");
            }
            Token::EndTag { name } if name == "dd" || name == "dt" => {
                let name = name.clone();
                if !self.open_elements.has_in_scope(&self.arena, &name) {
                    return;
                }
                self.open_elements
                    .generate_implied_end_tags(&self.arena, Some(&name));
                self.open_elements.pop_until(&self.arena, &name);
            }
            Token::EndTag { name }
                if matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") =>
            {
                if !self.open_elements.has_in_scope(&self.arena, "h1")
                    && !self.open_elements.has_in_scope(&self.arena, "h2")
                    && !self.open_elements.has_in_scope(&self.arena, "h3")
                    && !self.open_elements.has_in_scope(&self.arena, "h4")
                    && !self.open_elements.has_in_scope(&self.arena, "h5")
                    && !self.open_elements.has_in_scope(&self.arena, "h6")
                {
                    return;
                }
                self.open_elements
                    .generate_implied_end_tags(&self.arena, None);
                self.open_elements
                    .pop_until_one_of(&self.arena, &["h1", "h2", "h3", "h4", "h5", "h6"]);
            }
            Token::EndTag { name }
                if name == "a" || name == "nobr" || is_formatting_element(name) =>
            {
                let name = name.clone();
                self.run_adoption_agency(&name);
            }
            Token::EndTag { name } if matches!(name.as_str(), "applet" | "marquee" | "object") => {
                let name = name.clone();
                if !self.open_elements.has_in_scope(&self.arena, &name) {
                    return;
                }
                self.open_elements
                    .generate_implied_end_tags(&self.arena, None);
                self.open_elements.pop_until(&self.arena, &name);
                self.active_formatting.clear_up_to_last_marker();
            }
            Token::StartTag { name, .. } if name == "math" => {
                self.reconstruct_active_formatting();
                let element = self.create_foreign_element_from_token(&mut token, Namespace::MathML);
                self.insert_node_at_appropriate_location(element);
                if let Token::StartTag { self_closing, .. } = &token {
                    if !*self_closing {
                        self.open_elements.push(element);
                    }
                }
            }
            Token::StartTag { name, .. } if name == "svg" => {
                self.reconstruct_active_formatting();
                let element = self.create_foreign_element_from_token(&mut token, Namespace::Svg);
                self.insert_node_at_appropriate_location(element);
                if let Token::StartTag { self_closing, .. } = &token {
                    if !*self_closing {
                        self.open_elements.push(element);
                    }
                }
            }
            Token::EndOfFile => {
                // If the stack of template insertion modes is not empty,
                // process the token using the rules for the "in template" mode.
                if !self.template_modes.is_empty() {
                    self.handle_in_template(token);
                }
                // Otherwise stop parsing
            }
            // Any other start tag
            Token::StartTag { .. } => {
                self.reconstruct_active_formatting();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::EndTag { name } if name == "br" => {
                // Parse error: treat as <br> start tag
                self.reconstruct_active_formatting();
                let element = self.arena.create_node(NodeData::Element {
                    tag_name: "br".to_string(),
                    namespace: Namespace::Html,
                    attributes: vec![],
                });
                self.insert_node_at_appropriate_location(element);
                self.frameset_ok = false;
            }
            // Any other end tag
            Token::EndTag { name } => {
                let name = name.clone();
                // Walk the stack backwards
                for i in (0..self.open_elements.elements.len()).rev() {
                    let id = self.open_elements.elements[i];
                    let tag = self.open_elements.tag_name(&self.arena, id);
                    if tag == name {
                        self.open_elements
                            .generate_implied_end_tags(&self.arena, Some(&name));
                        // Pop up to and including this element
                        while self.open_elements.elements.len() > i {
                            self.open_elements.pop();
                        }
                        break;
                    }
                    if is_special(tag) {
                        // Parse error, ignore
                        break;
                    }
                }
            }
        }
    }

    fn handle_text(&mut self, token: Token) {
        match &token {
            Token::Character { data } => {
                self.insert_character(*data);
            }
            Token::EndOfFile => {
                self.open_elements.pop();
                self.insertion_mode = self
                    .original_insertion_mode
                    .unwrap_or(InsertionMode::InBody);
                self.process_token(token, self.insertion_mode);
            }
            Token::EndTag { name } if name == "script" => {
                let script_node = self.open_elements.current_node();
                self.open_elements.pop();
                self.insertion_mode = self
                    .original_insertion_mode
                    .unwrap_or(InsertionMode::InBody);
                if let Some(script_node) = script_node {
                    self.execute_script(script_node);
                }
            }
            Token::EndTag { .. } => {
                self.open_elements.pop();
                self.insertion_mode = self
                    .original_insertion_mode
                    .unwrap_or(InsertionMode::InBody);
            }
            _ => {}
        }
    }

    fn handle_after_body(&mut self, token: Token) {
        match &token {
            Token::Character { .. } if token_is_all_whitespace(&token) => {
                self.handle_in_body(token);
            }
            Token::Comment { data } => {
                // Append to html element
                let comment = self.arena.create_node(NodeData::Comment {
                    content: data.clone(),
                });
                if let Some(&html) = self.open_elements.elements.first() {
                    self.arena.append_child(html, comment);
                }
            }
            Token::Doctype { .. } => {}
            Token::StartTag { name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::EndTag { name } if name == "html" => {
                self.insertion_mode = InsertionMode::AfterAfterBody;
            }
            Token::EndOfFile => {}
            _ => {
                self.insertion_mode = InsertionMode::InBody;
                self.process_token(token, InsertionMode::InBody);
            }
        }
    }

    fn handle_after_after_body(&mut self, token: Token) {
        match &token {
            Token::Comment { data } => {
                let comment = self.arena.create_node(NodeData::Comment {
                    content: data.clone(),
                });
                self.arena.append_child(self.document, comment);
            }
            Token::Doctype { .. } => {
                self.handle_in_body(token);
            }
            Token::Character { .. } if token_is_all_whitespace(&token) => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::EndOfFile => {}
            _ => {
                self.insertion_mode = InsertionMode::InBody;
                self.process_token(token, InsertionMode::InBody);
            }
        }
    }

    // Stub handlers for modes we'll implement later
    fn handle_in_table(&mut self, mut token: Token) {
        match &token {
            Token::Character { .. }
                if self.current_node_is_one_of(&["table", "tbody", "tfoot", "thead", "tr"]) =>
            {
                self.pending_table_chars.clear();
                self.original_insertion_mode = Some(self.insertion_mode);
                self.insertion_mode = InsertionMode::InTableText;
                self.process_token(token, InsertionMode::InTableText);
            }
            Token::Comment { data } => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {}
            Token::StartTag { name, .. } if name == "caption" => {
                if !self.has_table_in_scope_or_template() {
                    // Parse error, ignore.
                    return;
                }
                self.clear_stack_to_table_context();
                self.active_formatting.push_marker();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InCaption;
            }
            Token::StartTag { name, .. } if name == "colgroup" => {
                if !self.has_table_in_scope_or_template() {
                    return;
                }
                self.clear_stack_to_table_context();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InColumnGroup;
            }
            Token::StartTag { name, .. } if name == "col" => {
                if !self.has_table_in_scope_or_template() {
                    return;
                }
                self.clear_stack_to_table_context();
                let colgroup = self.arena.create_node(NodeData::Element {
                    tag_name: "colgroup".to_string(),
                    namespace: Namespace::Html,
                    attributes: vec![],
                });
                let target = self.current_node_or_document();
                self.arena.append_child(target, colgroup);
                self.open_elements.push(colgroup);
                self.insertion_mode = InsertionMode::InColumnGroup;
                self.process_token(token, InsertionMode::InColumnGroup);
            }
            Token::StartTag { name, .. }
                if matches!(name.as_str(), "tbody" | "tfoot" | "thead") =>
            {
                if !self.has_table_in_scope_or_template() {
                    return;
                }
                self.clear_stack_to_table_context();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InTableBody;
            }
            Token::StartTag { name, .. } if name == "td" || name == "th" || name == "tr" => {
                if !self.has_table_in_scope_or_template() {
                    return;
                }
                self.clear_stack_to_table_context();
                let tbody = self.arena.create_node(NodeData::Element {
                    tag_name: "tbody".to_string(),
                    namespace: Namespace::Html,
                    attributes: vec![],
                });
                let target = self.current_node_or_document();
                self.arena.append_child(target, tbody);
                self.open_elements.push(tbody);
                self.insertion_mode = InsertionMode::InTableBody;
                self.process_token(token, InsertionMode::InTableBody);
            }
            Token::StartTag { name, .. } if name == "table" => {
                if self.last_matching_open_html_element_is_protected(&["table"]) {
                    return;
                }
                // Parse error, close current table and reprocess
                if self.open_elements.has_in_table_scope(&self.arena, "table") {
                    self.open_elements.pop_until(&self.arena, "table");
                    self.reset_insertion_mode();
                    self.process_token(token, self.insertion_mode);
                }
            }
            Token::EndTag { name } if name == "table" => {
                if self.last_matching_open_html_element_is_protected(&["table"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, "table") {
                    return;
                }
                self.open_elements.pop_until(&self.arena, "table");
                self.reset_insertion_mode();
            }
            Token::EndTag { name }
                if matches!(
                    name.as_str(),
                    "body"
                        | "caption"
                        | "col"
                        | "colgroup"
                        | "html"
                        | "tbody"
                        | "td"
                        | "tfoot"
                        | "th"
                        | "thead"
                        | "tr"
                ) =>
            {
                // Parse error, ignore
            }
            Token::StartTag { name, .. }
                if name == "style" || name == "script" || name == "template" =>
            {
                self.handle_in_head(token);
            }
            Token::EndTag { name } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::StartTag { name, .. } if name == "input" => {
                if let Token::StartTag { attributes, .. } = &token {
                    let is_hidden = attributes
                        .iter()
                        .any(|a| a.name == "type" && a.value.eq_ignore_ascii_case("hidden"));
                    if is_hidden {
                        let element = self.create_element_from_token(&mut token, Namespace::Html);
                        self.insert_node_at_appropriate_location(element);
                        return;
                    }
                }
                // Not hidden — foster parent
                self.foster_parenting = true;
                self.handle_in_body(token);
                self.foster_parenting = false;
            }
            Token::StartTag { name, .. } if name == "form" => {
                if self.open_elements.contains_tag(&self.arena, "template")
                    || self.form_pointer.is_some()
                {
                    return;
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.form_pointer = Some(element);
            }
            Token::EndOfFile => {
                self.handle_in_body(token);
            }
            _ => {
                // If the current node is in a foreign namespace (e.g. math/svg
                // foster-parented out of the table), process as foreign content
                // instead of foster-parenting through InBody.
                // But only if should_process_as_foreign agrees.
                if self.should_process_as_foreign(&token) {
                    self.handle_foreign_content(token);
                    return;
                }
                // Foster parent: process in InBody with foster parenting enabled
                self.foster_parenting = true;
                self.handle_in_body(token);
                self.foster_parenting = false;
            }
        }
    }

    fn handle_in_table_text(&mut self, token: Token) {
        match token {
            Token::Character { data: '\0' } => {}
            Token::Character { .. } => {
                self.pending_table_chars.push(token);
            }
            _ => {
                // Check if pending chars are all whitespace
                let all_whitespace = self.pending_table_chars.iter().all(token_is_all_whitespace);

                let chars = std::mem::take(&mut self.pending_table_chars);
                if all_whitespace {
                    for ch_token in chars {
                        if let Token::Character { data } = ch_token {
                            self.insert_character(data);
                        }
                    }
                } else {
                    // Foster parent the characters
                    for ch_token in chars {
                        self.foster_parenting = true;
                        self.handle_in_body(ch_token);
                        self.foster_parenting = false;
                    }
                }

                self.insertion_mode = self
                    .original_insertion_mode
                    .unwrap_or(InsertionMode::InTable);
                self.process_token(token, self.insertion_mode);
            }
        }
    }

    fn handle_in_caption(&mut self, token: Token) {
        match &token {
            Token::EndTag { name } if name == "caption" => {
                if !self
                    .open_elements
                    .has_in_table_scope(&self.arena, "caption")
                {
                    return;
                }
                self.open_elements
                    .generate_implied_end_tags(&self.arena, None);
                self.open_elements.pop_until(&self.arena, "caption");
                self.active_formatting.clear_up_to_last_marker();
                self.insertion_mode = InsertionMode::InTable;
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "caption"
                        | "col"
                        | "colgroup"
                        | "tbody"
                        | "td"
                        | "tfoot"
                        | "th"
                        | "thead"
                        | "tr"
                ) =>
            {
                if self.last_matching_open_html_element_is_protected(&["caption"]) {
                    return;
                }
                if !self
                    .open_elements
                    .has_in_table_scope(&self.arena, "caption")
                {
                    return;
                }
                self.open_elements
                    .generate_implied_end_tags(&self.arena, None);
                self.open_elements.pop_until(&self.arena, "caption");
                self.active_formatting.clear_up_to_last_marker();
                self.insertion_mode = InsertionMode::InTable;
                self.process_token(token, InsertionMode::InTable);
            }
            Token::EndTag { name } if name == "table" => {
                if self.last_matching_open_html_element_is_protected(&["caption"]) {
                    return;
                }
                if !self
                    .open_elements
                    .has_in_table_scope(&self.arena, "caption")
                {
                    return;
                }
                self.open_elements
                    .generate_implied_end_tags(&self.arena, None);
                self.open_elements.pop_until(&self.arena, "caption");
                self.active_formatting.clear_up_to_last_marker();
                self.insertion_mode = InsertionMode::InTable;
                self.process_token(token, InsertionMode::InTable);
            }
            Token::EndTag { name }
                if matches!(
                    name.as_str(),
                    "body"
                        | "col"
                        | "colgroup"
                        | "html"
                        | "tbody"
                        | "td"
                        | "tfoot"
                        | "th"
                        | "thead"
                        | "tr"
                ) =>
            {
                // Parse error, ignore
            }
            _ => {
                self.handle_in_body(token);
            }
        }
    }

    fn handle_in_column_group(&mut self, mut token: Token) {
        match &token {
            Token::Character { data } if token_is_all_whitespace(&token) => {
                self.insert_character(*data);
            }
            Token::Comment { data } => self.insert_comment(data),
            Token::Doctype { .. } => {}
            Token::StartTag { name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. } if name == "col" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
            }
            Token::StartTag { name, .. } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::EndTag { name } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::EndTag { name } if name == "colgroup" => {
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "colgroup" {
                        self.open_elements.pop();
                        self.insertion_mode = InsertionMode::InTable;
                    }
                }
            }
            Token::EndTag { name } if name == "col" => {
                // Parse error, ignore
            }
            Token::EndOfFile => {
                self.handle_in_body(token);
            }
            _ => {
                if let Some(current) = self.open_elements.current_node() {
                    if self.fragment_root.is_some() && Some(current) == self.fragment_context_node {
                        return;
                    }
                    if self.open_elements.tag_name(&self.arena, current) == "colgroup" {
                        self.open_elements.pop();
                        self.insertion_mode = InsertionMode::InTable;
                        self.process_token(token, InsertionMode::InTable);
                    }
                }
            }
        }
    }

    fn handle_in_table_body(&mut self, mut token: Token) {
        match &token {
            Token::StartTag { name, .. } if name == "tr" => {
                self.clear_stack_to_table_body_context();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InRow;
            }
            Token::StartTag { name, .. } if name == "th" || name == "td" => {
                self.clear_stack_to_table_body_context();
                let tr = self.arena.create_node(NodeData::Element {
                    tag_name: "tr".to_string(),
                    namespace: Namespace::Html,
                    attributes: vec![],
                });
                let target = self.current_node_or_document();
                self.arena.append_child(target, tr);
                self.open_elements.push(tr);
                self.insertion_mode = InsertionMode::InRow;
                self.process_token(token, InsertionMode::InRow);
            }
            Token::EndTag { name } if matches!(name.as_str(), "tbody" | "tfoot" | "thead") => {
                if self.last_matching_open_html_element_is_protected(&["tbody", "tfoot", "thead"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, name) {
                    return;
                }
                self.clear_stack_to_table_body_context();
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::InTable;
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "caption" | "col" | "colgroup" | "tbody" | "tfoot" | "thead"
                ) =>
            {
                if self.last_matching_open_html_element_is_protected(&["tbody", "tfoot", "thead"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, "tbody")
                    && !self.open_elements.has_in_table_scope(&self.arena, "thead")
                    && !self.open_elements.has_in_table_scope(&self.arena, "tfoot")
                {
                    return;
                }
                self.clear_stack_to_table_body_context();
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::InTable;
                self.process_token(token, InsertionMode::InTable);
            }
            Token::EndTag { name } if name == "table" => {
                if self.last_matching_open_html_element_is_protected(&["tbody", "tfoot", "thead"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, "tbody")
                    && !self.open_elements.has_in_table_scope(&self.arena, "thead")
                    && !self.open_elements.has_in_table_scope(&self.arena, "tfoot")
                {
                    return;
                }
                self.clear_stack_to_table_body_context();
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::InTable;
                self.process_token(token, InsertionMode::InTable);
            }
            Token::EndTag { name }
                if matches!(
                    name.as_str(),
                    "body" | "caption" | "col" | "colgroup" | "html" | "td" | "th" | "tr"
                ) =>
            {
                // Parse error, ignore
            }
            _ => {
                self.handle_in_table(token);
            }
        }
    }

    fn handle_in_row(&mut self, mut token: Token) {
        match &token {
            Token::StartTag { name, .. } if name == "th" || name == "td" => {
                self.clear_stack_to_table_row_context();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InCell;
                self.active_formatting.push_marker();
            }
            Token::EndTag { name } if name == "tr" => {
                if self.last_matching_open_html_element_is_protected(&["tr"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, "tr") {
                    return;
                }
                self.clear_stack_to_table_row_context();
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::InTableBody;
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "caption" | "col" | "colgroup" | "tbody" | "tfoot" | "thead" | "tr"
                ) =>
            {
                if self.last_matching_open_html_element_is_protected(&["tr"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, "tr") {
                    return;
                }
                self.clear_stack_to_table_row_context();
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::InTableBody;
                self.process_token(token, InsertionMode::InTableBody);
            }
            Token::EndTag { name } if name == "table" => {
                if self.last_matching_open_html_element_is_protected(&["tr"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, "tr") {
                    return;
                }
                self.clear_stack_to_table_row_context();
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::InTableBody;
                self.process_token(token, InsertionMode::InTableBody);
            }
            Token::EndTag { name } if matches!(name.as_str(), "tbody" | "tfoot" | "thead") => {
                if self.last_matching_open_html_element_is_protected(&["tr"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, name) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, "tr") {
                    return;
                }
                self.clear_stack_to_table_row_context();
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::InTableBody;
                self.process_token(token, InsertionMode::InTableBody);
            }
            Token::EndTag { name }
                if matches!(
                    name.as_str(),
                    "body" | "caption" | "col" | "colgroup" | "html" | "td" | "th"
                ) =>
            {
                // Parse error, ignore
            }
            _ => {
                self.handle_in_table(token);
            }
        }
    }

    fn handle_in_cell(&mut self, token: Token) {
        match &token {
            Token::EndTag { name } if name == "td" || name == "th" => {
                let name = name.clone();
                if !self.open_elements.has_in_table_scope(&self.arena, &name) {
                    return;
                }
                self.open_elements
                    .generate_implied_end_tags(&self.arena, None);
                self.open_elements.pop_until_html(&self.arena, &name);
                self.active_formatting.clear_up_to_last_marker();
                self.insertion_mode = InsertionMode::InRow;
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "caption"
                        | "col"
                        | "colgroup"
                        | "tbody"
                        | "td"
                        | "tfoot"
                        | "th"
                        | "thead"
                        | "tr"
                ) =>
            {
                if self.last_matching_open_html_element_is_protected(&["td", "th"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, "td")
                    && !self.open_elements.has_in_table_scope(&self.arena, "th")
                {
                    return;
                }
                self.close_cell();
                self.process_token(token, self.insertion_mode);
            }
            Token::EndTag { name }
                if matches!(
                    name.as_str(),
                    "body" | "caption" | "col" | "colgroup" | "html"
                ) =>
            {
                // Parse error, ignore
            }
            Token::EndTag { name }
                if matches!(name.as_str(), "table" | "tbody" | "tfoot" | "thead" | "tr") =>
            {
                if self.last_matching_open_html_element_is_protected(&["td", "th"]) {
                    return;
                }
                if !self.open_elements.has_in_table_scope(&self.arena, name) {
                    return;
                }
                self.close_cell();
                self.process_token(token, self.insertion_mode);
            }
            _ => {
                self.handle_in_body(token);
            }
        }
    }

    fn handle_in_select(&mut self, mut token: Token) {
        match &token {
            Token::Character { data } => {
                self.reconstruct_active_formatting();
                if *data == '\0' {
                    return;
                }
                self.insert_character(*data);
            }
            Token::Comment { data } => self.insert_comment(data),
            Token::Doctype { .. } => {}
            Token::StartTag { name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. } if name == "option" => {
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "option" {
                        self.open_elements.pop();
                    }
                }
                self.reconstruct_active_formatting();
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::StartTag { name, .. } if name == "optgroup" => {
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "option" {
                        self.open_elements.pop();
                    }
                }
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "optgroup" {
                        self.open_elements.pop();
                    }
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::EndTag { name } if name == "optgroup" => {
                // If current is option and previous is optgroup, pop option first
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "option"
                        && self.open_elements.len() >= 2
                    {
                        let prev =
                            self.open_elements.elements[self.open_elements.elements.len() - 2];
                        if self.open_elements.tag_name(&self.arena, prev) == "optgroup" {
                            self.open_elements.pop();
                        }
                    }
                }
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "optgroup" {
                        self.open_elements.pop();
                    }
                }
            }
            Token::EndTag { name } if name == "option" => {
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "option" {
                        self.open_elements.pop();
                    }
                }
            }
            Token::EndTag { name } if name == "select" => {
                if !self
                    .open_elements
                    .has_in_select_scope(&self.arena, "select")
                {
                    return;
                }
                self.open_elements.pop_until(&self.arena, "select");
                self.reset_insertion_mode();
            }
            Token::StartTag { name, .. } if name == "select" => {
                self.open_elements.pop_until(&self.arena, "select");
                self.reset_insertion_mode();
            }
            Token::StartTag { name, .. } if matches!(name.as_str(), "input" | "textarea") => {
                if self.last_matching_open_html_element_is_protected(&["select"]) {
                    self.open_elements.pop_until(&self.arena, "select");
                    self.insertion_mode = InsertionMode::InBody;
                    self.process_token(token, InsertionMode::InBody);
                    return;
                }
                if !self
                    .open_elements
                    .has_in_select_scope(&self.arena, "select")
                {
                    return;
                }
                self.open_elements.pop_until(&self.arena, "select");
                self.reset_insertion_mode();
                self.process_token(token, self.insertion_mode);
            }
            Token::StartTag { name, .. } if name == "hr" => {
                // Pop option and optgroup if open
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "option" {
                        self.open_elements.pop();
                    }
                }
                if let Some(current) = self.open_elements.current_node() {
                    if self.open_elements.tag_name(&self.arena, current) == "optgroup" {
                        self.open_elements.pop();
                    }
                }
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                // Don't push to stack — void element
            }
            Token::StartTag { name, .. } if name == "math" || name == "svg" => {
                // Relaxed select parser: math/svg create elements in their namespace
                let ns = if name == "math" {
                    Namespace::MathML
                } else {
                    Namespace::Svg
                };
                let element = self.create_foreign_element_from_token(&mut token, ns);
                self.insert_node_at_appropriate_location(element);
                if let Token::StartTag { self_closing, .. } = &token {
                    if !*self_closing {
                        self.open_elements.push(element);
                    }
                }
            }
            Token::StartTag { name, .. } if name == "script" || name == "template" => {
                self.handle_in_head(token);
            }
            Token::EndTag { name } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::EndOfFile => {
                self.handle_in_body(token);
            }
            // Relaxed <select> parser: process any other start tag using InBody rules
            Token::StartTag { .. } => {
                self.handle_in_body(token);
            }
            // For end tags, only process if the element is inside the select
            // (i.e., between the select and the top of the stack)
            Token::EndTag { name } => {
                let select_idx = self
                    .open_elements
                    .elements
                    .iter()
                    .rposition(|&id| self.open_elements.tag_name(&self.arena, id) == "select");
                if let Some(sel_idx) = select_idx {
                    let found_inside = self.open_elements.elements[sel_idx + 1..]
                        .iter()
                        .any(|&id| self.open_elements.tag_name(&self.arena, id) == name.as_str());
                    if found_inside {
                        self.handle_in_body(token);
                    }
                }
            }
        }
    }

    fn handle_in_select_in_table(&mut self, token: Token) {
        match &token {
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "caption" | "table" | "tbody" | "tfoot" | "thead" | "tr" | "td" | "th"
                ) =>
            {
                self.open_elements.pop_until(&self.arena, "select");
                self.reset_insertion_mode();
                self.process_token(token, self.insertion_mode);
            }
            Token::EndTag { name }
                if matches!(
                    name.as_str(),
                    "caption" | "table" | "tbody" | "tfoot" | "thead" | "tr" | "td" | "th"
                ) =>
            {
                if !self.open_elements.has_in_table_scope(&self.arena, name) {
                    return;
                }
                self.open_elements.pop_until(&self.arena, "select");
                self.reset_insertion_mode();
                self.process_token(token, self.insertion_mode);
            }
            _ => {
                self.handle_in_select(token);
            }
        }
    }

    fn handle_in_template(&mut self, token: Token) {
        match &token {
            Token::Character { .. } | Token::Comment { .. } | Token::Doctype { .. } => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "base"
                        | "basefont"
                        | "bgsound"
                        | "link"
                        | "meta"
                        | "noframes"
                        | "script"
                        | "style"
                        | "template"
                        | "title"
                ) =>
            {
                self.handle_in_head(token);
            }
            Token::EndTag { name } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::StartTag { name, .. }
                if matches!(
                    name.as_str(),
                    "caption" | "colgroup" | "tbody" | "tfoot" | "thead"
                ) =>
            {
                self.template_modes.pop();
                self.template_modes.push(InsertionMode::InTable);
                self.insertion_mode = InsertionMode::InTable;
                self.process_token(token, InsertionMode::InTable);
            }
            Token::StartTag { name, .. } if name == "col" => {
                self.template_modes.pop();
                self.template_modes.push(InsertionMode::InColumnGroup);
                self.insertion_mode = InsertionMode::InColumnGroup;
                self.process_token(token, InsertionMode::InColumnGroup);
            }
            Token::StartTag { name, .. } if name == "tr" => {
                self.template_modes.pop();
                self.template_modes.push(InsertionMode::InTableBody);
                self.insertion_mode = InsertionMode::InTableBody;
                self.process_token(token, InsertionMode::InTableBody);
            }
            Token::StartTag { name, .. } if name == "td" || name == "th" => {
                self.template_modes.pop();
                self.template_modes.push(InsertionMode::InRow);
                self.insertion_mode = InsertionMode::InRow;
                self.process_token(token, InsertionMode::InRow);
            }
            Token::EndOfFile => {
                if !self
                    .open_elements
                    .contains_html_tag(&self.arena, "template")
                {
                    return; // Stop parsing
                }
                self.open_elements.pop_until_html(&self.arena, "template");
                self.active_formatting.clear_up_to_last_marker();
                self.template_modes.pop();
                self.reset_insertion_mode();
                self.process_token(token, self.insertion_mode);
            }
            Token::StartTag { .. } => {
                self.template_modes.pop();
                self.template_modes.push(InsertionMode::InBody);
                self.insertion_mode = InsertionMode::InBody;
                self.process_token(token, InsertionMode::InBody);
            }
            Token::EndTag { .. } => {
                // Parse error, ignore
            }
        }
    }
    fn handle_in_frameset(&mut self, mut token: Token) {
        match &token {
            Token::Character { data } if token_is_all_whitespace(&token) => {
                self.insert_character(*data);
            }
            Token::Comment { data } => self.insert_comment(data),
            Token::StartTag { name, .. } if name == "frameset" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
                self.open_elements.push(element);
            }
            Token::EndTag { name } if name == "frameset" => {
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::AfterFrameset;
            }
            Token::StartTag { name, .. } if name == "frame" => {
                let element = self.create_element_from_token(&mut token, Namespace::Html);
                self.insert_node_at_appropriate_location(element);
            }
            Token::StartTag { name, .. } if name == "noframes" => {
                self.handle_in_head(token);
            }
            Token::EndOfFile => {}
            _ => {}
        }
    }
    fn handle_after_frameset(&mut self, token: Token) {
        match &token {
            Token::Character { data } if token_is_all_whitespace(&token) => {
                self.insert_character(*data);
            }
            Token::Comment { data } => self.insert_comment(data),
            Token::EndTag { name } if name == "html" => {
                self.insertion_mode = InsertionMode::AfterAfterFrameset;
            }
            Token::StartTag { name, .. } if name == "noframes" => {
                self.handle_in_head(token);
            }
            Token::EndOfFile => {}
            _ => {}
        }
    }
    fn handle_after_after_frameset(&mut self, token: Token) {
        match &token {
            Token::Comment { data } => {
                let comment = self.arena.create_node(NodeData::Comment {
                    content: data.clone(),
                });
                self.arena.append_child(self.document, comment);
            }
            Token::Character { .. } if token_is_all_whitespace(&token) => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. } if name == "noframes" => {
                self.handle_in_head(token);
            }
            Token::EndOfFile => {}
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Helper methods
    // -----------------------------------------------------------------------

    fn current_node_or_document(&self) -> NodeId {
        let node = self.open_elements.current_node().unwrap_or(self.document);
        // If the current node is a template, redirect to its content fragment
        self.template_content_of(node).unwrap_or(node)
    }

    fn adjusted_current_node(&self) -> Option<NodeId> {
        if self.fragment_root.is_some() && self.open_elements.len() == 1 {
            if let Some(context) = self.fragment_context_node {
                return Some(context);
            }
        }
        self.open_elements.current_node()
    }

    fn normalize_fragment_tree(&mut self) {
        let root = match self.fragment_root {
            Some(root) => root,
            None => return,
        };
        let mut to_move = Vec::new();
        let mut child = self.arena.get(self.document).first_child;
        while let Some(id) = child {
            if id != root {
                to_move.push(id);
            }
            child = self.arena.get(id).next_sibling;
        }
        for id in to_move {
            self.arena.detach(id);
            self.arena.append_child(root, id);
        }

        if let Some(context) = self.fragment_context_node {
            if context != root {
                self.flatten_fragment_context(root, context);
            }
        }

        while let Some(extra) = self.fragment_extra_context.pop() {
            if extra != root {
                self.flatten_fragment_context(root, extra);
            }
        }
    }

    /// Move the children of the synthetic fragment context onto the fragment root
    /// so serialization matches html5lib expectations (context is not part of output).
    fn flatten_fragment_context(&mut self, root: NodeId, context: NodeId) {
        let mut children = Vec::new();
        let mut child = self.arena.get(context).first_child;
        while let Some(id) = child {
            children.push(id);
            child = self.arena.get(id).next_sibling;
        }

        let after_context = self.arena.get(context).next_sibling;
        let parent = self.arena.get(context).parent.unwrap_or(root);

        for &id in &children {
            self.arena.detach(id);
        }

        if let Some(after) = after_context {
            // Insert in reverse order so the original order is preserved.
            for &id in children.iter().rev() {
                self.arena.insert_before(id, after);
            }
        } else {
            for &id in &children {
                self.arena.append_child(parent, id);
            }
        }

        self.arena.detach(context);
    }

    fn should_ignore_fragment_context_end_tag(&self, name: &str) -> bool {
        let Some(context) = self.fragment_context_node else {
            return false;
        };

        for &node in self.open_elements.elements.iter().rev() {
            if self.node_name_matches(node, name) {
                return node == context;
            }
        }
        false
    }

    fn is_fragment_protected_node(&self, node: NodeId) -> bool {
        self.fragment_root.is_some()
            && (Some(node) == self.fragment_context_node
                || self.fragment_extra_context.contains(&node))
    }

    fn last_matching_open_html_element_is_protected(&self, tags: &[&str]) -> bool {
        for &id in self.open_elements.elements.iter().rev() {
            if let NodeData::Element {
                tag_name,
                namespace,
                ..
            } = &self.arena.get(id).data
            {
                if *namespace == Namespace::Html && tags.contains(&tag_name.as_str()) {
                    return self.is_fragment_protected_node(id);
                }
            }
        }
        false
    }

    fn node_name_matches(&self, node: NodeId, name: &str) -> bool {
        if let NodeData::Element {
            tag_name,
            namespace,
            ..
        } = &self.arena.get(node).data
        {
            match namespace {
                Namespace::Html => tag_name.eq_ignore_ascii_case(name),
                _ => tag_name == name,
            }
        } else {
            false
        }
    }

    /// If node is a template element, return its content fragment child.
    fn template_content_of(&self, node: NodeId) -> Option<NodeId> {
        if let NodeData::Element {
            tag_name,
            namespace,
            ..
        } = &self.arena.get(node).data
        {
            if tag_name == "template" && *namespace == Namespace::Html {
                return self.arena.get(node).first_child;
            }
        }
        None
    }

    fn create_element_from_token(&mut self, token: &mut Token, namespace: Namespace) -> NodeId {
        if let Token::StartTag {
            name, attributes, ..
        } = token
        {
            if namespace == Namespace::Html && name == "selectedcontent" {
                self.needs_selectedcontent_population = true;
            }

            let attrs = std::mem::take(attributes)
                .into_iter()
                .map(|a| Attribute {
                    prefix: None,
                    name: a.name,
                    value: a.value,
                })
                .collect();
            self.arena.create_node(NodeData::Element {
                tag_name: std::mem::take(name),
                namespace,
                attributes: attrs,
            })
        } else {
            // Shouldn't happen, but create empty element
            self.arena.create_node(NodeData::Element {
                tag_name: String::new(),
                namespace,
                attributes: vec![],
            })
        }
    }

    fn insert_character(&mut self, ch: char) {
        if self.foster_parenting {
            // Per the spec, foster parenting applies only when the current
            // node is a table-context element.
            let should_foster = if let Some(current) = self.open_elements.current_node() {
                self.is_table_context_element(current)
            } else {
                false
            };
            if should_foster {
                match self.find_foster_parent_location() {
                    FosterLocation::InsertBefore(_parent, table) => {
                        // Check if prev sibling of table is a text node
                        let prev = self.arena.get(table).prev_sibling;
                        if let Some(prev_id) = prev {
                            if let NodeData::Text { content } =
                                &mut self.arena.get_mut(prev_id).data
                            {
                                content.push(ch);
                                return;
                            }
                        }
                        let text = self.arena.create_node(NodeData::Text {
                            content: ch.to_string(),
                        });
                        self.arena.insert_before(text, table);
                        return;
                    }
                    FosterLocation::AppendTo(parent) => {
                        // Try to append to existing last child text node
                        if let Some(last_child) = self.arena.get(parent).last_child {
                            if let NodeData::Text { content } =
                                &mut self.arena.get_mut(last_child).data
                            {
                                content.push(ch);
                                return;
                            }
                        }
                        let text = self.arena.create_node(NodeData::Text {
                            content: ch.to_string(),
                        });
                        self.arena.append_child(parent, text);
                        return;
                    }
                    FosterLocation::None => {}
                }
            }
        }

        let target = self.current_node_or_document();

        // Try to append to existing text node
        if let Some(last_child) = self.arena.get(target).last_child {
            if let NodeData::Text { content } = &mut self.arena.get_mut(last_child).data {
                content.push(ch);
                return;
            }
        }

        // Create new text node
        let text = self.arena.create_node(NodeData::Text {
            content: ch.to_string(),
        });
        self.arena.append_child(target, text);
    }

    fn insert_comment(&mut self, data: &str) {
        let comment = self.arena.create_node(NodeData::Comment {
            content: data.to_string(),
        });
        self.insert_node_at_appropriate_location(comment);
    }

    /// Find the foster parent location per §13.2.6.1.
    fn find_foster_parent_location(&self) -> FosterLocation {
        let mut last_template: Option<(usize, NodeId)> = None;
        let mut last_table: Option<(usize, NodeId)> = None;
        for (i, &id) in self.open_elements.elements.iter().enumerate().rev() {
            if let NodeData::Element {
                tag_name,
                namespace,
                ..
            } = &self.arena.get(id).data
            {
                if tag_name == "template"
                    && *namespace == Namespace::Html
                    && last_template.is_none()
                {
                    last_template = Some((i, id));
                }
                if tag_name == "table" && last_table.is_none() {
                    last_table = Some((i, id));
                }
                if last_template.is_some() && last_table.is_some() {
                    break;
                }
            }
        }
        // If last_template exists and (no last_table or last_template is lower in stack than last_table)
        if let Some((ti, template_id)) = last_template {
            if last_table.is_none() || ti > last_table.unwrap().0 {
                if let Some(content) = self.template_content_of(template_id) {
                    return FosterLocation::AppendTo(content);
                }
            }
        }
        // If last_table exists and has a parent, insert before last_table
        if let Some((_ti, table_id)) = last_table {
            if let Some(parent) = self.arena.get(table_id).parent {
                return FosterLocation::InsertBefore(parent, table_id);
            }
            // If last_table has no parent, use last_table as the parent
            return FosterLocation::AppendTo(table_id);
        }
        // No table — use the first element in the stack (html element)
        if let Some(&first) = self.open_elements.elements.first() {
            return FosterLocation::AppendTo(first);
        }
        FosterLocation::None
    }

    /// Handle a token in foreign content (§13.2.6.5).
    fn handle_foreign_content(&mut self, mut token: Token) {
        match &token {
            Token::Character { data: '\0' } => {
                self.insert_character('\u{FFFD}');
            }
            Token::Character { data } if is_whitespace(*data) => {
                self.insert_character(*data);
            }
            Token::Character { data } => {
                self.insert_character(*data);
                self.frameset_ok = false;
            }
            Token::Comment { data } => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {}
            Token::StartTag { name, .. }
                if foreign::is_html_element_that_causes_foreign_exit(name)
                    || (name == "font" && {
                        if let Token::StartTag { attributes, .. } = &token {
                            attributes
                                .iter()
                                .any(|a| a.name == "color" || a.name == "face" || a.name == "size")
                        } else {
                            false
                        }
                    }) =>
            {
                self.pop_until_html_or_integration_point();
                // Reprocess in the current mode
                self.process_token(token, self.insertion_mode);
            }
            Token::StartTag { name, .. } => {
                // svg and math tags always create elements in their own namespace;
                // other tags inherit the namespace of the adjusted current node.
                let ns = if name == "svg" {
                    Namespace::Svg
                } else if name == "math" {
                    Namespace::MathML
                } else if let Some(current) = self.adjusted_current_node() {
                    self.open_elements.namespace(&self.arena, current)
                } else {
                    Namespace::Html
                };

                let element = self.create_foreign_element_from_token(&mut token, ns);
                self.insert_node_at_appropriate_location(element);

                if let Token::StartTag { self_closing, .. } = &token {
                    if !*self_closing {
                        self.open_elements.push(element);
                    }
                }
            }
            Token::EndTag { name } if name == "br" || name == "p" => {
                // Parse error. These end tags break out of foreign content.
                self.pop_until_html_or_integration_point();
                // Reprocess as end tag in the current mode
                let name = name.clone();
                self.process_token(Token::EndTag { name }, self.insertion_mode);
            }
            Token::EndTag { name } => {
                let name = name.clone();
                // Walk the stack backwards looking for matching element
                if self.open_elements.is_empty() {
                    return;
                }
                let mut i = self.open_elements.elements.len() - 1;
                let node = self.open_elements.elements[i];
                let tag = self
                    .open_elements
                    .tag_name(&self.arena, node)
                    .to_ascii_lowercase();
                if tag != name {
                    // Parse error
                }
                loop {
                    let node = self.open_elements.elements[i];
                    let tag = self
                        .open_elements
                        .tag_name(&self.arena, node)
                        .to_ascii_lowercase();
                    if tag == name {
                        while self.open_elements.elements.len() > i {
                            self.open_elements.pop();
                        }
                        return;
                    }
                    if i == 0 {
                        return;
                    }
                    i -= 1;
                    let ns = self
                        .open_elements
                        .namespace(&self.arena, self.open_elements.elements[i]);
                    if ns == Namespace::Html {
                        // Process as in current insertion mode
                        self.process_token(
                            Token::EndTag { name: name.clone() },
                            self.insertion_mode,
                        );
                        return;
                    }
                }
            }
            Token::EndOfFile => {
                self.process_token(token, self.insertion_mode);
            }
        }
    }

    fn create_foreign_element_from_token(
        &mut self,
        token: &mut Token,
        namespace: Namespace,
    ) -> NodeId {
        if let Token::StartTag {
            name, attributes, ..
        } = token
        {
            let adjusted_name = match namespace {
                Namespace::Svg => {
                    let adjusted = foreign::adjust_svg_tag_name(name);
                    if adjusted == name.as_str() {
                        std::mem::take(name)
                    } else {
                        adjusted.to_string()
                    }
                }
                _ => std::mem::take(name),
            };

            let mut attrs = Vec::with_capacity(attributes.len());
            for attr in std::mem::take(attributes) {
                let TokenAttribute { name, value } = attr;
                let adjusted = match namespace {
                    Namespace::Svg => foreign::adjust_svg_attributes(&name),
                    Namespace::MathML => foreign::adjust_mathml_attributes(&name),
                    _ => name.as_str(),
                };
                // Handle foreign attribute namespaces
                let (prefix, local_name) = adjust_foreign_attribute(adjusted);
                let reuse_name = prefix.is_none() && local_name == name.as_str();
                attrs.push(Attribute {
                    prefix,
                    name: if reuse_name {
                        name
                    } else {
                        local_name.to_string()
                    },
                    value,
                });
            }

            self.arena.create_node(NodeData::Element {
                tag_name: adjusted_name,
                namespace,
                attributes: attrs,
            })
        } else {
            self.arena.create_node(NodeData::Element {
                tag_name: String::new(),
                namespace,
                attributes: vec![],
            })
        }
    }

    fn current_node_is_one_of(&self, tags: &[&str]) -> bool {
        if let Some(current) = self.open_elements.current_node() {
            let tag = self.open_elements.tag_name(&self.arena, current);
            tags.contains(&tag)
        } else {
            false
        }
    }

    fn clear_stack_to_table_context(&mut self) {
        while let Some(current) = self.open_elements.current_node() {
            let tag = self.open_elements.tag_name(&self.arena, current);
            if matches!(tag, "table" | "template" | "html") {
                break;
            }
            self.open_elements.pop();
        }
    }

    fn clear_stack_to_table_body_context(&mut self) {
        while let Some(current) = self.open_elements.current_node() {
            let tag = self.open_elements.tag_name(&self.arena, current);
            if matches!(tag, "tbody" | "tfoot" | "thead" | "template" | "html") {
                break;
            }
            self.open_elements.pop();
        }
    }

    fn clear_stack_to_table_row_context(&mut self) {
        while let Some(current) = self.open_elements.current_node() {
            let tag = self.open_elements.tag_name(&self.arena, current);
            if matches!(tag, "tr" | "template" | "html") {
                break;
            }
            self.open_elements.pop();
        }
    }

    fn close_cell(&mut self) {
        self.open_elements
            .generate_implied_end_tags(&self.arena, None);
        // Pop until td or th
        while let Some(id) = self.open_elements.pop() {
            let tag = self.open_elements.tag_name(&self.arena, id);
            let _ = tag;
            if let NodeData::Element { tag_name, .. } = &self.arena.get(id).data {
                if tag_name == "td" || tag_name == "th" {
                    break;
                }
            }
        }
        self.active_formatting.clear_up_to_last_marker();
        self.insertion_mode = InsertionMode::InRow;
    }

    /// Insert a node at the appropriate location (handling foster parenting).
    /// Per spec §13.2.6.1, when foster parenting is active, find the appropriate
    /// foster parent (before the table or in template content) instead of the
    /// current node.
    fn insert_node_at_appropriate_location(&mut self, node: NodeId) {
        if self.foster_parenting {
            // If the current node was itself foster-parented (not inside a table
            // Per the spec, foster parenting applies when the target
            // (current node) is table, tbody, tfoot, thead, or tr.
            let should_foster = if let Some(current) = self.open_elements.current_node() {
                self.is_table_context_element(current)
            } else {
                false
            };
            if should_foster {
                match self.find_foster_parent_location() {
                    FosterLocation::InsertBefore(_parent, table) => {
                        self.arena.insert_before(node, table);
                        return;
                    }
                    FosterLocation::AppendTo(parent) => {
                        self.arena.append_child(parent, node);
                        return;
                    }
                    FosterLocation::None => {}
                }
            }
        }
        let target = self.current_node_or_document();
        self.arena.append_child(target, node);
    }

    /// Check if a node is a table-context element where foster parenting applies.
    /// Per the spec, these are: table, tbody, tfoot, thead, tr.
    fn is_table_context_element(&self, node: NodeId) -> bool {
        if let NodeData::Element {
            tag_name,
            namespace,
            ..
        } = &self.arena.get(node).data
        {
            *namespace == Namespace::Html
                && matches!(
                    tag_name.as_str(),
                    "table" | "tbody" | "thead" | "tfoot" | "tr"
                )
        } else {
            false
        }
    }

    fn has_table_in_scope_or_template(&self) -> bool {
        self.open_elements.has_in_table_scope(&self.arena, "table")
            || self
                .open_elements
                .contains_html_tag(&self.arena, "template")
    }

    fn pop_until_html_or_integration_point(&mut self) {
        loop {
            let Some(current) = self.open_elements.current_node() else {
                return;
            };
            let ns = self.open_elements.namespace(&self.arena, current);
            if ns == Namespace::Html {
                return;
            }
            let tag = self.open_elements.tag_name(&self.arena, current);
            if ns == Namespace::MathML && foreign::is_mathml_text_integration_point(tag) {
                return;
            }
            if ns == Namespace::Svg && foreign::is_html_integration_point_svg(tag) {
                return;
            }
            self.open_elements.pop();
        }
    }

    fn close_p_element(&mut self) {
        self.open_elements
            .generate_implied_end_tags(&self.arena, Some("p"));
        self.open_elements.pop_until(&self.arena, "p");
    }

    fn reconstruct_active_formatting(&mut self) {
        use super::active::ActiveFormattingEntry;

        if self.active_formatting.entries.is_empty() {
            return;
        }

        // Check if the last entry is a marker or already in the stack
        match self.active_formatting.entries.last() {
            Some(ActiveFormattingEntry::Marker) => return,
            Some(ActiveFormattingEntry::Element(id)) => {
                if self.open_elements.elements.contains(id) {
                    return;
                }
            }
            None => return,
        }

        let mut i = self.active_formatting.entries.len() - 1;

        // Step back until we find a marker or an element in the stack
        loop {
            if i == 0 {
                break;
            }
            i -= 1;
            match &self.active_formatting.entries[i] {
                ActiveFormattingEntry::Marker => {
                    i += 1;
                    break;
                }
                ActiveFormattingEntry::Element(id) => {
                    if self.open_elements.elements.contains(id) {
                        i += 1;
                        break;
                    }
                }
            }
        }

        // Now advance forward, creating new elements
        while i < self.active_formatting.entries.len() {
            if let ActiveFormattingEntry::Element(old_id) = &self.active_formatting.entries[i] {
                let old_id = *old_id;
                // Create a new element with the same tag/namespace/attributes
                let new_id = if let Some(snapshot) = self.active_formatting.snapshot(old_id) {
                    self.arena.create_node(NodeData::Element {
                        tag_name: snapshot.tag_name.clone(),
                        namespace: snapshot.namespace,
                        attributes: snapshot.attributes.clone(),
                    })
                } else if let NodeData::Element {
                    tag_name,
                    namespace,
                    attributes,
                } = &self.arena.get(old_id).data
                {
                    self.arena.create_node(NodeData::Element {
                        tag_name: tag_name.clone(),
                        namespace: *namespace,
                        attributes: attributes.clone(),
                    })
                } else {
                    i += 1;
                    continue;
                };

                // Insert into DOM using appropriate location (respects foster parenting)
                self.insert_node_at_appropriate_location(new_id);
                self.open_elements.push(new_id);

                // Replace in active formatting list
                self.active_formatting.entries[i] = ActiveFormattingEntry::Element(new_id);
            }
            i += 1;
        }
    }

    /// The adoption agency algorithm (§13.2.6.4.7).
    fn run_adoption_agency(&mut self, tag: &str) {
        if let Some(current) = self.open_elements.current_node() {
            if self.open_elements.tag_name(&self.arena, current) == tag
                && !self.active_formatting.contains(current)
            {
                self.open_elements.pop();
                return;
            }
        }

        for _ in 0..8 {
            let (fmt_idx_in_afl, formatting_element) =
                match self.active_formatting.find_by_tag(&self.arena, tag) {
                    Some(x) => x,
                    None => {
                        self.handle_any_other_end_tag(tag);
                        return;
                    }
                };

            let fmt_idx_in_stack = match self
                .open_elements
                .elements
                .iter()
                .position(|&id| id == formatting_element)
            {
                Some(idx) => idx,
                None => {
                    self.active_formatting.remove(formatting_element);
                    return;
                }
            };

            if !self.open_elements.has_in_scope(&self.arena, tag) {
                return;
            }

            let furthest_block = self.open_elements.elements[fmt_idx_in_stack + 1..]
                .iter()
                .find(|&&id| {
                    let tag = self.open_elements.tag_name(&self.arena, id);
                    let ns = self.open_elements.namespace(&self.arena, id);
                    is_special_element(tag, ns)
                })
                .copied();

            let furthest_block = match furthest_block {
                Some(fb) => fb,
                None => {
                    while self.open_elements.current_node() != Some(formatting_element) {
                        self.open_elements.pop();
                    }
                    self.open_elements.pop();
                    self.active_formatting.remove(formatting_element);
                    return;
                }
            };

            let common_ancestor = self.open_elements.elements[fmt_idx_in_stack - 1];
            let mut bookmark = fmt_idx_in_afl;

            let mut last_node = furthest_block;
            let mut node_idx = self
                .open_elements
                .elements
                .iter()
                .position(|&id| id == furthest_block)
                .unwrap();
            let mut inner_count = 0u32;

            loop {
                inner_count += 1;
                if node_idx == 0 {
                    break;
                }
                node_idx -= 1;
                let node = self.open_elements.elements[node_idx];

                if node == formatting_element {
                    break;
                }

                if inner_count > 3 && self.active_formatting.contains(node) {
                    self.active_formatting.remove(node);
                }

                if !self.active_formatting.contains(node) {
                    self.open_elements.elements.remove(node_idx);
                    continue;
                }

                let new_element = if let Some(snapshot) = self.active_formatting.snapshot(node) {
                    self.arena.create_node(NodeData::Element {
                        tag_name: snapshot.tag_name.clone(),
                        namespace: snapshot.namespace,
                        attributes: snapshot.attributes.clone(),
                    })
                } else if let NodeData::Element {
                    tag_name,
                    namespace,
                    attributes,
                } = &self.arena.get(node).data
                {
                    self.arena.create_node(NodeData::Element {
                        tag_name: tag_name.clone(),
                        namespace: *namespace,
                        attributes: attributes.clone(),
                    })
                } else {
                    break;
                };

                if let Some(afl_pos) = self.active_formatting.entries.iter().position(
                    |e| matches!(e, super::active::ActiveFormattingEntry::Element(id) if *id == node),
                ) {
                    self.active_formatting.replace(afl_pos, new_element);
                }

                self.open_elements.elements[node_idx] = new_element;
                let node = new_element;

                if last_node == furthest_block {
                    bookmark = self
                        .active_formatting
                        .entries
                        .iter()
                        .position(|e| {
                            matches!(e, super::active::ActiveFormattingEntry::Element(id) if *id == new_element)
                        })
                        .map(|p| p + 1)
                        .unwrap_or(bookmark);
                }

                self.arena.detach(last_node);
                self.arena.append_child(node, last_node);
                last_node = node;
            }

            self.arena.detach(last_node);
            let ca_tag = self.open_elements.tag_name(&self.arena, common_ancestor);
            let ca_ns = self.open_elements.namespace(&self.arena, common_ancestor);
            if ca_ns == Namespace::Html
                && matches!(ca_tag, "table" | "tbody" | "tfoot" | "thead" | "tr")
            {
                match self.find_foster_parent_location() {
                    FosterLocation::InsertBefore(_parent, table) => {
                        self.arena.insert_before(last_node, table);
                    }
                    FosterLocation::AppendTo(parent) => {
                        self.arena.append_child(parent, last_node);
                    }
                    FosterLocation::None => {
                        self.arena.append_child(common_ancestor, last_node);
                    }
                }
            } else {
                self.arena.append_child(common_ancestor, last_node);
            }

            let new_element = if let NodeData::Element {
                tag_name,
                namespace,
                attributes,
            } = &self.arena.get(formatting_element).data
            {
                self.arena.create_node(NodeData::Element {
                    tag_name: tag_name.clone(),
                    namespace: *namespace,
                    attributes: attributes.clone(),
                })
            } else {
                return;
            };

            while let Some(child) = self.arena.get(furthest_block).first_child {
                self.arena.detach(child);
                self.arena.append_child(new_element, child);
            }

            self.arena.append_child(furthest_block, new_element);

            self.active_formatting.remove(formatting_element);
            if bookmark > self.active_formatting.entries.len() {
                bookmark = self.active_formatting.entries.len();
            }
            self.active_formatting.insert(bookmark, new_element);

            self.open_elements.remove(formatting_element);
            let fb_pos = self
                .open_elements
                .elements
                .iter()
                .position(|&id| id == furthest_block)
                .unwrap_or(self.open_elements.elements.len() - 1);
            self.open_elements.elements.insert(fb_pos + 1, new_element);
        }
    }

    /// "Any other end tag" logic for InBody.
    fn handle_any_other_end_tag(&mut self, name: &str) {
        for i in (0..self.open_elements.elements.len()).rev() {
            let id = self.open_elements.elements[i];
            let tag = self.open_elements.tag_name(&self.arena, id);
            if tag == name {
                self.open_elements
                    .generate_implied_end_tags(&self.arena, Some(name));
                while self.open_elements.elements.len() > i {
                    self.open_elements.pop();
                }
                break;
            }
            if is_special(tag) {
                break;
            }
        }
    }

    fn transfer_attributes(&mut self, target: NodeId, new_attrs: &[TokenAttribute]) {
        if let NodeData::Element { attributes, .. } = &mut self.arena.get_mut(target).data {
            for attr in new_attrs {
                if !attributes.iter().any(|a| a.name == attr.name) {
                    attributes.push(Attribute {
                        prefix: None,
                        name: attr.name.clone(),
                        value: attr.value.clone(),
                    });
                }
            }
        }
    }

    /// Generic parsing for RCDATA elements (title, textarea)
    fn parse_generic_rcdata(&mut self, token: &mut Token) {
        let element = self.create_element_from_token(token, Namespace::Html);
        self.insert_node_at_appropriate_location(element);
        self.open_elements.push(element);
        self.tokenizer.set_state(State::RcData);
        self.original_insertion_mode = Some(self.insertion_mode);
        self.insertion_mode = InsertionMode::Text;
    }

    /// Generic parsing for RAWTEXT elements (style, xmp, iframe, noembed, noframes)
    fn parse_generic_raw_text(&mut self, token: &mut Token) {
        let element = self.create_element_from_token(token, Namespace::Html);
        self.insert_node_at_appropriate_location(element);
        self.open_elements.push(element);
        self.tokenizer.set_state(State::RawText);
        self.original_insertion_mode = Some(self.insertion_mode);
        self.insertion_mode = InsertionMode::Text;
    }

    /// Reset the insertion mode appropriately.
    fn reset_insertion_mode(&mut self) {
        for i in (0..self.open_elements.elements.len()).rev() {
            let node = self.open_elements.elements[i];
            let last = i == 0;
            let mut tag = self.open_elements.tag_name(&self.arena, node).to_string();
            let mut treat_as_fragment_context = false;
            if last {
                if let Some(context) = self.fragment_context_node {
                    if let NodeData::Element { tag_name, .. } = &self.arena.get(context).data {
                        tag = tag_name.clone();
                        treat_as_fragment_context = true;
                    }
                } else if let Some(context) = &self.fragment_context {
                    tag = context.tag_name.clone();
                    treat_as_fragment_context = true;
                }
            }

            match tag.as_str() {
                "select" => {
                    // Per spec: walk ancestors to check for table (InSelectInTable)
                    // Stop at template or first element.
                    if !last {
                        let mut found_table = false;
                        for j in (0..i).rev() {
                            let ancestor = self.open_elements.elements[j];
                            let atag = self.open_elements.tag_name(&self.arena, ancestor);
                            if atag == "template" {
                                break;
                            }
                            if atag == "table" {
                                found_table = true;
                                break;
                            }
                        }
                        if found_table {
                            self.insertion_mode = InsertionMode::InSelectInTable;
                            return;
                        }
                    }
                    self.insertion_mode = InsertionMode::InSelect;
                    return;
                }
                "td" | "th" if !last || treat_as_fragment_context => {
                    self.insertion_mode = InsertionMode::InCell;
                    return;
                }
                "tr" => {
                    self.insertion_mode = InsertionMode::InRow;
                    return;
                }
                "tbody" | "thead" | "tfoot" => {
                    self.insertion_mode = InsertionMode::InTableBody;
                    return;
                }
                "caption" => {
                    self.insertion_mode = InsertionMode::InCaption;
                    return;
                }
                "colgroup" => {
                    self.insertion_mode = InsertionMode::InColumnGroup;
                    return;
                }
                "table" => {
                    self.insertion_mode = InsertionMode::InTable;
                    return;
                }
                "template" => {
                    self.insertion_mode = *self
                        .template_modes
                        .last()
                        .unwrap_or(&InsertionMode::InTemplate);
                    return;
                }
                "head" if !last || treat_as_fragment_context => {
                    self.insertion_mode = InsertionMode::InHead;
                    return;
                }
                "body" => {
                    self.insertion_mode = InsertionMode::InBody;
                    return;
                }
                "frameset" => {
                    self.insertion_mode = InsertionMode::InFrameset;
                    return;
                }
                "html" => {
                    if self.head_pointer.is_none() {
                        self.insertion_mode = InsertionMode::BeforeHead;
                    } else {
                        self.insertion_mode = InsertionMode::AfterHead;
                    }
                    return;
                }
                _ => {}
            }

            if last {
                self.insertion_mode = InsertionMode::InBody;
                return;
            }
        }
    }

    /// Post-parse step: populate <selectedcontent> elements inside <select>.
    /// The <selectedcontent> element (inside a <button> within a customizable <select>)
    /// should contain a deep clone of the selected <option>'s children.
    fn populate_selectedcontent(&mut self) {
        let mut selects = Vec::new();
        self.find_elements_by_tag(self.document, "select", &mut selects);

        for select_id in selects {
            // Find the <button> child of <select> that contains <selectedcontent>
            let mut selectedcontent_id = None;
            let mut child = self.arena.get(select_id).first_child;
            while let Some(cid) = child {
                if let NodeData::Element { tag_name, .. } = &self.arena.get(cid).data {
                    if tag_name == "button" {
                        // Look for <selectedcontent> inside this button
                        let mut bc = self.arena.get(cid).first_child;
                        while let Some(bcid) = bc {
                            if let NodeData::Element { tag_name: t, .. } =
                                &self.arena.get(bcid).data
                            {
                                if t == "selectedcontent" {
                                    selectedcontent_id = Some(bcid);
                                    break;
                                }
                            }
                            bc = self.arena.get(bcid).next_sibling;
                        }
                    }
                }
                child = self.arena.get(cid).next_sibling;
            }

            let sc_id = match selectedcontent_id {
                Some(id) => id,
                None => continue,
            };

            // Find the selected option: first with `selected` attr, or first option
            let mut selected_option = None;
            let mut first_option = None;
            let mut child = self.arena.get(select_id).first_child;
            while let Some(cid) = child {
                if let NodeData::Element {
                    tag_name,
                    attributes,
                    ..
                } = &self.arena.get(cid).data
                {
                    if tag_name == "option" {
                        if first_option.is_none() {
                            first_option = Some(cid);
                        }
                        if attributes.iter().any(|a| a.name == "selected") {
                            selected_option = Some(cid);
                        }
                    }
                }
                child = self.arena.get(cid).next_sibling;
            }

            let option_id = match selected_option.or(first_option) {
                Some(id) => id,
                None => continue,
            };

            // Deep clone the option's children and append to <selectedcontent>
            let mut src_child = self.arena.get(option_id).first_child;
            while let Some(src_id) = src_child {
                let cloned = self.deep_clone(src_id);
                self.arena.append_child(sc_id, cloned);
                src_child = self.arena.get(src_id).next_sibling;
            }
        }
    }

    /// Find all elements with a given tag name in the subtree rooted at `node`.
    fn find_elements_by_tag(&self, node: NodeId, tag: &str, result: &mut Vec<NodeId>) {
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            if let NodeData::Element { tag_name, .. } = &self.arena.get(current).data {
                if tag_name == tag {
                    result.push(current);
                }
            }

            let mut children = Vec::new();
            let mut child = self.arena.get(current).first_child;
            while let Some(cid) = child {
                children.push(cid);
                child = self.arena.get(cid).next_sibling;
            }

            for cid in children.into_iter().rev() {
                stack.push(cid);
            }
        }
    }

    /// Deep clone a node and all its descendants, returning the new root NodeId.
    fn deep_clone(&mut self, node: NodeId) -> NodeId {
        let root_clone = self.arena.create_node(self.arena.get(node).data.clone());
        let mut stack = vec![(node, root_clone)];

        while let Some((src, dst)) = stack.pop() {
            let mut children = Vec::new();
            let mut child = self.arena.get(src).first_child;
            while let Some(cid) = child {
                children.push(cid);
                child = self.arena.get(cid).next_sibling;
            }

            for cid in children {
                let cloned_child = self.arena.create_node(self.arena.get(cid).data.clone());
                self.arena.append_child(dst, cloned_child);
                stack.push((cid, cloned_child));
            }
        }

        root_clone
    }

    fn execute_script(&mut self, script_node: NodeId) {
        if !self.scripting_enabled {
            return;
        }

        let NodeData::Element {
            tag_name,
            namespace,
            ..
        } = &self.arena.get(script_node).data
        else {
            return;
        };
        if tag_name != "script" || *namespace != Namespace::Html {
            return;
        }

        let source = self.node_text_content(script_node);
        let source = source.trim();
        if source.is_empty() {
            return;
        }

        if let Some((target_id, new_id)) = parse_get_element_id_assignment(source) {
            if let Some(node) = self.find_element_by_id(self.document, &target_id) {
                self.set_element_attribute(node, "id", &new_id);
            }
            return;
        }

        if let Some((tag_name, index, attr_name, attr_value)) =
            parse_get_elements_by_tag_attribute_assignment(source)
        {
            let mut nodes = Vec::new();
            self.find_elements_by_tag(self.document, &tag_name, &mut nodes);
            if let Some(&node) = nodes.get(index) {
                self.set_element_attribute(node, &attr_name, &attr_value);
            }
            return;
        }

        if let Some(html) = parse_document_write(source) {
            self.tokenizer.input.insert_html_at_current_position(html);
        }
    }

    fn node_text_content(&self, node: NodeId) -> String {
        let mut content = String::new();
        self.collect_text_content(node, &mut content);
        content
    }

    fn collect_text_content(&self, node: NodeId, content: &mut String) {
        match &self.arena.get(node).data {
            NodeData::Text { content: text } => content.push_str(text),
            _ => {
                let mut child = self.arena.get(node).first_child;
                while let Some(cid) = child {
                    self.collect_text_content(cid, content);
                    child = self.arena.get(cid).next_sibling;
                }
            }
        }
    }

    fn find_element_by_id(&self, node: NodeId, id: &str) -> Option<NodeId> {
        if let NodeData::Element { attributes, .. } = &self.arena.get(node).data {
            if attributes
                .iter()
                .any(|attr| attr.name == "id" && attr.value == id)
            {
                return Some(node);
            }
        }

        let mut child = self.arena.get(node).first_child;
        while let Some(cid) = child {
            if let Some(found) = self.find_element_by_id(cid, id) {
                return Some(found);
            }
            child = self.arena.get(cid).next_sibling;
        }

        None
    }

    fn set_element_attribute(&mut self, node: NodeId, name: &str, value: &str) {
        let NodeData::Element { attributes, .. } = &mut self.arena.get_mut(node).data else {
            return;
        };

        if let Some(attr) = attributes.iter_mut().find(|attr| attr.name == name) {
            attr.value = value.to_string();
            return;
        }

        attributes.push(Attribute {
            prefix: None,
            name: name.to_string(),
            value: value.to_string(),
        });
    }
}

// -----------------------------------------------------------------------
// Free functions
// -----------------------------------------------------------------------

fn is_whitespace(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\x0C' | ' ')
}

fn token_is_all_whitespace(token: &Token) -> bool {
    matches!(token, Token::Character { data } if is_whitespace(*data))
}

fn parse_document_write(script: &str) -> Option<String> {
    let rest = script.trim().strip_prefix("document.write(")?;
    let rest = rest.trim_end();
    let rest = if let Some(rest) = rest.strip_suffix(';') {
        rest.trim_end()
    } else {
        rest
    };
    let expr = rest.strip_suffix(')')?.trim();
    parse_js_string_concat(expr)
}

fn parse_get_element_id_assignment(script: &str) -> Option<(String, String)> {
    let rest = script.trim().strip_prefix("document.getElementById(")?;
    let (target_id, rest) = parse_js_string_literal(rest.trim_start())?;
    let rest = rest.trim_start().strip_prefix(')')?.trim_start();
    let rest = rest.strip_prefix(".id")?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let (new_id, rest) = parse_js_string_literal(rest)?;
    if script_tail_is_empty(rest) {
        Some((target_id, new_id))
    } else {
        None
    }
}

fn parse_get_elements_by_tag_attribute_assignment(
    script: &str,
) -> Option<(String, usize, String, String)> {
    let rest = script
        .trim()
        .strip_prefix("document.getElementsByTagName(")?;
    let (tag_name, rest) = parse_js_string_literal(rest.trim_start())?;
    let rest = rest.trim_start().strip_prefix(')')?.trim_start();
    let rest = rest.strip_prefix('[')?;
    let index_end = rest.find(']')?;
    let index = rest[..index_end].trim().parse().ok()?;
    let rest = rest[index_end + 1..].trim_start();
    let rest = rest.strip_prefix(".setAttribute(")?.trim_start();
    let (attr_name, rest) = parse_js_string_literal(rest)?;
    let rest = rest.trim_start().strip_prefix(',')?.trim_start();
    let (attr_value, rest) = parse_js_string_literal(rest)?;
    let rest = rest.trim_start().strip_prefix(')')?;
    if script_tail_is_empty(rest) {
        Some((tag_name, index, attr_name, attr_value))
    } else {
        None
    }
}

fn parse_js_string_concat(expr: &str) -> Option<String> {
    let mut rest = expr.trim();
    let mut result = String::new();

    loop {
        let (part, next) = parse_js_string_literal(rest)?;
        result.push_str(&part);
        rest = next.trim_start();
        if rest.is_empty() {
            return Some(result);
        }
        rest = rest.strip_prefix('+')?.trim_start();
    }
}

fn parse_js_string_literal(input: &str) -> Option<(String, &str)> {
    let mut chars = input.char_indices();
    let (_, quote) = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;

    for (idx, ch) in chars {
        if escaped {
            let unescaped = match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                other => other,
            };
            value.push(unescaped);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == quote {
            return Some((value, &input[idx + ch.len_utf8()..]));
        }

        value.push(ch);
    }

    None
}

fn script_tail_is_empty(rest: &str) -> bool {
    let rest = rest.trim();
    rest.is_empty() || rest == ";"
}

/// Adjust foreign attributes: split xlink:href → (Some("xlink"), "href"), etc.
fn adjust_foreign_attribute(name: &str) -> (Option<String>, &str) {
    match name {
        "xlink:actuate" => (Some("xlink".to_string()), "actuate"),
        "xlink:arcrole" => (Some("xlink".to_string()), "arcrole"),
        "xlink:href" => (Some("xlink".to_string()), "href"),
        "xlink:role" => (Some("xlink".to_string()), "role"),
        "xlink:show" => (Some("xlink".to_string()), "show"),
        "xlink:title" => (Some("xlink".to_string()), "title"),
        "xlink:type" => (Some("xlink".to_string()), "type"),
        "xml:lang" => (Some("xml".to_string()), "lang"),
        "xml:space" => (Some("xml".to_string()), "space"),
        "xmlns" => (None, "xmlns"),
        "xmlns:xlink" => (Some("xmlns".to_string()), "xlink"),
        _ => (None, name),
    }
}

fn matches_table_fragment_tag(name: &str) -> bool {
    matches!(
        name,
        "caption" | "col" | "colgroup" | "table" | "tbody" | "td" | "tfoot" | "th" | "thead" | "tr"
    )
}

fn is_formatting_element(tag: &str) -> bool {
    matches!(
        tag,
        "b" | "big"
            | "code"
            | "em"
            | "font"
            | "i"
            | "s"
            | "small"
            | "strike"
            | "strong"
            | "tt"
            | "u"
    )
}

fn is_special(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "applet"
            | "area"
            | "article"
            | "aside"
            | "base"
            | "basefont"
            | "bgsound"
            | "blockquote"
            | "body"
            | "br"
            | "button"
            | "caption"
            | "center"
            | "col"
            | "colgroup"
            | "dd"
            | "details"
            | "dir"
            | "div"
            | "dl"
            | "dt"
            | "embed"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "frame"
            | "frameset"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "head"
            | "header"
            | "hgroup"
            | "hr"
            | "html"
            | "iframe"
            | "img"
            | "input"
            | "keygen"
            | "li"
            | "link"
            | "listing"
            | "main"
            | "marquee"
            | "menu"
            | "meta"
            | "nav"
            | "noembed"
            | "noframes"
            | "noscript"
            | "object"
            | "ol"
            | "p"
            | "param"
            | "plaintext"
            | "pre"
            | "script"
            | "search"
            | "section"
            | "select"
            | "source"
            | "style"
            | "summary"
            | "table"
            | "tbody"
            | "td"
            | "template"
            | "textarea"
            | "tfoot"
            | "th"
            | "thead"
            | "title"
            | "tr"
            | "track"
            | "ul"
            | "wbr"
            | "xmp"
    )
}

/// Check if an element (considering namespace) is a "special" element per the spec.
fn is_special_element(tag: &str, namespace: Namespace) -> bool {
    match namespace {
        Namespace::Html => is_special(tag),
        Namespace::MathML => matches!(tag, "mi" | "mo" | "mn" | "ms" | "mtext" | "annotation-xml"),
        Namespace::Svg => matches!(tag, "foreignObject" | "desc" | "title"),
    }
}

/// Determine the quirks mode from a DOCTYPE token.
fn determine_quirks_mode(
    name: Option<&str>,
    public_id: Option<&str>,
    system_id: Option<&str>,
    force_quirks: bool,
) -> QuirksMode {
    if force_quirks {
        return QuirksMode::Quirks;
    }

    if name != Some("html") {
        return QuirksMode::Quirks;
    }

    let pub_id = public_id.unwrap_or("");
    let sys_id = system_id.unwrap_or("");
    let pub_lower = pub_id.to_ascii_lowercase();

    // Quirks mode system identifier
    if sys_id == "http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd" {
        return QuirksMode::Quirks;
    }

    // Quirks mode public identifiers
    const QUIRKS_PUBLIC_IDS: &[&str] = &[
        "-//w3o//dtd w3 html strict 3.0//en//",
        "-/w3c/dtd html 4.0 transitional/en",
        "html",
    ];

    for &qpid in QUIRKS_PUBLIC_IDS {
        if pub_lower == qpid {
            return QuirksMode::Quirks;
        }
    }

    const QUIRKS_PUBLIC_PREFIXES: &[&str] = &[
        "+//silmaril//dtd html pro v0r11 19970101//",
        "-//as//dtd html 3.0 aswedit 5.0//",
        "-//advasoft ltd//dtd html 3.0 aswedit 5.0//",
        "-//ietf//dtd html 2.0 level 1//",
        "-//ietf//dtd html 2.0 level 2//",
        "-//ietf//dtd html 2.0 strict level 1//",
        "-//ietf//dtd html 2.0 strict level 2//",
        "-//ietf//dtd html 2.0 strict//",
        "-//ietf//dtd html 2.0//",
        "-//ietf//dtd html 2.1e//",
        "-//ietf//dtd html 3.0//",
        "-//ietf//dtd html 3.2 final//",
        "-//ietf//dtd html 3.2//",
        "-//ietf//dtd html 3//",
        "-//ietf//dtd html level 0//",
        "-//ietf//dtd html level 1//",
        "-//ietf//dtd html level 2//",
        "-//ietf//dtd html level 3//",
        "-//ietf//dtd html strict level 0//",
        "-//ietf//dtd html strict level 1//",
        "-//ietf//dtd html strict level 2//",
        "-//ietf//dtd html strict level 3//",
        "-//ietf//dtd html strict//",
        "-//ietf//dtd html//",
        "-//metrius//dtd metrius presentational//",
        "-//microsoft//dtd internet explorer 2.0 html strict//",
        "-//microsoft//dtd internet explorer 2.0 html//",
        "-//microsoft//dtd internet explorer 2.0 tables//",
        "-//microsoft//dtd internet explorer 3.0 html strict//",
        "-//microsoft//dtd internet explorer 3.0 html//",
        "-//microsoft//dtd internet explorer 3.0 tables//",
        "-//netscape comm. corp.//dtd html//",
        "-//netscape comm. corp.//dtd strict html//",
        "-//o'reilly and associates//dtd html 2.0//",
        "-//o'reilly and associates//dtd html extended 1.0//",
        "-//o'reilly and associates//dtd html extended relaxed 1.0//",
        "-//sq//dtd html 2.0 hotmetal + extensions//",
        "-//softquad software//dtd hotmetal pro 6.0::19990601::extensions to html 4.0//",
        "-//softquad//dtd hotmetal pro 4.0::19971010::extensions to html 4.0//",
        "-//spyglass//dtd html 2.0 extended//",
        "-//sun microsystems corp.//dtd hotjava html//",
        "-//sun microsystems corp.//dtd hotjava strict html//",
        "-//w3c//dtd html 3 1995-03-24//",
        "-//w3c//dtd html 3.2 draft//",
        "-//w3c//dtd html 3.2 final//",
        "-//w3c//dtd html 3.2//",
        "-//w3c//dtd html 3.2s draft//",
        "-//w3c//dtd html 4.0 frameset//",
        "-//w3c//dtd html 4.0 transitional//",
        "-//w3c//dtd html experimental 19960712//",
        "-//w3c//dtd html experimental 970421//",
        "-//w3c//dtd w3 html//",
        "-//w3o//dtd w3 html 3.0//",
        "-//webtechs//dtd mozilla html 2.0//",
        "-//webtechs//dtd mozilla html//",
    ];

    for prefix in QUIRKS_PUBLIC_PREFIXES {
        if pub_lower.starts_with(prefix) {
            return QuirksMode::Quirks;
        }
    }

    // System identifier missing triggers quirks for some public IDs
    if system_id.is_none() {
        const QUIRKS_NO_SYSTEM: &[&str] = &[
            "-//w3c//dtd html 4.01 frameset//",
            "-//w3c//dtd html 4.01 transitional//",
        ];
        for prefix in QUIRKS_NO_SYSTEM {
            if pub_lower.starts_with(prefix) {
                return QuirksMode::Quirks;
            }
        }
    }

    // Limited quirks
    const LIMITED_QUIRKS_PREFIXES: &[&str] = &[
        "-//w3c//dtd xhtml 1.0 frameset//",
        "-//w3c//dtd xhtml 1.0 transitional//",
    ];
    for prefix in LIMITED_QUIRKS_PREFIXES {
        if pub_lower.starts_with(prefix) {
            return QuirksMode::LimitedQuirks;
        }
    }

    if system_id.is_some() {
        const LIMITED_QUIRKS_WITH_SYSTEM: &[&str] = &[
            "-//w3c//dtd html 4.01 frameset//",
            "-//w3c//dtd html 4.01 transitional//",
        ];
        for prefix in LIMITED_QUIRKS_WITH_SYSTEM {
            if pub_lower.starts_with(prefix) {
                return QuirksMode::LimitedQuirks;
            }
        }
    }

    QuirksMode::NoQuirks
}

/// Parse a complete HTML document from a string.
pub fn parse_str(input: &str) -> ParseResult {
    let builder = TreeBuilder::new(input);
    builder.run()
}

/// Parse with scripting flag.
pub fn parse_str_scripting(input: &str, scripting_enabled: bool) -> ParseResult {
    let mut builder = TreeBuilder::new(input);
    builder.scripting_enabled = scripting_enabled;
    builder.run()
}

/// Parse using the fragment parsing algorithm with the provided context element.
pub fn parse_fragment(
    input: &str,
    context: FragmentContext,
    scripting_enabled: bool,
) -> ParseResult {
    let mut builder = TreeBuilder::new(input);
    builder.scripting_enabled = scripting_enabled;
    builder.frameset_ok = false;
    builder.fragment_context = Some(context.clone());

    // Root <html> element that collects parsed nodes.
    let fragment_root = builder.arena.create_node(NodeData::Element {
        tag_name: "html".to_string(),
        namespace: Namespace::Html,
        attributes: Vec::new(),
    });
    builder.arena.append_child(builder.document, fragment_root);
    builder.open_elements.push(fragment_root);
    builder.fragment_root = Some(fragment_root);
    builder.fragment_extra_context.clear();

    // Synthetic context element kept on the stack to mimic the surrounding DOM.
    let context_node = match context.namespace {
        Namespace::Html => match context.tag_name.as_str() {
            "html" => None,
            _ => builder.create_fragment_html_context_chain(fragment_root, &context),
        },
        _ => {
            let node = builder.arena.create_node(NodeData::Element {
                tag_name: context.tag_name.clone(),
                namespace: context.namespace,
                attributes: Vec::new(),
            });
            builder.arena.append_child(fragment_root, node);
            builder.open_elements.push(node);
            Some(node)
        }
    };
    builder.fragment_context_node = context_node;

    if context.namespace == Namespace::Html && context.tag_name == "head" {
        if let Some(context_node) = context_node {
            builder.head_pointer = Some(context_node);
        }
    }

    if context.namespace == Namespace::Html && context.tag_name == "template" {
        builder.template_modes.push(InsertionMode::InTemplate);
    }

    if context.namespace == Namespace::Html && context.tag_name == "form" {
        if let Some(context_node) = context_node {
            builder.form_pointer = Some(context_node);
        }
    }

    builder.set_fragment_tokenizer_state(&context);
    builder.reset_insertion_mode();

    builder.run()
}

impl<'a> TreeBuilder<'a> {
    fn set_fragment_tokenizer_state(&mut self, context: &FragmentContext) {
        if context.namespace != Namespace::Html {
            return;
        }
        match context.tag_name.as_str() {
            "title" | "textarea" => self.tokenizer.set_state(State::RcData),
            "style" | "xmp" | "iframe" | "noembed" | "noframes" => {
                self.tokenizer.set_state(State::RawText)
            }
            "script" => self.tokenizer.set_state(State::ScriptData),
            "noscript" => {
                if self.scripting_enabled {
                    self.tokenizer.set_state(State::RawText);
                }
            }
            "plaintext" => self.tokenizer.set_state(State::PlainText),
            _ => {}
        }
    }

    fn create_fragment_html_context_chain(
        &mut self,
        fragment_root: NodeId,
        context: &FragmentContext,
    ) -> Option<NodeId> {
        let tag_lower = context.tag_name.to_ascii_lowercase();
        let mut parent = fragment_root;

        for ancestor in Self::html_fragment_ancestors(&tag_lower) {
            let node = self.create_synthetic_html_element(ancestor);
            self.arena.append_child(parent, node);
            self.open_elements.push(node);
            self.fragment_extra_context.push(node);
            parent = node;
        }

        let context_node = self.create_synthetic_html_element(&context.tag_name);
        self.arena.append_child(parent, context_node);
        self.open_elements.push(context_node);
        Some(context_node)
    }

    fn create_synthetic_html_element(&mut self, tag_name: &str) -> NodeId {
        let node = self.arena.create_node(NodeData::Element {
            tag_name: tag_name.to_string(),
            namespace: Namespace::Html,
            attributes: Vec::new(),
        });

        if tag_name == "template" {
            let content = self.arena.create_node(NodeData::Document {
                quirks_mode: QuirksMode::NoQuirks,
            });
            self.arena.append_child(node, content);
        }

        node
    }

    fn html_fragment_ancestors(tag: &str) -> &'static [&'static str] {
        match tag {
            "caption" => &["body", "table"],
            "colgroup" => &["body", "table"],
            "col" => &["body", "table", "colgroup"],
            "tbody" | "thead" | "tfoot" => &["body", "table"],
            "tr" => &["body", "table", "tbody"],
            "td" | "th" => &["body", "table", "tbody", "tr"],
            "table" => &["body"],
            _ => &[],
        }
    }
}
