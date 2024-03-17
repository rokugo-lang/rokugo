//! Resilient parser inspired by [matklad's _Resilient LL Parsing Tutorial_][matklad].
//! This makes the parser usable for code analysis purposes beyond compilation to binaries.
//!
//! [matklad]: https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html

pub mod expression;
mod skip_list;

use std::{cell::Cell, fmt, ops::Range, panic::Location};

use rokugo_ast::{Child, Tree, TreeKind};
use rokugo_diagnostic::Diagnostic;
use rokugo_lexis::token::{Token, TokenKind};
use rokugo_source_code::{FileId, SourceSpan, Sources};

pub use skip_list::TokenSkipList;

/// Parser state.
pub struct Parser<'s> {
    sources: &'s Sources,
    file_id: FileId,
    tokens: TokenSkipList,
    /// Position within the skip list.
    position: usize,
    fuel: Cell<u32>,
    events: Vec<Event>,
    pub diagnostics: Vec<Diagnostic>,
}

/// A tree that was [`open`][Parser::open]ed. It later has to be closed using [`Parser::close`] to
/// keep the tree balanced.
pub struct Opened {
    event_index: usize,
}

/// A tree that was [`close`][Parser::close]d. Any closed tree can be wrapped in a parent tree using
/// [`Parser::open_before`].
pub struct Closed {
    event_index: usize,
}

/// The kind of a parsing event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventKind {
    /// A tree was opened.
    Open { kind: TreeKind },
    /// A tree was closed.
    Close,
    /// A token has been consumed.
    Advance,
}

/// Debug metadata of an [`Event`]. Only present on debug builds.
#[derive(Clone, Copy)]
struct EventDebug {
    location: &'static Location<'static>,
}

/// A parse event.
#[derive(Debug, Clone, Copy)]
struct Event {
    kind: EventKind,
    #[cfg(debug_assertions)]
    _debug: EventDebug,
}

impl<'s> Parser<'s> {
    /// Limit how many lookahead operations the parser can do without advancing.
    /// This is used to prevent the parser from getting stuck forever.
    const MAX_FUEL: u32 = 256;

    /// Creates a new parser from the given source set and lexed+skipped tokens.
    pub fn new(sources: &'s Sources, file_id: FileId, tokens: TokenSkipList) -> Self {
        Self {
            sources,
            file_id,
            tokens,
            position: 0,
            fuel: Cell::new(Self::MAX_FUEL),
            events: vec![],
            diagnostics: vec![],
        }
    }

    /// Begins constructing a new tree node.
    ///
    /// The `Opened` token must be used (passed into [`close`][Self::close]) before it goes out of
    /// scope. Otherwise an assertion will be hit in [`into_tree`][Self::into_tree] due to
    /// unbalanced `open`/`close` pairs.
    #[track_caller]
    pub fn open(&mut self) -> Opened {
        let opened = Opened {
            event_index: self.events.len(),
        };
        self.events.push(Event::new(EventKind::Open {
            kind: TreeKind::Error,
        }));
        opened
    }

    pub fn open_before(&mut self, closed: Closed) -> Opened {
        let opened = Opened {
            event_index: closed.event_index,
        };
        self.events.insert(
            opened.event_index,
            Event::new(EventKind::Open {
                kind: TreeKind::Error,
            }),
        );
        opened
    }

    /// Ends constructing a new tree node, and assigns it a tree kind.
    ///
    /// Note that the tree kind is not assigned until the node is fully parsed. This way later
    /// stages of the compilation pipeline can ignore nodes that failed to parse to the end.
    #[track_caller]
    pub fn close(&mut self, opened: Opened, kind: TreeKind) -> Closed {
        self.events[opened.event_index] = Event::new(EventKind::Open { kind });
        self.events.push(Event::new(EventKind::Close));
        Closed {
            event_index: opened.event_index,
        }
    }

    /// Advances by a single token.
    pub fn advance(&mut self) {
        assert!(!self.at_end(), "parser must not advance past the end");

        self.fuel.set(Self::MAX_FUEL);
        self.events.push(Event::new(EventKind::Advance));
        self.position += 1;
    }

    /// Returns whether the parser is at the end of input.
    pub fn at_end(&self) -> bool {
        self.position >= self.tokens.code.len()
    }

    /// Returns the [`Range`] that points to the end of the file.
    pub fn eof_range(&self) -> Range<usize> {
        let len = self.sources.get(self.file_id).source.len();
        len..len
    }

    /// Constructs an [`EndOfFile`][TokenKind::EndOfFile] token at the
    /// [`eof_range`][Self::eof_range].
    pub fn eof_token(&self) -> Token {
        Token {
            kind: TokenKind::EndOfFile,
            range: self.eof_range(),
        }
    }

