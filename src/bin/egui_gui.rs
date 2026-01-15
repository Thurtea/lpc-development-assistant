//! LPC MUD Development Assistant - egui Native GUI
//! 
//! A working native GUI using egui with actual text selection and copy/paste.

use eframe::egui;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::path::PathBuf;
use chrono::Utc;

// For async cloud API calls
use tokio::runtime::Runtime;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "LPC MUD Development Assistant",
        options,
        Box::new(|cc| {
            // Default to light theme
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            Ok(Box::new(LPCDevApp::new()))
        }),
    )
}

#[derive(Debug, Clone, PartialEq)]
enum Context {
    Driver,
    Efuns,
    Mudlib,
    References,
}

impl Context {
    fn name(&self) -> &str {
        match self {
            Context::Driver => "Driver Development",
            Context::Efuns => "Efuns Implementation",
            Context::Mudlib => "MudLib/LPC Code",
            Context::References => "Reference Libraries",
        }
    }

    fn filename(&self) -> &str {
        match self {
            Context::Driver => "driver_context.txt",
            Context::Efuns => "efuns_context.txt",
            Context::Mudlib => "mudlib_context.txt",
            Context::References => "reference_sources.txt",
        }
    }

    fn save_prefix(&self) -> &str {
        match self {
            Context::Driver => "driver",
            Context::Efuns => "efuns",
            Context::Mudlib => "mudlib",
            Context::References => "reference",
        }
    }
}

enum GenerationState {
    Idle,
    Loading,
    Complete(String),
    Error(String),
}

#[derive(Clone)]
struct ModelInfo {
    name: String,
    is_cloud: bool,
    est_ms: u32,
}

#[derive(Clone, PartialEq)]
enum Provider {
    LocalOllama,
    OpenAI,
    Claude,
}

struct LPCDevApp {
    // UI State
    models: Vec<ModelInfo>,
    selected_model: Option<usize>,
    selected_context: Context,
    question: String,
    response: String,
    references: Vec<String>,
    status_message: String,

    // Theme
    is_light: bool,

    // Provider / API Key
    provider: Provider,
    api_key: String,
    
    // Generation state
    generation_state: GenerationState,
    generation_rx: Option<Receiver<Result<String, String>>>,
    
    // Paths
    workspace_root: PathBuf,
}

impl LPCDevApp {
    fn new() -> Self {
        let workspace_root = PathBuf::from("E:/Work/AMLP");
        
        Self {
            models: Self::load_models(),
            selected_model: Some(0),
            selected_context: Context::Driver,
            question: String::new(),
            response: String::new(),
            references: Vec::new(),
            status_message: String::new(),
            is_light: true,
            provider: Provider::LocalOllama,
            api_key: String::new(),
            generation_state: GenerationState::Idle,
            generation_rx: None,
            workspace_root,
        }
    }

    fn load_models() -> Vec<ModelInfo> {
        // Try to get models from Ollama (local)
        if let Ok(output) = std::process::Command::new("ollama")
            .arg("list")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let models: Vec<ModelInfo> = stdout
                .lines()
                .skip(1)
                .filter_map(|line| {
                    line.split_whitespace().next().map(|s| {
                        // assume local models
                        let name = s.to_string();
                        let est = if name.contains("1.5b") { 5000 } else if name.contains("3b") { 8000 } else { 30000 };
                        ModelInfo { name, is_cloud: false, est_ms: est }
                    })
                })
                .collect();
            
            if !models.is_empty() {
                return models;
            }
        }
        
