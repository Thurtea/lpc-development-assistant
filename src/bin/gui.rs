use iced::{executor, Application, Command, Element, Settings, Theme, Length, Color};
use iced::theme;
use iced::widget::{column, row, Text, TextInput, Button, Scrollable, PickList, Container};

use chrono::Utc;
use walkdir::WalkDir;
use std::fs::{self, File};
use std::io::Write;
use std::process::Command as SysCommand;
use std::fmt;
// use serde_json::Value;
use std::sync::Arc;

// syntect for syntax highlighting
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;
// use syntect::highlighting::Color as SyntectColor;

mod efun_loader;
use efun_loader::lookup_efun;


#[derive(Debug, Clone, PartialEq, Eq)]
enum Context {
    Driver,
    Efuns,
    Mudlib,
    References,
}

impl Context {
    const ALL: [Context; 4] = [
        Context::Driver,
        Context::Efuns,
        Context::Mudlib,
        Context::References,
    ];
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Context::Driver => write!(f, "Driver"),
            Context::Efuns => write!(f, "Efuns"),
            Context::Mudlib => write!(f, "Mudlib"),
            Context::References => write!(f, "Reference Libraries"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    PromptChanged(String),
    AskOllama,
    ModelsLoaded(Vec<String>),
    ModelSelected(String),
    ContextSelected(Context),
    SavePressed,
    SearchReferences,
    ReferencesFound(Vec<String>),
    ReceivedResponse(String),
    ErrorOccurred(String),

    DriverPathChanged(String),
    AnalyzeDriver,
    DriverAnalyzed(Result<Vec<(String, String)>, String>),
    OpenEfun(String),
    SearchEfun(String),
    EfunDetails(Result<Option<(String, String)>, String>),
    EfunFilterChanged(String),
}

struct LPCGui {
    models: Vec<String>,
    selected_model: Option<String>,
    context: Context,
    prompt: String,
    response: String,
    loading: bool,

    references: Vec<String>,
    error_message: Option<String>,

    // Driver analysis state
    driver_path: String,
    efuns: Vec<(String, String)>, // (name, path)
    analysis_loading: bool,

    // Efuns reference list (loaded from gen/efuns_v2.txt)
    efuns_list: Vec<String>,

    // Selected efun details (name, docs/signature)
    efun_details: Option<(String, String)>,

