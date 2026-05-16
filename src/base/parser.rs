use std::fmt;

/// A leaf in the bracket structure tree.
/// Stores the start and end byte offsets into the original source string.
#[derive(Debug, Clone, Copy)]
pub struct Leaf {
    pub begin: usize,
    pub end: usize,
}

impl Leaf {
    pub fn is_empty(&self) -> bool {
        self.begin >= self.end
    }
}

/// Represents a node in the parsed bracket structure.
#[derive(Debug)]
pub enum Node {
    /// A leaf value (text between delimiters)
    Leaf(Leaf),
    /// A nested structure { ... }
    Branch(Vec<Node>),
}

/// Parser for 1C bracket structures like:
/// `{value,{nested1,nested2},{"quoted,value"}}`
///
/// Handles newlines and whitespace between values.
/// The parser maintains a flat list of leaves with begin/end ranges
/// into the original source string, avoiding string copying.
#[derive(Debug)]
pub struct StructParser {
    /// Raw source string
    source: String,
    /// Root node of the parsed tree
    root: Node,
}

impl StructParser {
    /// Parse a bracket structure string.
    /// The string must start with `{` and end with `}`.
    pub fn new(source: String) -> Result<Self, ParseError> {
        let bytes = source.as_bytes();
        if bytes.is_empty() || bytes[0] != b'{' {
            return Err(ParseError::NoOpeningBrace);
        }

        let mut pos = 0;
        let root = Self::parse_node(&source, &mut pos)?;

        Ok(Self { source, root })
    }