        // Fallback models
        vec![
            ModelInfo { name: "qwen2.5-coder:7b".to_string(), is_cloud: false, est_ms: 45000 },
            ModelInfo { name: "qwen2.5:7b-instruct".to_string(), is_cloud: false, est_ms: 45000 },
            ModelInfo { name: "llama2:latest".to_string(), is_cloud: false, est_ms: 60000 },
        ]
    }

    fn load_context(&self) -> String {
        let templates_path = self.workspace_root
            .join("lpc-dev-assistant")
            .join("templates")
            .join(self.selected_context.filename());
        
        std::fs::read_to_string(templates_path).unwrap_or_default()
    }

    fn start_generation(&mut self) {
        if self.question.trim().is_empty() {
            self.status_message = "⚠️ Please enter a question".to_string();
            return;
        }

        if self.selected_model.is_none() {
            self.status_message = "⚠️ Please select a model".to_string();
            return;
        }

        let model = self.models[self.selected_model.unwrap()].name.clone();
        let question = self.question.clone();
        let context = self.load_context();
        let provider = self.provider.clone();
        let api_key = self.api_key.clone();

        self.generation_state = GenerationState::Loading;
        self.response.clear();
        self.status_message = "Generating response...".to_string();

        // Create channel for background thread
        let (tx, rx) = channel();
        self.generation_rx = Some(rx);

        // Spawn background thread
        thread::spawn(move || {
            let result = generate_response(&model, &question, &context, provider, &api_key);
            let _ = tx.send(result);
        });
    }

    fn check_generation(&mut self) {
        if let Some(rx) = &self.generation_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(response) => {
                        self.response = response;
                        self.generation_state = GenerationState::Complete(self.response.clone());
                        self.status_message = "✅ Response received!".to_string();
                    }
                    Err(error) => {
                        self.generation_state = GenerationState::Error(error.clone());
                        self.status_message = format!("❌ Error: {}", error);
                    }
                }
                self.generation_rx = None;
            }
        }
    }

    fn save_response(&mut self) {
        if self.response.is_empty() {
            self.status_message = "⚠️ No response to save".to_string();
            return;
        }

        let gen_path = self.workspace_root.join("lpc-dev-assistant").join("gen");
        if let Err(e) = std::fs::create_dir_all(&gen_path) {
            self.status_message = format!("❌ Failed to create gen/ directory: {}", e);
            return;
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.c", self.selected_context.save_prefix(), timestamp);
        let filepath = gen_path.join(&filename);

        match std::fs::write(&filepath, &self.response) {
            Ok(_) => {
                self.status_message = format!("✅ Saved to: {}", filename);
            }
            Err(e) => {
                self.status_message = format!("❌ Save failed: {}", e);
            }
        }
    }

    fn search_references(&mut self) {
        self.references.clear();
        
        let refs_path = self.workspace_root
            .join("mud-references")
            .join("extracted");

        if !refs_path.exists() {
            self.references.push("❌ References not extracted yet".to_string());
            self.status_message = "Run extraction in mud-references folder".to_string();
            return;
        }

        for entry in walkdir::WalkDir::new(refs_path)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if matches!(ext, "c" | "h" | "lpc") {
                        self.references.push(entry.path().display().to_string());
                    }
                }
            }

            if self.references.len() >= 100 {
                break;
            }
        }

        self.status_message = format!("Found {} reference files", self.references.len());
    }
}

impl eframe::App for LPCDevApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for generation completion
        self.check_generation();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Title
            ui.heading("🎮 LPC MUD Development Assistant");
            ui.add_space(10.0);

            // Top controls
            ui.horizontal(|ui| {
                // Theme toggle
                let theme_label = if self.is_light { "🌙" } else { "☀️" };
                if ui.button(theme_label).clicked() {
                    self.is_light = !self.is_light;
                    if self.is_light {
                        ctx.set_visuals(egui::Visuals::light());
                    } else {
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                }

                ui.add_space(8.0);

                // Provider selection
                ui.label("Provider:");
                egui::ComboBox::from_id_source("provider_select")
                    .selected_text(match self.provider {
                        Provider::LocalOllama => "Local Ollama",
                        Provider::OpenAI => "OpenAI",
                        Provider::Claude => "Claude",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.provider, Provider::LocalOllama, "Local Ollama");
                        ui.selectable_value(&mut self.provider, Provider::OpenAI, "OpenAI");
                        ui.selectable_value(&mut self.provider, Provider::Claude, "Claude");
                    });

                // API key for cloud providers
                if self.provider != Provider::LocalOllama {
                    ui.add_space(8.0);
                    ui.label("API Key:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.api_key)
                            .password(true)
                            .desired_width(200.0)
                    );
                }

                ui.add_space(8.0);

                ui.label("Model:");
                egui::ComboBox::from_id_source("model_select")
                    .selected_text(
                        self.selected_model
                            .map(|i| self.models[i].name.as_str())
                            .unwrap_or("Select model...")
                    )
                    .show_ui(ui, |ui| {
                        for (i, model) in self.models.iter().enumerate() {
                            let label = format!("{} {} (~{}s)", model.name, if model.is_cloud { "(cloud)" } else { "(local)" }, model.est_ms / 1000);
                            ui.selectable_value(&mut self.selected_model, Some(i), label);
                        }
                    });

                ui.add_space(8.0);

                let is_loading = matches!(self.generation_state, GenerationState::Loading);
                
                if ui.add_enabled(!is_loading, egui::Button::new("🚀 Generate")).clicked() {
                    self.start_generation();
                }

                if ui.add_enabled(!self.response.is_empty(), egui::Button::new("💾 Save")).clicked() {
                    self.save_response();
                }

                if ui.button("🔍 Search Refs").clicked() {
                    self.search_references();
                }
            });

            // Status message
            if !self.status_message.is_empty() {
                ui.add_space(5.0);
                ui.colored_label(
                    if self.status_message.starts_with("✅") {
                        egui::Color32::from_rgb(76, 175, 80)
                    } else if self.status_message.starts_with("❌") || self.status_message.starts_with("⚠️") {
                        egui::Color32::from_rgb(244, 67, 54)
                    } else {
                        egui::Color32::from_rgb(33, 150, 243)
                    },
                    &self.status_message
                );
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Question input
            ui.label("📝 Your Question:");
            let _question_response = ui.add(
                egui::TextEdit::multiline(&mut self.question)
                    .desired_width(f32::INFINITY)
                    .desired_rows(5)
                    .hint_text("Ask about LPC driver implementation, mudlib features, or C programming...\n\nExamples:\n- Write the complete lexer.c for LPC tokens\n- Implement the VM bytecode interpreter\n- Create a combat system for the mudlib")
                    .font(egui::FontId::monospace(13.0))
            );

            ui.add_space(10.0);

            // Response display
            ui.label("💬 Response:");
            
            let response_text = if matches!(self.generation_state, GenerationState::Loading) {
                match self.provider {
                    Provider::LocalOllama => "⏳ Generating response from local Ollama...",
                    Provider::OpenAI => "⏳ Generating response from OpenAI...",
                    Provider::Claude => "⏳ Generating response from Claude...",
                }
            } else if self.response.is_empty() {
                "Response will appear here...\n\n💡 Tip: Select a model, choose context, and ask a question!"
            } else {
                &self.response
            };

            egui::ScrollArea::vertical()
                .id_source("response_scroll")
                .max_height(300.0)
                .show(ui, |ui| {
                    // Use a mutable local string to satisfy TextEdit API when response is read-only
                    let mut local = response_text.to_string();
                    ui.add(
                        egui::TextEdit::multiline(&mut local)
                            .desired_width(f32::INFINITY)
                            .font(egui::FontId::monospace(12.0))
                            .interactive(true)  // This enables selection!
                            .code_editor()    // monospace, code-like
                    );
                });

            ui.add_space(10.0);

            // References display
            ui.label("📚 References:");
            egui::ScrollArea::vertical()
                .id_source("refs_scroll")
                .max_height(150.0)
                .show(ui, |ui| {
                    if self.references.is_empty() {
                        ui.label("No references searched yet\n\nClick 'Search Refs' to scan mud-references folder");
                    } else {
                        for (i, reference) in self.references.iter().take(50).enumerate() {
                            ui.label(format!("{}. {}", i + 1, reference));
                        }
                        if self.references.len() > 50 {
                            ui.label(format!("... and {} more files", self.references.len() - 50));
                        }
                    }
                });
        });

        // Request repaint if loading
        if matches!(self.generation_state, GenerationState::Loading) {
            ctx.request_repaint();
        }
    }
}