    // Syntax highlighting (syntect)
    syntax_set: Arc<SyntaxSet>,
    theme_set: Arc<ThemeSet>,
    theme_name: String,
    efun_filter: String,
}

impl Default for LPCGui {
    fn default() -> Self {
        Self {
            models: vec![],
            selected_model: None,
            context: Context::Driver,
            prompt: String::new(),
            response: String::new(),
            loading: false,
            references: vec![],
            error_message: None,
            driver_path: String::new(),
            efuns: Vec::new(),
            analysis_loading: false,
            efuns_list: fs::read_to_string("gen\\efuns_v2.txt").map(|s| s.lines().map(|l| l.to_string()).collect()).unwrap_or_default(),
            efun_details: None,
            syntax_set: Arc::new(SyntaxSet::load_defaults_newlines()),
            theme_set: Arc::new(ThemeSet::load_defaults()),
            theme_name: ThemeSet::load_defaults().themes.keys().next().cloned().unwrap_or_else(|| "InspiredGitHub".to_string()),
            efun_filter: String::new(),
        }
    }
}

impl Application for LPCGui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self::default(),
            Command::perform(load_ollama_models(), Message::ModelsLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("LPC Dev Assistant - Native GUI")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PromptChanged(p) => self.prompt = p,



            Message::ModelsLoaded(ms) => {
                self.models = ms;
                if self.selected_model.is_none() {
                    self.selected_model = self.models.get(0).cloned();
                }
            }

            Message::ModelSelected(m) => {
                self.selected_model = Some(m);
            }

            Message::ContextSelected(c) => {
                self.context = c;
            }

            Message::SearchReferences => {
                return Command::perform(search_mud_references(), Message::ReferencesFound);
            }

            Message::ReferencesFound(list) => {
                self.references = list;
            }

            Message::SearchEfun(name) => {
                // Set prompt to efun name and show in response for quick lookup
                self.prompt = name.clone();
                self.response = format!("Selected efun: {}", name);
                let n = name.clone();
                return Command::perform(async move { lookup_efun(&n).map_err(|e| e.to_string()) }, Message::EfunDetails);
            }

            Message::EfunDetails(result) => {
                match result {
                    Ok(opt) => {
                        self.efun_details = opt;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
            }

            Message::AskOllama => {
                if self.prompt.is_empty() {
                    self.error_message = Some("Please enter a question".to_string());
                    return Command::none();
                }

                if self.selected_model.is_none() {
                    self.error_message = Some("Please select a model".to_string());
                    return Command::none();
                }

                self.loading = true;
                self.response.clear();
                self.error_message = None;

                let model = self.selected_model.clone().unwrap();
                let prompt = self.prompt.clone();
                let context = self.context.clone();

                return Command::perform(
                    async move {
                        // Run async Ollama calls on a dedicated Tokio runtime to avoid "no reactor" panics
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .expect("failed to build tokio runtime");
                        rt.block_on(generate_response(model, prompt, context))
                    },
                    |result| match result {
                        Ok(response) => Message::ReceivedResponse(response),
                        Err(err) => Message::ErrorOccurred(err),
                    },
                );
            }

            Message::ReceivedResponse(s) => {
                self.loading = false;
                self.response = s.clone();
            }

            Message::ErrorOccurred(err) => {
                self.loading = false;
                self.error_message = Some(err);
            }

            Message::SavePressed => {
                let content = self.response.clone();
                let dir = std::path::Path::new("gen");
                let _ = std::fs::create_dir_all(dir);
                let filename = format!("mudlib_{}.c", Utc::now().format("%Y%m%d%H%M%S"));
                let path = dir.join(filename);
                let _ = File::create(path).and_then(|mut f| f.write_all(content.as_bytes()));
            }

            Message::DriverPathChanged(p) => {
                self.driver_path = p;
            }

            Message::AnalyzeDriver => {
                if self.driver_path.is_empty() {
                    self.error_message = Some("Please provide a driver path to analyze".to_string());
                    return Command::none();
                }
                self.analysis_loading = true;
                self.error_message = None;
                let path = self.driver_path.clone();
                return Command::perform(
                    async move {
                        // Call analyzer from library crate; returns Result<Vec<(String,String)>, String>
                        lpc_dev_assistant::driver_analyzer::efuns_json(&path)
                    },
                    Message::DriverAnalyzed,
                );
            }

            Message::DriverAnalyzed(result) => {
                self.analysis_loading = false;
                match result {
                    Ok(list) => {
                        self.efuns = list;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
            }

            Message::OpenEfun(file_path) => {
                // Try opening in VS Code first; fall back to notepad.
                if SysCommand::new("code").arg(&file_path).spawn().is_err() {
                    let _ = SysCommand::new("notepad").arg(&file_path).spawn();
                }
            }

            Message::EfunFilterChanged(s) => {
                self.efun_filter = s;
            }

        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Title
        let title = Text::new("🎮 LPC MUD Development Assistant")
            .size(24);

        // Model picker
        let model_picker = PickList::new(
            self.models.clone(),
            self.selected_model.clone(),
            Message::ModelSelected,
        )
        .placeholder("Select model...")
        .width(Length::Fixed(250.0));

        // Context tabs as buttons with icons and subtitles
        let driver_tab = column![
            Button::new(Text::new("⚙️ Driver")).on_press(Message::ContextSelected(Context::Driver)),
            Text::new("C source analysis (efuns, config)").size(12),
        ]
        .spacing(4);

        let efuns_tab = column![
            Button::new(Text::new("🔧 Efuns")).on_press(Message::ContextSelected(Context::Efuns)),
            Text::new("Function reference (this_player, call_out)").size(12),
        ]
        .spacing(4);

        let mudlib_tab = column![
            Button::new(Text::new("📚 Mudlib")).on_press(Message::ContextSelected(Context::Mudlib)),
            Text::new("LPC inheritance/objects").size(12),
        ]
        .spacing(4);

        let refs_tab = column![
            Button::new(Text::new("📖 Libraries")).on_press(Message::ContextSelected(Context::References)),
            Text::new("Mudlib libraries index").size(12),
        ]
        .spacing(4);

        let context_buttons = row![driver_tab, efuns_tab, mudlib_tab, refs_tab].spacing(10);

        // Buttons (styled)
        let ask_button = Button::new(
            Text::new(if self.loading { "⏳ Generating..." } else { "🚀 Ask Ollama" })
                .size(14)
        )
        .on_press(Message::AskOllama)
        .padding([8, 16]);

        let save_button = if !self.response.is_empty() {
            Button::new(Text::new("💾 Save").size(14))
                .on_press(Message::SavePressed)
                .padding([8, 16])
        } else {
            Button::new(Text::new("💾 Save").size(14))
                .padding([8, 16])
        };

        let search_button = Button::new(Text::new("🔍 Search").size(14))
            .on_press(Message::SearchReferences)
            .padding([8, 16]);

        // Controls row
        let controls = row![
            model_picker,
            context_buttons,
            ask_button,
            save_button,
            search_button,
        ]
        .spacing(10)
        .padding(10);

        // Error/status message
        let status_text = if let Some(err) = &self.error_message {
            Text::new(err).size(12)
        } else {
            Text::new("").size(12)
        };

        // Subtitle for current tab
        let subtitle = match self.context {
            Context::Driver => Text::new("C source analysis (efuns, config)").size(12),
            Context::Efuns => Text::new("Function reference (this_player, call_out)").size(12),
            Context::Mudlib => Text::new("LPC inheritance/objects").size(12),
            Context::References => Text::new("Mudlib libraries index").size(12),
        };
        // Prompt input
        let prompt_input = TextInput::new(
            "Ask about LPC driver implementation, mudlib features, or C programming...",
            &self.prompt,
        )
        .on_input(Message::PromptChanged)
        .padding(12)
        .size(14);

        // Response display (plain, selectable text)
        let response_content = if !self.response.is_empty() {
            self.response.clone()
        } else if self.loading {
            "⏳ Generating response from Ollama...\nThis may take 30-60 seconds.".to_string()
        } else {
            "💡 Response will appear here...".to_string()
        };

        let response_view: Element<Message> = if !self.response.is_empty() {
            render_highlighted_element(self.response.clone(), self.syntax_set.clone(), self.theme_set.clone(), &self.theme_name)
        } else {
            let txt = if self.loading {
                "⏳ Generating response from Ollama...\nThis may take 30-60 seconds.".to_string()
            } else {
                "💡 Response will appear here...".to_string()
            };
            Container::new(Text::new(txt).size(13)).padding(15).into()
        };

        let response_view = Scrollable::new(response_view)
            .height(Length::Fill);

        // Efun details side pane
        let efun_details_view: Element<Message> = if let Some((name, docs)) = &self.efun_details {
            let sig = docs.lines().next().unwrap_or("");
            let details = column![
                Text::new(format!("efun: {}", name)).size(16),
                Text::new(sig).size(13),
                Scrollable::new(Container::new(Text::new(docs).size(12)).padding(8)).height(Length::Fixed(200.0)),
            ]
            .spacing(6)
            .padding(8);
            Container::new(details).width(Length::Fixed(360.0)).into()
        } else {
            Container::new(Text::new("")).width(Length::Fixed(0.0)).into()
        };

        // References list
        let refs_content = if !self.references.is_empty() {
            let mut text = String::from("Found references:\n\n");
            for (i, r) in self.references.iter().take(20).enumerate() {
                text.push_str(&format!("{}. {}\n", i + 1, r));
            }
            if self.references.len() > 20 {
                text.push_str(&format!("\n... and {} more files", self.references.len() - 20));
            }
            text
        } else {
            "No references searched yet\n\nClick 'Search Refs' to scan mud-references folder".to_string()
        };

        let refs_display = Container::new(
            Text::new(refs_content)
                .size(11)
        )
        .padding(10);

        let refs_view = Scrollable::new(refs_display)
            .height(Length::Fixed(180.0));

        // Efuns reference list view (shown when Efuns tab selected)
        let efuns_list_view: Element<Message> = if self.context == Context::Efuns {
            let mut col_ef = column![];
            for e in &self.efuns_list {
                col_ef = col_ef.push(Button::new(Text::new(e).size(12)).on_press(Message::SearchEfun(e.clone())).padding(4));
            }
            Scrollable::new(col_ef).height(Length::Fixed(200.0)).into()
        } else {
            Container::new(Text::new("")).into()
        };

        // Driver analysis controls
        let driver_input = row![
            TextInput::new("Path to driver root (e.g., your_driver/src)", &self.driver_path)
                .on_input(Message::DriverPathChanged)
                .padding(8)
                .width(Length::Fill),
            Button::new(Text::new(if self.analysis_loading { "⏳ Analyzing..." } else { "🔧 Analyze Driver" }))
                .on_press(Message::AnalyzeDriver)
                .padding([8, 12])
        ]
        .spacing(10);

        let efuns_view: Element<Message> = if self.analysis_loading {
            Container::new(Text::new("Analyzing driver...").size(12)).padding(10).into()
        } else if !self.efuns.is_empty() {
            // Filter input
            let filter_input = TextInput::new("Filter efuns (e.g., socket, inventory)", &self.efun_filter)
                .on_input(Message::EfunFilterChanged)
                .padding(8)
                .width(Length::Fill);

            // Build clickable efun list with filter applied
            let mut col = column![filter_input].spacing(6);
            let filt = self.efun_filter.to_lowercase();
            for (i, (name, path)) in self.efuns.iter().enumerate() {
                if !filt.is_empty() {
                    let n = name.to_lowercase();
                    let p = path.to_lowercase();
                    if !n.contains(&filt) && !p.contains(&filt) { continue; }
                }
                let label = format!("{}. {} -> {}", i + 1, name, path);
                col = col.push(Button::new(Text::new(label).size(12)).on_press(Message::OpenEfun(path.clone())).padding([4, 8]));
            }
            Scrollable::new(col).height(Length::Fixed(200.0)).into()
        } else {
            Container::new(Text::new("No analysis results yet. Provide a path and click Analyze Driver.").size(12)).padding(10).into()
        };

        // Split layout: left (input + analysis) | right (response + efun details)
        let left_panel = column![
            subtitle,
            status_text,
            Text::new("📝 Query:").size(16),
            prompt_input,
            Text::new("🔧 Driver Analysis:").size(16),
            driver_input,
            efuns_view,
            Text::new("📚 References:").size(16),
            refs_view,
            if self.context == Context::Efuns { Text::new("🔧 Efuns Reference:").size(16) } else { Text::new("").size(0) },
            efuns_list_view,
        ]
        .spacing(10)
        .width(Length::FillPortion(1));

        let right_panel = column![
            Text::new("💬 Response:").size(16),
            row![response_view, efun_details_view],
        ]
        .spacing(10)
        .width(Length::FillPortion(1));

        let content = column![
            title,
            controls,
            row![left_panel, right_panel].spacing(12),
        ]
        .spacing(10)
        .padding(20);

        content.into()
    }
}

// Async helpers and main
async fn load_ollama_models() -> Vec<String> {
    // Try to get models from ollama CLI
    let output = SysCommand::new("ollama")
        .arg("list")
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let models: Vec<String> = stdout
            .lines()
            .skip(1) // Skip header
            .filter_map(|line| {
                line.split_whitespace().next().map(|s| s.to_string())
            })
            .collect();

        if !models.is_empty() {
            return models;
        }
    }

    // Fallback models
    vec![
        "qwen2.5-coder:7b".to_string(),
        "qwen2.5:7b-instruct".to_string(),
    ]
}

async fn search_mud_references() -> Vec<String> {
    let mut results = Vec::new();
    let base_path = "E:\\Work\\AMLP\\mud-references";

    for entry in WalkDir::new(base_path)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("c")
                    || ext.eq_ignore_ascii_case("h")
                    || ext.eq_ignore_ascii_case("lpc")
                {
                    results.push(entry.path().display().to_string());
                }
            }
        }

        // Limit results
        if results.len() >= 100 {
            break;
        }
    }

