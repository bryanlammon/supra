//! The structures and functions for configuration. Must be accessible to main.

/// The overall options.
pub struct SupraConfig<'a> {
    pub command: SupraCommand<'a>,
    pub output: Option<Output>,
    pub pre_config: Option<PreConfig<'a>>,
    pub pan_config: Option<PanConfig<'a>>,
    pub post_config: Option<PostConfig>,
}

impl SupraConfig<'_> {
    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(
        command: SupraCommand<'a>,
        output: Option<Output>,
        pre_config: Option<PreConfig<'a>>,
        pan_config: Option<PanConfig<'a>>,
        post_config: Option<PostConfig>,
    ) -> SupraConfig<'a> {
        SupraConfig {
            command,
            output,
            pre_config,
            pan_config,
            post_config,
        }
    }
}

/// The types of subcommands.
pub enum SupraCommand<'a> {
    Main,
    NewUserJournalFile,
    NewProject(&'a str, bool, bool),
    ReplaceMake,
}

/// Output options
#[derive(PartialEq, Eq, Debug)]
pub enum Output {
    StandardOut,
    Markdown,
    Docx,
}

/// Pre-processor configuration.
pub struct PreConfig<'a> {
    pub input: &'a str,
    pub library: &'a str,
    pub offset: i32,
    pub user_journals: Option<&'a str>,
    pub smallcaps: bool,
}

impl PreConfig<'_> {
    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(
        input: &'a str,
        library: &'a str,
        offset: i32,
        user_journals: Option<&'a str>,
        smallcaps: bool,
    ) -> PreConfig<'a> {
        PreConfig {
            input,
            library,
            offset,
            user_journals,
            smallcaps,
        }
    }
}

/// Pandoc configuration.
pub struct PanConfig<'a> {
    pub output: Option<&'a str>,
    pub pandoc_reference: Option<&'a str>,
}

impl PanConfig<'_> {
    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(output: Option<&'a str>, pandoc_reference: Option<&'a str>) -> PanConfig<'a> {
        PanConfig {
            output,
            pandoc_reference,
        }
    }
}

/// Post-processor configuration.
pub struct PostConfig {
    pub autocref: bool,
    pub author_note: bool,
    pub tabbed_footnotes: bool,
    pub no_superscript: bool,
    pub running_header: bool,
}

impl PostConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        autocref: bool,
        author_note: bool,
        tabbed_footnotes: bool,
        no_superscript: bool,
        running_header: bool,
    ) -> PostConfig {
        PostConfig {
            autocref,
            author_note,
            tabbed_footnotes,
            no_superscript,
            running_header,
        }
    }
}