fn generate_response(model: &str, question: &str, context: &str, provider: Provider, api_key: &str) -> Result<String, String> {
    let full_prompt = if !context.is_empty() {
        format!(
            "{}\n\n---\n\nQUESTION:\n{}\n\nProvide complete, production-ready code with detailed comments.",
            context, question
        )
    } else {
        question.to_string()
    };

    match provider {
        Provider::LocalOllama => {
            // Try HTTP API first
            match call_ollama_http(model, &full_prompt) {
                Ok(response) => Ok(response),
                Err(e) => {
                    // Fallback to CLI
                    eprintln!("HTTP failed: {}, trying CLI...", e);
                    call_ollama_cli(model, &full_prompt)
                }
            }
        }
        Provider::OpenAI => {
            // Use async reqwest inside a runtime
            let rt = Runtime::new().map_err(|e| format!("Failed to init runtime: {}", e))?;
            rt.block_on(call_openai_async(api_key, model, &full_prompt))
        }
        Provider::Claude => {
            let rt = Runtime::new().map_err(|e| format!("Failed to init runtime: {}", e))?;
            rt.block_on(call_anthropic_async(api_key, model, &full_prompt))
        }
    }
}

async fn call_openai_async(api_key: &str, model: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 3000,
        "temperature": 0.2
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("OpenAI request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("OpenAI error: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("OpenAI parse failed: {}", e))?;
    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
        Ok(content.to_string())
    } else if let Some(text) = json["choices"][0]["text"].as_str() {
        Ok(text.to_string())
    } else {
        Err("OpenAI returned unexpected payload".to_string())
    }
}

async fn call_anthropic_async(api_key: &str, model: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Anthropic request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Anthropic error: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Anthropic parse failed: {}", e))?;
    // Try multiple common fields
    if let Some(text) = json["completion"].as_str() {
        Ok(text.to_string())
    } else if let Some(text) = json["message"]["content"].as_str() {
        Ok(text.to_string())
    } else if let Some(text) = json["output"].as_str() {
        Ok(text.to_string())
    } else {
        Err("Anthropic returned unexpected payload".to_string())
    }
}

fn call_ollama_http(model: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();

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
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama returned error: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(json["response"]
        .as_str()
        .unwrap_or("No response from Ollama")
        .to_string())
}

fn call_ollama_cli(model: &str, prompt: &str) -> Result<String, String> {
    let output = std::process::Command::new("ollama")
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