    /// Returns the current code token, or [`None`] if at the end of file.
    pub fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Peeks at the current code token and returns a copy of it. Returns
    /// [`eof_token`][Self::eof_token] at the end of the file.
    pub fn peek(&self) -> Token {
        assert!(self.fuel.get() > 0, "parser is stuck (out of fuel)");
        self.fuel.set(self.fuel.get() - 1);
        self.current().cloned().unwrap_or_else(|| self.eof_token())
    }

    /// Returns a [`SourceSpan`] representing the given range of characters in the file.
    pub fn span(&self, range: &Range<usize>) -> SourceSpan {
        SourceSpan {
            file_id: self.file_id,
            span: range.clone(),
        }
    }

    /// Returns the [`SourceSpan`] of the current token.
    pub fn current_span(&self) -> SourceSpan {
        SourceSpan {
            file_id: self.file_id,
            span: self.current().map_or_else(
                || {
                    let len = self.sources.get(self.file_id).source.len();
                    len..len
                },
                |it| it.range.clone(),
            ),
        }
    }

    /// Returns the token's text.
    pub fn text(&self, token: &Token) -> &str {
        self.sources.span(&self.span(&token.range))
    }

    /// Returns whether the parser is at the given token.
    pub fn at(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    /// Eats a token of the given kind. Does not do advance if the current token is not of the
    /// given kind.
    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expects that the next token will be of a given kind, emitting a diagnostic if it isn't.
    pub fn expect(
        &mut self,
        kind: TokenKind,
        error_diagnostic: impl FnOnce(&mut Self, SourceSpan) -> Diagnostic,
    ) {
        let span = self.current_span();
        if !self.eat(kind) {
            let diagnostic = error_diagnostic(self, span);
            self.diagnostics.push(diagnostic);
        }
    }

    /// Returns the list of trivia tokens following the current token.
    pub fn preceding_trivia(&self) -> &[Token] {
        self.tokens.before(self.position)
    }

    /// Emits a diagnostic.
    pub fn emit(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Constructs an error tree with the next token inside.
    pub fn advance_with_error(&mut self) -> Closed {
        let opened = self.open();
        self.advance();
        self.close(opened, TreeKind::Error)
    }

    /// Turns the flat list of parsed events into a [`Tree`].
    pub fn into_tree(self) -> Tree {
        #[derive(Debug)]
        struct StackTree {
            tree: Tree,
            #[cfg(debug_assertions)]
            _source_event: Event,
        }

        let mut token_indices = self.tokens.code.iter().copied();
        let mut events = self.events;
        let mut stack: Vec<StackTree> = vec![];

        // `Event::Close` pops a node off the stack and adds it into the parent node.
        // To prevent that from happening from the root node, we pop the last produced event off
        // the event list, such that the root node stays on the stack and we can return it.
        // The last event is expected to be `Close` because the root node is supposed to be, well,
        // _a node._
        match events.pop() {
            None => panic!("into_tree requires at least one event to be emitted by the parser"),
            Some(event) => {
                assert!(
                    event.kind == EventKind::Close,
                    "last event should be `Close`, but was {event:#?}"
                );
            }
        }

        for event in events {
            match event.kind {
                EventKind::Open { kind } => {
                    stack.push(StackTree {
                        tree: Tree {
                            kind,
                            children: vec![],
                        },
                        #[cfg(debug_assertions)]
                        _source_event: event,
                    });
                }
                EventKind::Close => {
                    if let Some(tree) = stack.pop() {
                        stack
                            .last_mut()
                            .expect("last `Close` event should have been popped")
                            .tree
                            .children
                            .push(Child::Tree(tree.tree));
                    } else {
                        panic!("`Close` event without a matching `Open`: {event:#?}");
                    }
                }
                EventKind::Advance => {
                    if let Some(token_index) = token_indices.next() {
                        let token = self.tokens.tokens[self.tokens.code[token_index]].clone();
                        let (leading, trailing) = self.tokens.trivia(token_index);
                        let tree = &mut stack.last_mut().expect("`Advance` should be inside of an `Open` to have a node to add tokens to").tree;
                        tree.children.push(Child::Token(token));
                        tree.children
                            .extend(leading.iter().cloned().map(Child::Token));
                        tree.children
                            .extend(trailing.iter().cloned().map(Child::Token));
                    } else {
                        panic!("`Advance` event went past the end of file: {event:#?}");
                    }
                }
            }
        }

        assert!(
            stack.len() == 1,
            "`Open` without a matching `Close`. stack: {stack:#?}"
        );
        assert_eq!(
            token_indices.next(),
            None,
            "not all tokens were consumed.\nremaining: {:?}\n      all: {:?}",
            token_indices.collect::<Vec<_>>(),
            self.tokens.tokens,
        );

        stack
            .pop()
            .expect("there should be a node on the stack to return")
            .tree
    }
}

impl Event {
    #[track_caller]
    fn new(kind: EventKind) -> Self {
        Self {
            kind,
            #[cfg(debug_assertions)]
            _debug: EventDebug {
                location: Location::caller(),
            },
        }
    }
}

impl fmt::Debug for EventDebug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@ {}", self.location)
    }
}