    /// Skip whitespace characters
    fn skip_ws(source: &str, pos: &mut usize) {
        let bytes = source.as_bytes();
        while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
            *pos += 1;
        }
    }

    /// Parse a single node (either a branch or a sequence of leaves)
    fn parse_node(source: &str, pos: &mut usize) -> Result<Node, ParseError> {
        Self::skip_ws(source, pos);

        let bytes = source.as_bytes();
        if *pos >= bytes.len() {
            return Err(ParseError::UnexpectedEnd);
        }

        if bytes[*pos] == b'{' {
            *pos += 1; // skip '{'
            let mut children = Vec::new();

            loop {
                Self::skip_ws(source, pos);
                if *pos >= bytes.len() {
                    return Err(ParseError::UnexpectedEnd);
                }

                match bytes[*pos] {
                    b'}' => {
                        *pos += 1;
                        return Ok(Node::Branch(children));
                    }
                    b'{' => {
                        children.push(Self::parse_node(source, pos)?);
                    }
                    b',' => {
                        *pos += 1;
                    }
                    _ => {
                        let start = *pos;
                        Self::read_value(source, pos);
                        let end = *pos;
                        children.push(Node::Leaf(Leaf { begin: start, end }));
                    }
                }
            }
        } else {
            let start = *pos;
            Self::read_value(source, pos);
            let end = *pos;
            Ok(Node::Leaf(Leaf { begin: start, end }))
        }
    }

    /// Read a value until delimiter (comma, brace)
    fn read_value(source: &str, pos: &mut usize) {
        let bytes = source.as_bytes();
        let mut in_quotes = false;

        while *pos < bytes.len() {
            let ch = bytes[*pos];

            // Skip whitespace outside quotes
            if !in_quotes && ch.is_ascii_whitespace() {
                *pos += 1;
                continue;
            }

            if ch == b'"' {
                in_quotes = !in_quotes;
                *pos += 1;
            } else if !in_quotes && (ch == b'{' || ch == b'}' || ch == b',') {
                break;
            } else {
                *pos += 1;
            }
        }
    }

    /// Navigate to a nested node by a path of indices.
    /// E.g., `goto(&[4, 2, 2, 4, 2, 3])` follows the tree depth-first.
    pub fn get_leaf(&self, path: &[usize]) -> Option<&str> {
        let node = Self::navigate(&self.root, path)?;
        match node {
            Node::Leaf(leaf) => {
                if leaf.is_empty() {
                    None
                } else {
                    Some(&self.source[leaf.begin..leaf.end])
                }
            }
            Node::Branch(_) => None,
        }
    }

    /// Navigate to a node (branch or leaf) by path.
    fn navigate<'a>(node: &'a Node, path: &[usize]) -> Option<&'a Node> {
        if path.is_empty() {
            return Some(node);
        }

        match node {
            Node::Branch(children) => {
                let idx = path[0];
                if idx >= children.len() {
                    return None;
                }
                Self::navigate(&children[idx], &path[1..])
            }
            Node::Leaf(_) => None,
        }
    }

    /// Navigate to a branch node by path, returning its children.
    pub fn get_branch(&self, path: &[usize]) -> Option<&[Node]> {
        let node = Self::navigate(&self.root, path)?;
        match node {
            Node::Branch(children) => Some(children),
            Node::Leaf(_) => None,
        }
    }

    /// Count children in a branch at the given path.
    pub fn branch_len(&self, path: &[usize]) -> Option<usize> {
        self.get_branch(path).map(|c| c.len())
    }

    /// Get the source string reference.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the root node.
    pub fn root(&self) -> &Node {
        &self.root
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    NoOpeningBrace,
    UnexpectedEnd,
    UnmatchedBrace,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NoOpeningBrace => write!(f, "bracket structure must start with '{{'"),
            ParseError::UnexpectedEnd => write!(f, "unexpected end of input"),
            ParseError::UnmatchedBrace => write!(f, "unmatched closing brace '}}'"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Convenience: strip surrounding quotes from a value string.
/// E.g., `"Hello"` -> `Hello`
pub fn strip_quotes(s: &str) -> &str {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_values() {
        let parser = StructParser::new("{a,b,c}".to_string()).unwrap();
        assert_eq!(parser.get_leaf(&[0]), Some("a"));
        assert_eq!(parser.get_leaf(&[1]), Some("b"));
        assert_eq!(parser.get_leaf(&[2]), Some("c"));
    }

    #[test]
    fn test_nested_branches() {
        let parser = StructParser::new("{a,{b,c},d}".to_string()).unwrap();
        assert_eq!(parser.get_leaf(&[0]), Some("a"));
        assert_eq!(parser.get_leaf(&[1, 0]), Some("b"));
        assert_eq!(parser.get_leaf(&[1, 1]), Some("c"));
        assert_eq!(parser.get_leaf(&[2]), Some("d"));
    }

    #[test]
    fn test_quoted_values() {
        let parser = StructParser::new("{{\"quoted,value\"},plain}".to_string()).unwrap();
        assert_eq!(parser.get_leaf(&[0, 0]), Some("\"quoted,value\""));
        assert_eq!(parser.get_leaf(&[1]), Some("plain"));
    }

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
        assert_eq!(strip_quotes("hello"), "hello");
        assert_eq!(strip_quotes("\""), "\"");
    }

    #[test]
    fn test_deep_navigation() {
        // {0,1,2,3,{0,1,{0,1,{0,1,2,3,{0,1,2,leaf}}}}}
        // [4,2,2,4,3] = "leaf"
        let parser =
            StructParser::new("{0,1,2,3,{0,1,{0,1,{0,1,2,3,{0,1,2,leaf}}}}}".to_string()).unwrap();
        assert_eq!(parser.get_leaf(&[4, 2, 2, 4, 3]), Some("leaf"));
    }

    #[test]
    fn test_empty_branch() {
        let parser = StructParser::new("{a,{},b}".to_string()).unwrap();
        assert_eq!(parser.get_leaf(&[0]), Some("a"));
        // [1] is an empty branch
        assert!(parser.get_leaf(&[1]).is_none());
        assert_eq!(parser.get_leaf(&[2]), Some("b"));
    }

    #[test]
    fn test_invalid_no_brace() {
        assert!(StructParser::new("no brace".to_string()).is_err());
    }

    #[test]
    fn test_path_out_of_bounds() {
        let parser = StructParser::new("{a,b}".to_string()).unwrap();
        assert!(parser.get_leaf(&[5]).is_none());
    }

    #[test]
    fn test_leaf_on_branch_returns_none() {
        let parser = StructParser::new("{a,{b,c}}".to_string()).unwrap();
        assert!(parser.get_leaf(&[1]).is_none()); // [1] is a branch, not a leaf
    }

    #[test]
    fn test_with_nested_newlines() {
        let parser = StructParser::new("{a,\nb,\nc}".to_string()).unwrap();
        assert_eq!(parser.get_leaf(&[0]), Some("a"));
        assert_eq!(parser.get_leaf(&[1]), Some("b"));
        assert_eq!(parser.get_leaf(&[2]), Some("c"));
    }

    #[test]
    fn test_incomplete_input() {
        // Incomplete input should return error
        assert!(StructParser::new("{a,".to_string()).is_err());
    }
}