    results
}

async fn generate_response(
    model: String,
    prompt: String,
    context: Context,
) -> Result<String, String> {
    // Load context template
    let context_text = load_context_template(&context)?;

    // Combine context + prompt
    let full_prompt = if !context_text.is_empty() {
        format!(
            "{}\n\n---\n\nQUESTION:\n{}\n\nProvide complete, production-ready code with detailed comments.",
            context_text, prompt
        )
    } else {
        prompt
    };

    // Try Ollama HTTP API first
    match call_ollama_http(&model, &full_prompt).await {
        Ok(response) => Ok(response),
        Err(http_err) => {
            // Fallback to CLI
            eprintln!("HTTP failed: {}, trying CLI...", http_err);
            call_ollama_cli(&model, &full_prompt).await
        }
    }
}

fn load_context_template(context: &Context) -> Result<String, String> {
    let base_path = "E:\\Work\\AMLP\\lpc-dev-assistant\\templates";

    let filename = match context {
        Context::Driver => "driver_context.txt",
        Context::Efuns => "efuns_context.txt",
        Context::Mudlib => "mudlib_context.txt",
        Context::References => "reference_sources.txt",
    };

    let path = std::path::Path::new(base_path).join(filename);

    fs::read_to_string(&path).map_err(|e| {
        format!("Failed to load context template '{}': {}", filename, e)
    })
}

async fn call_ollama_http(model: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
            "num_predict": 4096,
        }
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama returned error: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(json["response"]
        .as_str()
        .unwrap_or("No response from Ollama")
        .to_string())
}

async fn call_ollama_cli(model: &str, prompt: &str) -> Result<String, String> {
    let output = SysCommand::new("ollama")
        .arg("run")
        .arg(model)
        .arg(prompt)
        .output()
        .map_err(|e| format!("Failed to run ollama CLI: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Ollama CLI failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}



fn render_highlighted_element(code: String, ss: Arc<SyntaxSet>, ts: Arc<ThemeSet>, theme_name: &str) -> Element<'static, Message> {
    // Prefer C syntax; fall back to plain text
    let syntax = ss.find_syntax_by_extension("c").unwrap_or_else(|| ss.find_syntax_plain_text());
    let theme = &ts.themes[theme_name];
    let mut h = HighlightLines::new(syntax, theme);

    let mut col = column![];
    for line in LinesWithEndings::from(&code) {
        let ranges = h.highlight_line(line, ss.as_ref()).unwrap_or_default();
        let mut row_w = row![];

        for (style, text_str) in ranges {
            // Map syntect color to iced Color
            let fg = style.foreground;
            let color = Color::from_rgb(
                fg.r as f32 / 255.0,
                fg.g as f32 / 255.0,
                fg.b as f32 / 255.0,
            );

            let text_widget = Text::new(text_str.to_string())
                .size(12)
                .style(theme::Text::Color(color));
            row_w = row_w.push(text_widget);
        }
        col = col.push(row_w);
    }

    Container::new(col).padding(15).into()
}

fn main() -> iced::Result {
    LPCGui::run(Settings {
        ..Settings::default()
    })
}

